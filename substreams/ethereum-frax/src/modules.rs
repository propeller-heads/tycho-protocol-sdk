use crate::{abi, pool_factories};
use anyhow::Result;
use itertools::Itertools;
use std::collections::HashMap;
use substreams::{
    pb::substreams::StoreDeltas,
    store::{
        StoreAddBigInt, StoreGet, StoreGetInt64, StoreGetProto, StoreNew, StoreSet, StoreSetProto,
    },
};
use substreams_ethereum::{pb::eth, Event};
use tycho_substreams::{
    balances::aggregate_balances_changes, contract::extract_contract_changes, prelude::*,
};

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
                    .filter(|(_, call)| !call.call.state_reverted)
                    .filter_map(|(log, call)| {
                        pool_factories::address_map(
                            call.call.address.as_slice(),
                            log,
                            call.call,
                            &(tx.into()),
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

/// Stores the `ProtocolComponent`s with the pool id as the key, together with the token pair as
/// events do not contain the pair info
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

/// we need to leverage a
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

            if let Some(event) = abi::pair_contract::events::Mint::match_and_decode(log) {
                // Mint event: (reserve0, reserve1) += (amount0, amount1)
                let component_id = address_to_hex(log.address());

                if let Some((token0, token1)) = maybe_get_pool_tokens(&store, &component_id) {
                    deltas.extend_from_slice(&[
                        BalanceDelta {
                            ord: log.ordinal(),
                            tx: Some(log.receipt.transaction.into()),
                            token: token0,
                            delta: event.amount0.to_signed_bytes_be(),
                            component_id: string_to_bytes(&component_id),
                        },
                        BalanceDelta {
                            ord: log.ordinal(),
                            tx: Some(log.receipt.transaction.into()),
                            token: token1,
                            delta: event.amount1.to_signed_bytes_be(),
                            component_id: string_to_bytes(&component_id),
                        },
                    ]);
                }
            } else if let Some(event) = abi::pair_contract::events::Burn::match_and_decode(log) {
                // Burn event: (reserve0, reserve1) -= (amount0, amount1)
                let component_id = address_to_hex(log.address());

                if let Some((token0, token1)) = maybe_get_pool_tokens(&store, &component_id) {
                    deltas.extend_from_slice(&[
                        BalanceDelta {
                            ord: log.ordinal(),
                            tx: Some(log.receipt.transaction.into()),
                            token: token0,
                            delta: event.amount0.to_signed_bytes_be(),
                            component_id: string_to_bytes(&component_id),
                        },
                        BalanceDelta {
                            ord: log.ordinal(),
                            tx: Some(log.receipt.transaction.into()),
                            token: token1,
                            delta: event.amount1.to_signed_bytes_be(),
                            component_id: string_to_bytes(&component_id),
                        },
                    ]);
                }
            } else if let Some(event) = abi::pair_contract::events::Swap::match_and_decode(log) {
                // Swap event: (reserve0, reserve1) += (amount0In - amount0Out, amount1In -
                // amount1Out)
                let component_id = address_to_hex(log.address());

                if let Some((token0, token1)) = maybe_get_pool_tokens(&store, &component_id) {
                    deltas.extend_from_slice(&[
                        BalanceDelta {
                            ord: log.ordinal(),
                            tx: Some(log.receipt.transaction.into()),
                            token: token0.clone(),
                            delta: (event.amount0_in - event.amount0_out).to_signed_bytes_be(),
                            component_id: string_to_bytes(&component_id),
                        },
                        BalanceDelta {
                            ord: log.ordinal(),
                            tx: Some(log.receipt.transaction.into()),
                            token: token1.clone(),
                            delta: (event.amount1_in - event.amount1_out).to_signed_bytes_be(),
                            component_id: string_to_bytes(&component_id),
                        },
                    ]);
                }
            } else if let Some(event) =
                abi::pair_contract::events::VirtualOrderExecution::match_and_decode(log)
            {
                // VirtualOrderExecution event: (reserve0, reserve1) += (amount0Sold, amount1Sold) -
                // (amount0Bought, amount1Bought)
                let component_id = address_to_hex(log.address());

                if let Some((token0, token1)) = maybe_get_pool_tokens(&store, &component_id) {
                    deltas.extend_from_slice(&[
                        BalanceDelta {
                            ord: log.ordinal(),
                            tx: Some(log.receipt.transaction.into()),
                            token: token0.clone(),
                            delta: (event.token0_sold - event.token0_bought).to_signed_bytes_be(),
                            component_id: string_to_bytes(&component_id),
                        },
                        BalanceDelta {
                            ord: log.ordinal(),
                            tx: Some(log.receipt.transaction.into()),
                            token: token1.clone(),
                            delta: (event.token1_sold - event.token1_bought).to_signed_bytes_be(),
                            component_id: string_to_bytes(&component_id),
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

fn maybe_get_pool_tokens(
    store: &StoreGetProto<ProtocolComponent>,
    component_id: &str,
) -> Option<(Vec<u8>, Vec<u8>)> {
    store
        .get_last(format!("pool:{}", component_id))
        .map(|component| (component.tokens[0].to_vec(), component.tokens[1].to_vec()))
}

fn address_to_hex(address: &[u8]) -> String {
    format!("0x{}", hex::encode(address))
}

fn string_to_bytes(string: &str) -> Vec<u8> {
    string.as_bytes().to_vec()
}

fn store_component(store: &StoreSetProto<ProtocolComponent>, component: &ProtocolComponent) {
    store.set(1, format!("pool:{}", component.id), component);
}
