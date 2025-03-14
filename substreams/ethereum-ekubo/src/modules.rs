use std::collections::HashMap;
use anyhow::{ensure, Context, Result};
use substreams::pb::substreams::StoreDeltas;
use substreams::prelude::*;
use substreams_ethereum::Event;
use substreams_ethereum::pb::eth;
use tycho_substreams::balances::aggregate_balances_changes;
use tycho_substreams::contract::extract_contract_changes_builder;
use tycho_substreams::prelude::*;
use itertools::Itertools;
use substreams::hex;
use crate::identifiers::pool_id_to_component_id;
use crate::pb::ekubo::PoolDetails;
use crate::pool_factories;
use crate::pool_factories::DeploymentConfig;
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
                    .collect_vec();

                (!components.is_empty()).then(|| TransactionProtocolComponents { tx: Some(tx.into()), components })
            })
            .collect_vec(),
    })
}

#[substreams::handlers::store]
fn store_pool_details(components: BlockTransactionProtocolComponents, store: StoreSetProto<PoolDetails>) {
    let components = components
        .tx_components
        .into_iter()
        .flat_map(|comp| comp.components);

    for component in components {
        let attrs = component.static_att;

        let pool_details = PoolDetails {
            token0: attrs[0].value.clone(),
            token1: attrs[1].value.clone(),
            fee: u64::from_be_bytes(attrs[2].value.clone().try_into().unwrap()),
        };

        store.set(0, component.id, &pool_details);
    }
}

/// Extracts balance changes per component
///
/// Indexes balance changes according to Ekubo core events.
#[substreams::handlers::map]
fn map_relative_component_balance(params: String, block: eth::v2::Block, store: StoreGetProto<PoolDetails>) -> Result<BlockBalanceDeltas> {
    let config: DeploymentConfig = serde_qs::from_str(params.as_str())?;
    let res = block
        .transactions()
        .map(|tx| {
            tx
                .logs_with_calls()
                .map(|(log, _)| {
                    if log.address != config.core {
                        return Ok(vec![]);
                    }

                    Ok(if let Some(ev) = core_events::PositionUpdated::match_and_decode(log) {
                        let component_id = pool_id_to_component_id(ev.pool_id);
                        let pool_details = get_pool_details(&store, &component_id)?;

                        vec![
                            BalanceDelta {
                                ord: log.ordinal,
                                tx: Some(tx.into()),
                                token: pool_details.token0,
                                delta: adjust_delta_by_fee(ev.delta0, pool_details.fee).to_signed_bytes_be(),
                                component_id: component_id.clone().into(),
                            },
                            BalanceDelta {
                                ord: log.ordinal,
                                tx: Some(tx.into()),
                                token: pool_details.token1,
                                delta: adjust_delta_by_fee(ev.delta1, pool_details.fee).to_signed_bytes_be(),
                                component_id: component_id.into(),
                            }
                        ]
                    } else if let Some(ev) = core_events::PositionFeesCollected::match_and_decode(log) {
                        let component_id = pool_id_to_component_id(ev.pool_id);
                        let pool_details = get_pool_details(&store, &component_id)?;

                        vec![
                            BalanceDelta {
                                ord: log.ordinal,
                                tx: Some(tx.into()),
                                token: pool_details.token0,
                                delta: ev.amount0.neg().to_signed_bytes_be(),
                                component_id: ev.pool_id.into(),
                            },
                            BalanceDelta {
                                ord: log.ordinal,
                                tx: Some(tx.into()),
                                token: pool_details.token1,
                                delta: ev.amount1.neg().to_signed_bytes_be(),
                                component_id: component_id.into(),
                            }
                        ]
                    } else if let Some(ev) = core_events::FeesAccumulated::match_and_decode(log) {
                        let component_id = pool_id_to_component_id(ev.pool_id);
                        let pool_details = get_pool_details(&store, &component_id)?;

                        vec![
                            BalanceDelta {
                                ord: log.ordinal,
                                tx: Some(tx.into()),
                                token: pool_details.token0,
                                delta: ev.amount0.to_signed_bytes_be(),
                                component_id: component_id.clone().into(),
                            },
                            BalanceDelta {
                                ord: log.ordinal,
                                tx: Some(tx.into()),
                                token: pool_details.token1,
                                delta: ev.amount1.to_signed_bytes_be(),
                                component_id: component_id.into(),
                            }
                        ]
                    } else if log.topics.is_empty() {
                        let data = &log.data;

                        ensure!(data.len() == 116, "swapped event data length mismatch");

                        let component_id = pool_id_to_component_id(&data[20..52]);
                        let (delta0, delta1) = (
                            i128::from_be_bytes(data[52..68].try_into().unwrap()),
                            i128::from_be_bytes(data[68..84].try_into().unwrap()),
                        );

                        let pool_details = get_pool_details(&store, &component_id)?;

                        vec![
                            BalanceDelta {
                                ord: log.ordinal,
                                tx: Some(tx.into()),
                                token: pool_details.token0,
                                delta: delta0.to_be_bytes().into(),
                                component_id: component_id.clone().into(),
                            },
                            BalanceDelta {
                                ord: log.ordinal,
                                tx: Some(tx.into()),
                                token: pool_details.token1,
                                delta: delta1.to_be_bytes().into(),
                                component_id: component_id.into(),
                            }
                        ]
                    } else {
                        vec![]
                    })
                })
                .try_collect()
                .with_context(|| format!("handling tx {}", hex::encode(&tx.hash)))
        })
        .collect::<Result<Vec<Vec<Vec<BalanceDelta>>>>>()?
        .into_iter()
        .flatten()
        .flatten()
        .collect();

    Ok(BlockBalanceDeltas { balance_deltas: res })
}

fn get_pool_details(store: &StoreGetProto<PoolDetails>, component_id: &str) -> Result<PoolDetails> {
    store
        .get_at(0, component_id)
        .context("pool id should exist in store")
}


fn adjust_delta_by_fee(delta: BigInt, fee: u64) -> BigInt {
    if delta < BigInt::zero() {
        let denom = BigInt::from_signed_bytes_be(&hex!("0100000000000000000000000000000000"));
        (delta * denom.clone()) / (denom - fee)
    } else {
        delta
    }
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
