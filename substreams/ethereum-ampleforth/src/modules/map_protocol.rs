use anyhow::Result;
use itertools::Itertools;
use std::collections::HashMap;
use substreams::{
    pb::substreams::StoreDeltas,
    scalar::BigInt,
    store::{
        StoreAdd, StoreAddBigInt, StoreAddInt64, StoreGet, StoreGetBigInt, StoreGetInt64, StoreNew,
    },
};
use substreams_ethereum::{
    block_view::LogView,
    pb::eth::{self},
    Event,
};
use tycho_substreams::{
    balances::aggregate_balances_changes, contract::extract_contract_changes, prelude::*,
};

use crate::abi;
use crate::modules::map_and_store_ampl::{
    fetch_balance_delta_after_rebase, AMPL_ADDRESS_HEX, INTERESTED_POOLS, INTERESTED_TOKENS,
    WAMPL_ADDRESS_HEX, ZERO_ADDRESS_HEX,
};
use crate::pb::ampleforth::AmplRebases;

// --------------
// Keep track of pools
// --------------

#[substreams::handlers::map]
pub fn map_components(
    block: eth::v2::Block,
) -> Result<BlockTransactionProtocolComponents, anyhow::Error> {
    let ampl_addr = hex::decode(AMPL_ADDRESS_HEX).unwrap();
    let wampl_addr = hex::decode(WAMPL_ADDRESS_HEX).unwrap();

    let mut tx_components = Vec::new();

    for tx in block.transactions() {
        let mut created_accounts = Vec::new();
        for call in &tx.calls {
            for account_creation in &call.account_creations {
                created_accounts.push(account_creation.account.clone());
            }
        }

        let mut components = Vec::new();
        if created_accounts.contains(&wampl_addr) {
            let tx_converted = tx.into();
            let comp = ProtocolComponent::at_contract(&wampl_addr, &tx_converted)
                .with_tokens(&[ampl_addr.as_slice(), wampl_addr.as_slice()])
                .as_swap_type("wampl_wrapper", ImplementationType::Vm);
            substreams::log::debug!("pool:{}", comp.id);
            components.push(comp);
        }

        if !components.is_empty() {
            let tx_converted = tx.into();
            let tx_proto_components =
                TransactionProtocolComponents { tx: Some(tx_converted), components };
            tx_components.push(tx_proto_components);
        }
    }

    Ok(BlockTransactionProtocolComponents { tx_components })
}

#[substreams::handlers::store]
pub fn store_components(map: BlockTransactionProtocolComponents, store: StoreAddInt64) {
    store.add_many(
        0,
        &map.tx_components
            .iter()
            .flat_map(|tx_components| &tx_components.components)
            .map(|component| format!("pool:{0}", component.id))
            .collect::<Vec<_>>(),
        1,
    );
}

// -----------------------------------------------------------------------------
// Keep track of pool balances
// -----------------------------------------------------------------------------

#[substreams::handlers::map]
pub fn map_relative_balances(
    block: eth::v2::Block,
    ampl_supply_store: StoreGetBigInt,
    ampl_gon_balance_store: StoreGetBigInt,
) -> Result<BlockBalanceDeltas, anyhow::Error> {
    let mut balance_deltas = Vec::new();
    for tx_log in block.logs() {
        balance_deltas.extend(extract_balance_deltas_from_transfer(&tx_log));
        balance_deltas.extend(extract_balance_deltas_from_rebase(
            &tx_log,
            &ampl_supply_store,
            &ampl_gon_balance_store,
        ));
    }

    if balance_deltas.len() > 0 {
        substreams::log::debug!("delta_registered");
    }

    Ok(BlockBalanceDeltas { balance_deltas })
}

fn extract_balance_deltas_from_transfer(tx_log: &LogView) -> Vec<BalanceDelta> {
    let mut deltas = vec![];

    if let Some(ev) = abi::erc20_contract::events::Transfer::match_and_decode(tx_log.log) {
        let token_addr = tx_log.address();
        let token_hex = hex::encode(token_addr);
        let from_hex = hex::encode(ev.from);
        let to_hex = hex::encode(ev.to);

        let is_token_of_interest = INTERESTED_TOKENS.contains(&token_hex.as_str());
        if is_token_of_interest {
            substreams::log::debug!("is_token_of_interest:{}", is_token_of_interest);
        }

        let is_token_out = INTERESTED_POOLS.contains(&from_hex.as_str());
        let is_token_in = INTERESTED_POOLS.contains(&to_hex.as_str());

        if is_token_of_interest && is_token_out {
            substreams::log::debug!("pool_out");
            deltas.push(BalanceDelta {
                ord: tx_log.ordinal(),
                tx: Some(tx_log.receipt.transaction.into()),
                token: token_addr.to_vec(),
                delta: ev.value.neg().to_signed_bytes_be(),
                component_id: format!("0x{}", from_hex).as_bytes().to_vec(),
            });
        }

        if is_token_of_interest && is_token_in {
            substreams::log::debug!("in_out");
            deltas.push(BalanceDelta {
                ord: tx_log.ordinal(),
                tx: Some(tx_log.receipt.transaction.into()),
                token: token_addr.to_vec(),
                delta: ev.value.to_signed_bytes_be(),
                component_id: format!("0x{}", to_hex).as_bytes().to_vec(),
            });
        }

        // NOTE: We have special handling for the WAMPL pool,
        // because tokens are minted/burnt by transferring from/to address(0).
        // The "virtual" balance of the pool is the total supply of wampl tokens in circulation,
        // which is calculated by the sum of all mints - sum of all burns.
        if token_hex == WAMPL_ADDRESS_HEX && from_hex == ZERO_ADDRESS_HEX {
            substreams::log::debug!("wampl_mint");
            deltas.push(BalanceDelta {
                ord: tx_log.ordinal(),
                tx: Some(tx_log.receipt.transaction.into()),
                token: token_addr.to_vec(),
                delta: ev.value.to_signed_bytes_be(),
                component_id: format!("0x{}", token_hex).as_bytes().to_vec(),
            });
        }

        if token_hex == WAMPL_ADDRESS_HEX && to_hex == ZERO_ADDRESS_HEX {
            substreams::log::debug!("wampl_burn");
            deltas.push(BalanceDelta {
                ord: tx_log.ordinal(),
                tx: Some(tx_log.receipt.transaction.into()),
                token: token_addr.to_vec(),
                delta: ev.value.neg().to_signed_bytes_be(),
                component_id: format!("0x{}", token_hex).as_bytes().to_vec(),
            });
        }
    }

    deltas
}

