use crate::abi;
use anyhow::Result;
use itertools::Itertools;
use std::collections::HashMap;
use substreams::{
    hex,
    pb::substreams::StoreDeltas,
    store::{StoreAdd, StoreAddBigInt, StoreAddInt64, StoreGet, StoreGetInt64, StoreNew},
};
use substreams_ethereum::{pb::eth, Event};
use tycho_substreams::{
    balances::aggregate_balances_changes, contract::extract_contract_changes, prelude::*,
};

const RETH_ADDRESS: [u8; 20] = hex!("D33526068D116cE69F19A9ee46F0bd304F21A51f"); //RPL rocketPool Token
const VAULT_ADDRESS: [u8; 20] = hex!("3bDC69C4E5e13E52A65f5583c23EFB9636b469d6"); //vault
const ETHER_TOKEN_ADDRESS: [u8; 20] = hex!("0000000000000000000000000000000000000000"); //ETH

#[substreams::handlers::map]
pub fn map_components(block: eth::v2::Block) -> Result<BlockTransactionProtocolComponents> {
    // We store these as a hashmap by tx hash since we need to agg by tx hash later
    Ok(BlockTransactionProtocolComponents {
        tx_components: block
            .transactions()
            .filter_map(|tx| {
                let components = tx
                    .calls()
                    .filter(|call| !call.call.state_reverted)
                    .filter_map(|_| {
                        if is_deployment_tx(tx, &VAULT_ADDRESS) {
                            Some(create_vault_component(&tx.into()))
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();

                if !components.is_empty() {
                    Some(TransactionProtocolComponents { tx: Some(tx.into()), components })
                } else {
                    None
                }
            })
            .collect::<Vec<_>>(),
    })
}

/// Simply stores the `ProtocolComponent`s with the pool id as the key
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

/// This is the main map that handles most of the indexing of this substream.
/// Every contract change is grouped by transaction index via the `transaction_contract_changes`
///  map. Each block of code will extend the `TransactionContractChanges` struct with the
///  cooresponding changes (balance, component, contract), inserting a new one if it doesn't exist.
///  At the very end, the map can easily be sorted by index to ensure the final
/// `BlockContractChanges`  is ordered by transactions properly.
#[substreams::handlers::map]
pub fn map_protocol_changes(
    block: eth::v2::Block,
    grouped_components: BlockTransactionProtocolComponents,
    deltas: BlockBalanceDeltas,
    components_store: StoreGetInt64,
    balance_store: StoreDeltas, // Note, this map module is using the `deltas` mode for the store.
) -> Result<BlockContractChanges> {
    // We merge contract changes by transaction (identified by transaction index) making it easy to
    //  sort them at the very end.
    let mut transaction_contract_changes: HashMap<_, TransactionContractChanges> = HashMap::new();

    // `ProtocolComponents` are gathered from `map_pools_created` which just need a bit of work to
    //   convert into `TransactionContractChanges`
    grouped_components
        .tx_components
        .iter()
        .for_each(|tx_component| {
            let tx = tx_component.tx.as_ref().unwrap();
            transaction_contract_changes
                .entry(tx.index)
                .or_insert_with(|| TransactionContractChanges::new(tx))
                .component_changes
                .extend_from_slice(&tx_component.components);
        });

    // Balance changes are gathered by the `StoreDelta` based on `PoolBalanceChanged` creating
    //  `BlockBalanceDeltas`. We essentially just process the changes that occurred to the `store`
    // this  block. Then, these balance changes are merged onto the existing map of tx contract
    // changes,  inserting a new one if it doesn't exist.
    aggregate_balances_changes(balance_store, deltas)
        .into_iter()
        .for_each(|(_, (tx, balances))| {
            transaction_contract_changes
                .entry(tx.index)
                .or_insert_with(|| TransactionContractChanges::new(&tx))
                .balance_changes
                .extend(balances.into_values());
        });

    // Extract and insert any storage changes that happened for any of the components.
    extract_contract_changes(
        &block,
        |addr| {
            components_store
                .get_last(format!("pool:0x{0}", hex::encode(addr)))
                .is_some()
        },
        &mut transaction_contract_changes,
    );

    // Process all `transaction_contract_changes` for final output in the `BlockContractChanges`,
    //  sorted by transaction index (the key).
    Ok(BlockContractChanges {
        block: Some((&block).into()),
        changes: transaction_contract_changes
            .drain()
            .sorted_unstable_by_key(|(index, _)| *index)
            .filter_map(|(_, change)| {
                if change.contract_changes.is_empty() &&
                    change.component_changes.is_empty() &&
                    change.balance_changes.is_empty()
                {
                    None
                } else {
                    Some(change)
                }
            })
            .collect::<Vec<_>>(),
    })
}

#[substreams::handlers::map]
pub fn map_relative_balances(
    block: eth::v2::Block,
    store: StoreGetInt64,
) -> Result<BlockBalanceDeltas, anyhow::Error> {
    let balance_deltas = block
        .logs()
        .filter(|log| log.address() == VAULT_ADDRESS)
        .flat_map(|vault_log| {
            let mut deltas = Vec::new();
            let address_bytes_be = vault_log.address();
            let address_hex = format!("0x{}", hex::encode(address_bytes_be));

            if store
                .get_last(format!("pool:{}", address_hex))
                .is_some()
            {
                if let Some(ev) =
                    abi::rocketvault_contract::events::EtherDeposited::match_and_decode(
                        vault_log.log,
                    )
                {
                    deltas.push(BalanceDelta {
                        ord: vault_log.ordinal(),
                        tx: Some(vault_log.receipt.transaction.into()),
                        token: ETHER_TOKEN_ADDRESS.to_vec(), // ETH address
                        delta: ev.amount.to_signed_bytes_be(),
                        component_id: address_hex.as_bytes().to_vec(),
                    });
                } else if let Some(ev) =
                    abi::rocketvault_contract::events::EtherWithdrawn::match_and_decode(
                        vault_log.log,
                    )
                {
                    deltas.push(BalanceDelta {
                        ord: vault_log.ordinal(),
                        tx: Some(vault_log.receipt.transaction.into()),
                        token: ETHER_TOKEN_ADDRESS.to_vec(), // ETH address
                        delta: ev.amount.neg().to_signed_bytes_be(),
                        component_id: address_hex.as_bytes().to_vec(),
                    });
                } else if let Some(ev) =
                    abi::rocketvault_contract::events::TokenDeposited::match_and_decode(
                        vault_log.log,
                    )
                {
                    deltas.push(BalanceDelta {
                        ord: vault_log.ordinal(),
                        tx: Some(vault_log.receipt.transaction.into()),
                        token: RETH_ADDRESS.to_vec(),
                        delta: ev.amount.to_signed_bytes_be(),
                        component_id: address_hex.as_bytes().to_vec(),
                    });
                } else if let Some(ev) =
                    abi::rocketvault_contract::events::TokenWithdrawn::match_and_decode(
                        vault_log.log,
                    )
                {
                    deltas.push(BalanceDelta {
                        ord: vault_log.ordinal(),
                        tx: Some(vault_log.receipt.transaction.into()),
                        token: RETH_ADDRESS.to_vec(),
                        delta: ev.amount.neg().to_signed_bytes_be(),
                        component_id: address_hex.as_bytes().to_vec(),
                    });
                } else if let Some(ev) =
                    abi::rocketvault_contract::events::TokenBurned::match_and_decode(vault_log.log)
                {
                    deltas.push(BalanceDelta {
                        ord: vault_log.ordinal(),
                        tx: Some(vault_log.receipt.transaction.into()),
                        token: RETH_ADDRESS.to_vec(),
                        delta: ev.amount.neg().to_signed_bytes_be(),
                        component_id: address_hex.as_bytes().to_vec(),
                    });
                }
            }
            deltas
        })
        .collect::<Vec<_>>();

    Ok(BlockBalanceDeltas { balance_deltas })
}

/// It's significant to include both the `pool_id` and the `token_id` for each balance delta as the
///  store key to ensure that there's a unique balance being tallied for each.
#[substreams::handlers::store]
pub fn store_balances(deltas: BlockBalanceDeltas, store: StoreAddBigInt) {
    tycho_substreams::balances::store_balance_changes(deltas, store);
}

fn is_deployment_tx(tx: &eth::v2::TransactionTrace, vault_address: &[u8]) -> bool {
    let created_accounts = tx
        .calls
        .iter()
        .flat_map(|call| {
            call.account_creations
                .iter()
                .map(|ac| ac.account.to_owned())
        })
        .collect::<Vec<_>>();

    if let Some(deployed_address) = created_accounts.first() {
        return deployed_address.as_slice() == vault_address;
    }
    false
}

fn create_vault_component(tx: &Transaction) -> ProtocolComponent {
    ProtocolComponent::at_contract(RETH_ADDRESS.as_slice(), tx)
        .with_tokens(&[RETH_ADDRESS, ETHER_TOKEN_ADDRESS])
        .as_swap_type("rocketvault", ImplementationType::Vm)
}
