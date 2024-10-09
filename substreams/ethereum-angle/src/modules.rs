use crate::{abi, consts};
use anyhow::Result;
use itertools::Itertools;
use std::collections::HashMap;
use substreams::{
    pb::substreams::StoreDeltas,
    store::{StoreAdd, StoreAddBigInt, StoreAddInt64, StoreGet, StoreGetInt64, StoreNew},
};
use substreams_ethereum::{pb::eth, Event};
use tycho_substreams::{
    balances::aggregate_balances_changes, contract::extract_contract_changes, prelude::*,
};

#[substreams::handlers::map]
pub fn map_components(
    params: String,
    block: eth::v2::Block,
) -> Result<BlockTransactionProtocolComponents> {
    let eur_transmuter = hex::decode(params).unwrap();
    let find_second_transmuter = find_usd_transmuter(&eur_transmuter);
    let mut usd_transmuter: [u8; 20] = [0; 20];
    if find_second_transmuter.is_some() {
        usd_transmuter = find_second_transmuter.unwrap();
    }

    // We store these as a hashmap by tx hash since we need to agg by tx hash later
    Ok(BlockTransactionProtocolComponents {
        tx_components: block
            .transactions()
            .filter_map(|tx| {
                let components = tx
                    .logs_with_calls()
                    .filter(|(log, call)| {
                        !call.call.state_reverted && (log.address == usd_transmuter || log.address == eur_transmuter)
                    })
                    .filter_map(|(log, _)| {
                        // event contains collateral added by the governance, examples:
                        // - https://etherscan.io/tx/0xc12328a517e216ee37f974281e019e0041ad755c4868e3b7a8366948ebc55388#eventlog
                        // - https://arbiscan.io/tx/0x821ac64db1ddcdf75c18206a5c4523aabb71b92c80a473a2cddb4ae57b6a4eb1#eventlog
                        let transmuter_address_be = log.address.to_owned();
                        if let Some(ev) =
                            abi::setters_governors_contract::events::CollateralAdded::match_and_decode(log)
                        {
                            let collateral_address = ev.collateral;
                            let is_usd_transmuter = transmuter_address_be == usd_transmuter;
                            let ag_token = find_ag_token(&transmuter_address_be).unwrap();
                            Some(create_vault_component(
                                &tx.into(),
                                &ag_token,
                                &collateral_address,
                                is_usd_transmuter,
                            ))
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

#[substreams::handlers::map]
pub fn map_relative_balances(
    block: eth::v2::Block,
    store: StoreGetInt64,
) -> Result<BlockBalanceDeltas, anyhow::Error> {
    let balance_deltas = block
        .logs()
        .flat_map(|vault_log| {
            let mut deltas = Vec::new();

            let address_bytes_be = vault_log.address();
            let ag_token = find_ag_token(address_bytes_be).unwrap();

            if let Some(ev) = abi::swapper_contract::events::Swap::match_and_decode(vault_log.log) {
                let component_id = component_id_from_tokens(&ag_token, &ev.token_in);
                if store.get_last(&component_id).is_some() {
                    deltas.extend_from_slice(&[
                        BalanceDelta {
                            ord: vault_log.ordinal(),
                            tx: Some(vault_log.receipt.transaction.into()),
                            token: ev.token_in,
                            delta: ev.amount_in.to_signed_bytes_be(),
                            component_id: component_id.as_bytes().to_vec(),
                        },
                        BalanceDelta {
                            ord: vault_log.ordinal(),
                            tx: Some(vault_log.receipt.transaction.into()),
                            token: ev.token_out.to_vec(),
                            delta: ev.amount_out.neg().to_signed_bytes_be(),
                            component_id: component_id.as_bytes().to_vec(),
                        },
                    ]);
                }
            } else if let Some(ev) =
                abi::redeemer_contract::events::Redeemed::match_and_decode(vault_log.log)
            {
                // burn DAI, mint sDAI
                // Tokens mint
                for i in 0..ev.tokens.len() {
                    if !ev
                        .forfeit_tokens
                        .contains(&ev.tokens[i])
                    {
                        let component_id = component_id_from_tokens(&ag_token, &ev.tokens[i]);
                        if store.get_last(&component_id).is_some() {
                            deltas.extend([
                                BalanceDelta {
                                    ord: vault_log.ordinal(),
                                    tx: Some(vault_log.receipt.transaction.into()),
                                    token: ag_token.to_vec(),
                                    delta: ev.amount.neg().to_signed_bytes_be(),
                                    component_id: component_id.as_bytes().to_vec(),
                                },
                                BalanceDelta {
                                    ord: vault_log.ordinal(),
                                    tx: Some(vault_log.receipt.transaction.into()),
                                    token: ev.tokens[i].to_vec(),
                                    delta: ev.amounts[i].to_signed_bytes_be(),
                                    component_id: component_id.as_bytes().to_vec(),
                                },
                            ]);
                        }
                    }
                }
            } else if let Some(ev) =
                abi::setters_governors_contract::events::ReservesAdjusted::match_and_decode(
                    vault_log.log,
                )
            {
                let ag_token = find_ag_token(address_bytes_be).unwrap();
                let component_id = component_id_from_tokens(&ag_token, &ev.collateral);
                let delta = if ev.increase { ev.amount } else { ev.amount.neg() };
                deltas.push(BalanceDelta {
                    ord: vault_log.ordinal(),
                    tx: Some(vault_log.receipt.transaction.into()),
                    token: ev.collateral,
                    delta: delta.to_signed_bytes_be(),
                    component_id: component_id.as_bytes().to_vec(),
                });
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

fn create_vault_component(
    tx: &Transaction,
    collateral_address: &[u8],
    ag_token_address: &[u8],
    usd_transmuter: bool,
) -> ProtocolComponent {
    let transmuter_attribute = if usd_transmuter { "USD" } else { "EUR" };
    let id = component_id_from_tokens(ag_token_address, collateral_address);
    ProtocolComponent::new(&id, tx)
        .as_swap_type("ANGLE_TRANSMUTER", ImplementationType::Vm)
        .with_tokens(&[collateral_address, ag_token_address])
        .with_attributes(&[("Transmuter Type", transmuter_attribute)])
}

fn find_usd_transmuter(eur_transmuter: &[u8]) -> Option<[u8; 20]> {
    for i in 0..consts::TRANSMUTERS_USD.len() {
        if !consts::TRANSMUTERS_EUR[i].is_empty() && consts::TRANSMUTERS_EUR[i] == eur_transmuter {
            return Some(consts::TRANSMUTERS_USD[i]);
        }
    }
    None
}

// agToken is the token burnt or minted, obtained by transmuter.agToken()
fn find_ag_token(transmuter: &[u8]) -> Option<[u8; 20]> {
    // Transmuter is EUR
    for i in 0..consts::TRANSMUTERS_EUR.len() {
        if !consts::TRANSMUTERS_EUR[i].is_empty() && consts::TRANSMUTERS_EUR[i] == transmuter {
            return Some(consts::AGTOKENS_EUR[i]);
        }
    }

    // Transmuter is USD
    for j in 0..consts::TRANSMUTERS_USD.len() {
        if !consts::TRANSMUTERS_USD[j].is_empty() && consts::TRANSMUTERS_USD[j] == transmuter {
            return Some(consts::AGTOKENS_USD[j]);
        }
    }

    None
}

fn component_id_from_tokens(ag_token: &[u8], collateral: &[u8]) -> String {
    format!("{}:{}", hex::encode(ag_token), hex::encode(collateral))
}