fn extract_balance_deltas_from_rebase(
    tx_log: &LogView,
    ampl_supply_store: &StoreGetBigInt,
    ampl_gon_balance_store: &StoreGetBigInt,
) -> Vec<BalanceDelta> {
    let mut deltas = Vec::new();
    if let Some(_ev) = abi::ampl_contract::events::LogRebase::match_and_decode(tx_log.log) {
        let token_addr = tx_log.address();
        let token_hex = hex::encode(token_addr);
        if token_hex == AMPL_ADDRESS_HEX {
            substreams::log::debug!("handing_rebase_scaling");
            for pool_hex in INTERESTED_POOLS {
                let delta = fetch_balance_delta_after_rebase(
                    ampl_supply_store,
                    ampl_gon_balance_store,
                    pool_hex,
                );
                substreams::log::debug!("rebase_delta:{}", delta);
                if delta != BigInt::zero() {
                    deltas.push(BalanceDelta {
                        ord: tx_log.ordinal(),
                        tx: Some(tx_log.receipt.transaction.into()),
                        token: token_addr.to_vec(),
                        delta: delta.to_signed_bytes_be(),
                        component_id: format!("0x{}", pool_hex).as_bytes().to_vec(),
                    });
                }
            }
        }
    }
    deltas
}

#[substreams::handlers::store]
pub fn store_balances(deltas: BlockBalanceDeltas, balance_store: StoreAddBigInt) {
    tycho_substreams::balances::store_balance_changes(deltas, balance_store);
}

// -----------------------------------------------------------------------------
// Keep track of storage state
// -----------------------------------------------------------------------------
#[substreams::handlers::map]
pub fn map_protocol_changes(
    block: eth::v2::Block,
    rebases: AmplRebases,
    grouped_components: BlockTransactionProtocolComponents,
    deltas: BlockBalanceDeltas,
    _components_store: StoreGetInt64,
    balance_store: StoreDeltas,
) -> Result<BlockChanges, anyhow::Error> {
    let mut transaction_contract: HashMap<u64, TransactionChanges> = HashMap::new();

    grouped_components
        .tx_components
        .iter()
        .for_each(|tx_component| {
            let tx = tx_component.tx.as_ref().unwrap();
            transaction_contract
                .entry(tx.index)
                .or_insert_with(|| TransactionChanges::new(tx))
                .component_changes
                .extend_from_slice(&tx_component.components);
        });

    aggregate_balances_changes(balance_store, deltas)
        .into_iter()
        .for_each(|(_, (tx, balances))| {
            let tx_change = transaction_contract
                .entry(tx.index)
                .or_insert_with(|| TransactionChanges::new(&tx));

            balances
                .into_values()
                .for_each(|token_bc_map| {
                    tx_change
                        .balance_changes
                        .extend(token_bc_map.into_values());
                });
        });

    let block_has_rebase = rebases.rebase_events.len() > 0;
    extract_contract_changes(
        &block,
        |addr| {
            let addr_hex = hex::encode(addr);
            let is_component_call = INTERESTED_POOLS.contains(&addr_hex.as_str());
            let is_rebase_call = block_has_rebase && addr_hex == AMPL_ADDRESS_HEX;
            is_rebase_call || is_component_call
        },
        &mut transaction_contract,
    );

    Ok(BlockChanges {
        block: Some((&block).into()),
        changes: transaction_contract
            .drain()
            .sorted_unstable_by_key(|(index, _)| *index)
            .filter_map(|(_, change)| {
                if change.contract_changes.is_empty()
                    && change.component_changes.is_empty()
                    && change.balance_changes.is_empty()
                {
                    None
                } else {
                    Some(change)
                }
            })
            .collect::<Vec<_>>(),
    })
}
