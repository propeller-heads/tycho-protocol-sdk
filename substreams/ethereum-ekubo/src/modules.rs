//! Template for Protocols with singleton contract
//!
//!
use std::collections::HashMap;
use anyhow::Result;
use substreams::pb::substreams::StoreDeltas;
use substreams::prelude::*;
use substreams_ethereum::Event;
use substreams_ethereum::pb::eth;
use tycho_substreams::balances::aggregate_balances_changes;
use tycho_substreams::contract::extract_contract_changes_builder;
use tycho_substreams::prelude::*;
use itertools::Itertools;
use crate::pool_factories;
use crate::pool_factories::{hash_pool_key, DeploymentConfig};
use crate::abi::core::events as core_events;

/// Find and create all relevant protocol components
///
/// This method maps over blocks and instantiates ProtocolComponents with a unique ids
/// as well as all necessary metadata for routing and encoding.
#[substreams::handlers::map]
fn map_protocol_components(
    params: String,
    block: eth::v2::Block,
) -> Result<BlockTransactionProtocolComponents> {
    let config = serde_qs::from_str(params.as_str())?;
    Ok(BlockTransactionProtocolComponents {
        tx_components: block
            .transactions()
            .filter_map(|tx| {
                let components = tx
                    .logs_with_calls()
                    .filter_map(|(log, call)| {
                        pool_factories::maybe_create_component(
                            call.call,
                            log,
                            tx,
                            &config,
                        )
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

/// Extracts balance changes per component
///
/// Indexes balance changes according to Ekubo core events.
#[substreams::handlers::map]
fn map_relative_component_balance(params: String, block: eth::v2::Block) -> Result<BlockBalanceDeltas> {
    let config: DeploymentConfig = serde_qs::from_str(params.as_str())?;
    let res = block
        .transactions()
        .flat_map(|tx| {
            tx.logs_with_calls()
                .map(|(log, _)| {
                    if log.address != config.core {
                        return vec![];
                    }
                    if let Some(ev) = core_events::PositionUpdated::match_and_decode(log) {
                        let pool_id = hash_pool_key(&ev.pool_key);

                        vec![
                            BalanceDelta {
                                ord: log.ordinal,
                                tx: Some(tx.into()),
                                token: ev.pool_key.0.clone(),
                                delta: ev.delta0.to_signed_bytes_be(),
                                component_id: pool_id.clone().into(),
                            },
                            BalanceDelta {
                                ord: log.ordinal,
                                tx: Some(tx.into()),
                                token: ev.pool_key.1.clone(),
                                delta: ev.delta1.to_signed_bytes_be(),
                                component_id: pool_id.into(),
                            }
                        ]
                    } else if let Some(ev) = core_events::PositionFeesCollected::match_and_decode(log) {
                        let pool_id = hash_pool_key(&ev.pool_key);

                        vec![
                            BalanceDelta {
                                ord: log.ordinal,
                                tx: Some(tx.into()),
                                token: ev.pool_key.0.clone(),
                                delta: ev.amount0.neg().to_signed_bytes_be(),
                                component_id: pool_id.clone().into(),
                            },
                            BalanceDelta {
                                ord: log.ordinal,
                                tx: Some(tx.into()),
                                token: ev.pool_key.1.clone(),
                                delta: ev.amount1.neg().to_signed_bytes_be(),
                                component_id: pool_id.into(),
                            }
                        ]
                    } else if let Some(ev) = core_events::Swapped::match_and_decode(log) {
                        let pool_id = hash_pool_key(&ev.pool_key);
                        vec![
                            BalanceDelta {
                                ord: log.ordinal,
                                tx: Some(tx.into()),
                                token: ev.pool_key.0.clone(),
                                delta: ev.delta0.to_signed_bytes_be(),
                                component_id: pool_id.clone().into(),
                            },
                            BalanceDelta {
                                ord: log.ordinal,
                                tx: Some(tx.into()),
                                token: ev.pool_key.1.clone(),
                                delta: ev.delta1.to_signed_bytes_be(),
                                component_id: pool_id.into(),
                            }
                        ]
                    } else if let Some(ev) = core_events::FeesAccumulated::match_and_decode(log) {
                        let pool_id = hash_pool_key(&ev.pool_key);
                        vec![
                            BalanceDelta {
                                ord: log.ordinal,
                                tx: Some(tx.into()),
                                token: ev.pool_key.0.clone(),
                                delta: ev.amount0.to_signed_bytes_be(),
                                component_id: pool_id.clone().into(),
                            },
                            BalanceDelta {
                                ord: log.ordinal,
                                tx: Some(tx.into()),
                                token: ev.pool_key.1.clone(),
                                delta: ev.amount1.to_signed_bytes_be(),
                                component_id: pool_id.into(),
                            }
                        ]
                    } else {
                        vec![]
                    }
                })
        })
        .flatten()
        .collect::<Vec<_>>();

    Ok(BlockBalanceDeltas { balance_deltas: res })
}

/// Aggregates relative balances values into absolute values
///
/// Aggregate the relative balances in an additive store since tycho-indexer expects
/// absolute balance inputs.
#[substreams::handlers::store]
pub fn store_component_balances(deltas: BlockBalanceDeltas, store: StoreAddBigInt) {
    tycho_substreams::balances::store_balance_changes(deltas, store);
}

/// Aggregates protocol components and balance changes by transaction.
///
/// This is the main method that will aggregate all changes as well as extract all
/// relevant contract storage deltas.
#[substreams::handlers::map]
fn map_protocol_changes(
    params: String,
    block: eth::v2::Block,
    new_components: BlockTransactionProtocolComponents,
    deltas: BlockBalanceDeltas,
    balance_store: StoreDeltas,
) -> Result<BlockChanges, substreams::errors::Error> {
    let config: DeploymentConfig = serde_qs::from_str(params.as_str())?;
    // We merge contract changes by transaction (identified by transaction index)
    // making it easy to sort them at the very end.
    let mut transaction_changes: HashMap<_, TransactionChangesBuilder> = HashMap::new();

    // Aggregate newly created components per tx
    new_components
        .tx_components
        .iter()
        .for_each(|tx_component| {
            // initialise builder if not yet present for this tx
            let tx = tx_component.tx.as_ref().unwrap();
            let builder = transaction_changes
                .entry(tx.index)
                .or_insert_with(|| TransactionChangesBuilder::new(tx));

            // iterate over individual components created within this tx
            tx_component
                .components
                .iter()
                .for_each(|component| {
                    builder.add_protocol_component(component);
                });
        });

    // Aggregate absolute balances per transaction.
    aggregate_balances_changes(balance_store, deltas)
        .into_iter()
        .for_each(|(_, (tx, balances))| {
            let builder = transaction_changes
                .entry(tx.index)
                .or_insert_with(|| TransactionChangesBuilder::new(&tx));
            let mut contract_changes = InterimContractChange::new(&config.core, false);
            balances
                .values()
                .for_each(|token_bc_map| {
                    token_bc_map
                        .values()
                        .for_each(|bc| {
                            // track component balance
                            builder.add_balance_change(bc);
                            // track vault contract balance
                            contract_changes.upsert_token_balance(bc.token.as_slice(), bc.balance.as_slice())
                        })
                });
            builder.add_contract_changes(&contract_changes);
        });


    // Extract and insert any storage changes that happened for any of the components.
    extract_contract_changes_builder(
        &block,
        |addr| {
            (addr == config.core) || (addr == config.oracle)
        },
        &mut transaction_changes,
    );

    // Process all `transaction_changes` for final output in the `BlockChanges`,
    //  sorted by transaction index (the key).
    Ok(BlockChanges {
        block: Some((&block).into()),
        changes: transaction_changes
            .drain()
            .sorted_unstable_by_key(|(index, _)| *index)
            .filter_map(|(_, builder)| builder.build())
            .collect::<Vec<_>>(),
    })
}