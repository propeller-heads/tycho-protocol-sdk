use crate::{abi, pool_factories};
use anyhow::Result;
use itertools::Itertools;
use std::collections::HashMap;
use substreams::{
    hex,
    pb::substreams::StoreDeltas,
    store::{
        StoreAddBigInt, StoreGet, StoreGetInt64, StoreGetProto, StoreNew, StoreSet, StoreSetProto,
    },
};
use substreams_ethereum::{pb::eth, Event};
use tycho_substreams::{
    balances::aggregate_balances_changes, contract::extract_contract_changes, prelude::*,
};

pub const XGRAIL_ADDRESS: [u8; 20] = hex!("8D9bA570D6cb60C7e3e0F31346Efe05AB882Aa54");
pub const GRAIL_ADDRESS: [u8; 20] = hex!("D3d2E2692501A5c9Ca623199D38826e513033a17");

#[substreams::handlers::map]
pub fn map_components(block: eth::v2::Block) -> Result<BlockTransactionProtocolComponents> {
    // Gather contract changes by indexing `PoolCreated` events and analysing the `Create` call
    // We store these as a hashmap by tx hash since we need to agg by tx hash later
    Ok(BlockTransactionProtocolComponents {
        tx_components: block
            .transactions()
            .filter_map(|tx| {
                let components = tx
                    .logs_with_calls()
                    .filter_map(|(log, _)| pool_factories::address_map(log, &tx))
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
pub fn store_components(
    map: BlockTransactionProtocolComponents,
    store: StoreSetProto<ProtocolComponent>,
) {
    map.tx_components
        .iter()
        .for_each(|tx_components| {
            tx_components
                .components
                .iter()
                .for_each(|component| store_component(&store, component));
        })
}

/// Since the `PoolBalanceChanged` and `Swap` events administer only deltas, we need to leverage a
/// map and a  store to be able to tally up final balances for tokens in a pool.
#[substreams::handlers::map]
pub fn map_relative_balances(
    block: eth::v2::Block,
    store: StoreGetProto<ProtocolComponent>,
) -> Result<BlockBalanceDeltas, anyhow::Error> {
    let balance_deltas = block
        .logs()
        .flat_map(|log| {
            let mut deltas = Vec::new();

            if let Some(ev) = abi::xgrail::events::Convert::match_and_decode(log) {
                let component_id = address_to_hex_string(&XGRAIL_ADDRESS);

                if maybe_get_pool_tokens(&store, &component_id).is_some() {
                    deltas.extend_from_slice(&[
                        BalanceDelta {
                            ord: log.ordinal(),
                            tx: Some(log.receipt.transaction.into()),
                            token: XGRAIL_ADDRESS.to_vec(),
                            delta: ev.amount.to_signed_bytes_be(),
                            component_id: string_to_bytes_vec(&component_id),
                        },
                        BalanceDelta {
                            ord: log.ordinal(),
                            tx: Some(log.receipt.transaction.into()),
                            token: GRAIL_ADDRESS.to_vec(),
                            delta: ev.amount.to_signed_bytes_be(),
                            component_id: string_to_bytes_vec(&component_id),
                        },
                    ]);
                }
            } else if let Some(ev) = abi::xgrail::events::FinalizeRedeem::match_and_decode(log) {
                let component_id = address_to_hex_string(&XGRAIL_ADDRESS);

                if maybe_get_pool_tokens(&store, &component_id).is_some() {
                    deltas.extend_from_slice(&[
                        BalanceDelta {
                            ord: log.ordinal(),
                            tx: Some(log.receipt.transaction.into()),
                            token: XGRAIL_ADDRESS.to_vec(),
                            delta: ev
                                .x_grail_amount
                                .neg()
                                .to_signed_bytes_be(),
                            component_id: string_to_bytes_vec(&component_id),
                        },
                        BalanceDelta {
                            ord: log.ordinal(),
                            tx: Some(log.receipt.transaction.into()),
                            token: GRAIL_ADDRESS.to_vec(),
                            delta: ev
                                .grail_amount
                                .neg()
                                .to_signed_bytes_be(),
                            component_id: string_to_bytes_vec(&component_id),
                        },
                    ]);
                }
            } else if let Some(ev) = abi::xgrail::events::Deallocate::match_and_decode(log) {
                let component_id = address_to_hex_string(&XGRAIL_ADDRESS);

                if maybe_get_pool_tokens(&store, &component_id).is_some() {
                    deltas.extend_from_slice(&[
                        BalanceDelta {
                            ord: log.ordinal(),
                            tx: Some(log.receipt.transaction.into()),
                            token: XGRAIL_ADDRESS.to_vec(),
                            delta: ev.fee.to_signed_bytes_be(),
                            component_id: string_to_bytes_vec(&component_id),
                        },
                        BalanceDelta {
                            ord: log.ordinal(),
                            tx: Some(log.receipt.transaction.into()),
                            token: GRAIL_ADDRESS.to_vec(),
                            delta: ev.fee.to_signed_bytes_be(),
                            component_id: string_to_bytes_vec(&component_id),
                        },
                    ]);
                }
            } else if let Some(ev) = abi::pair::events::Mint::match_and_decode(log) {
                let component_id = address_to_hex_string(&log.address());

                if let Some((token_0, token_1)) = maybe_get_pool_tokens(&store, &component_id) {
                    let amount_0 = ev.amount0;
                    let amount_1 = ev.amount1;
                    deltas.extend_from_slice(&[
                        BalanceDelta {
                            ord: log.ordinal(),
                            tx: Some(log.receipt.transaction.into()),
                            token: token_0,
                            delta: amount_0.to_signed_bytes_be(),
                            component_id: string_to_bytes_vec(&component_id),
                        },
                        BalanceDelta {
                            ord: log.ordinal(),
                            tx: Some(log.receipt.transaction.into()),
                            token: token_1,
                            delta: amount_1.to_signed_bytes_be(),
                            component_id: string_to_bytes_vec(&component_id),
                        },
                    ]);
                }
            } else if let Some(ev) = abi::pair::events::Burn::match_and_decode(log) {
                let component_id = address_to_hex_string(&log.address());

                if let Some((token_0, token_1)) = maybe_get_pool_tokens(&store, &component_id) {
                    let amount_0 = ev.amount0;
                    let amount_1 = ev.amount1;
                    deltas.extend_from_slice(&[
                        BalanceDelta {
                            ord: log.ordinal(),
                            tx: Some(log.receipt.transaction.into()),
                            token: token_0,
                            delta: amount_0.neg().to_signed_bytes_be(),
                            component_id: string_to_bytes_vec(&component_id),
                        },
                        BalanceDelta {
                            ord: log.ordinal(),
                            tx: Some(log.receipt.transaction.into()),
                            token: token_1,
                            delta: amount_1.neg().to_signed_bytes_be(),
                            component_id: string_to_bytes_vec(&component_id),
                        },
                    ]);
                }
            } else if let Some(ev) = abi::pair::events::Swap::match_and_decode(log) {
                let component_id = address_to_hex_string(&log.address());

                if let Some((token_0, token_1)) = maybe_get_pool_tokens(&store, &component_id) {
                    let amount_0_in = ev.amount0_in;
                    let amount_1_in = ev.amount1_in;
                    let amount_0_out = ev.amount0_out;
                    let amount_1_out = ev.amount1_out;

                    deltas.extend_from_slice(&[
                        BalanceDelta {
                            ord: log.ordinal(),
                            tx: Some(log.receipt.transaction.into()),
                            token: token_0,
                            delta: (amount_0_in - amount_0_out).to_signed_bytes_be(),
                            component_id: string_to_bytes_vec(&component_id),
                        },
                        BalanceDelta {
                            ord: log.ordinal(),
                            tx: Some(log.receipt.transaction.into()),
                            token: token_1,
                            delta: (amount_1_in - amount_1_out).to_signed_bytes_be(),
                            component_id: string_to_bytes_vec(&component_id),
                        },
                    ]);
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
fn store_component(store: &StoreSetProto<ProtocolComponent>, component: &ProtocolComponent) {
    store.set(1, format!("pool:{}", component.id), component);
}
fn maybe_get_pool_tokens(
    store: &StoreGetProto<ProtocolComponent>,
    component_id: &str,
) -> Option<(Vec<u8>, Vec<u8>)> {
    store
        .get_last(format!("pool:{}", component_id))
        .map(|component| (component.tokens[0].to_vec(), component.tokens[1].to_vec()))
}
fn address_to_hex_string(address: &[u8]) -> String {
    format!("0x{}", hex::encode(address))
}

fn string_to_bytes_vec(s: &str) -> Vec<u8> {
    s.as_bytes().to_vec()
}
