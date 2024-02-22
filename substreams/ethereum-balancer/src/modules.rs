use std::collections::HashMap;

use anyhow::Result;
use substreams::hex;
use substreams::pb::substreams::StoreDeltas;
use substreams::store::{
    StoreAdd, StoreAddBigInt, StoreAddInt64, StoreGet, StoreGetInt64, StoreNew,
};

use substreams::key;
use substreams::scalar::BigInt;

use substreams_ethereum::pb::eth;

use itertools::Itertools;
use pb::tycho::evm::v1::{self as tycho};

use contract_changes::extract_contract_changes;

use crate::{abi, contract_changes, pb, pool_factories};

const VAULT_ADDRESS: &[u8] = &hex!("BA12222222228d8Ba445958a75a0704d566BF2C8");

/// This struct purely exists to spoof the `PartialEq` trait for `Transaction` so we can use it in
///  a later groupby operation.
#[derive(Debug)]
struct TransactionWrapper(tycho::Transaction);

impl PartialEq for TransactionWrapper {
    fn eq(&self, other: &Self) -> bool {
        self.0.hash == other.0.hash
    }
}

#[substreams::handlers::map]
pub fn map_pools_created(
    block: eth::v2::Block,
) -> Result<tycho::GroupedTransactionProtocolComponents> {
    // Gather contract changes by indexing `PoolCreated` events and analysing the `Create` call
    // We store these as a hashmap by tx hash since we need to agg by tx hash later
    Ok(tycho::GroupedTransactionProtocolComponents {
        tx_components: block
            .transactions()
            .filter_map(|tx| {
                let components = tx
                    .logs_with_calls()
                    .filter(|(_, call)| !call.call.state_reverted)
                    .filter_map(|(log, call)| {
                        Some(pool_factories::address_map(
                            call.call.address.as_slice(),
                            log,
                            call.call,
                        )?)
                    })
                    .collect::<Vec<_>>();

                if !components.is_empty() {
                    Some(tycho::TransactionProtocolComponents {
                        tx: Some(tycho::Transaction {
                            hash: tx.hash.clone(),
                            from: tx.from.clone(),
                            to: tx.to.clone(),
                            index: Into::<u64>::into(tx.index),
                        }),
                        components,
                    })
                } else {
                    None
                }
            })
            .collect::<Vec<_>>(),
    })
}

/// Simply stores the `ProtocolComponent`s with the pool id as the key
#[substreams::handlers::store]
pub fn store_pools_created(map: tycho::GroupedTransactionProtocolComponents, store: StoreAddInt64) {
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

/// Since the `PoolBalanceChanged` events administer only deltas, we need to leverage a map and a
///  store to be able to tally up final balances for tokens in a pool.
#[substreams::handlers::map]
pub fn map_balance_deltas(
    block: eth::v2::Block,
    store: StoreGetInt64,
) -> Result<tycho::BalanceDeltas, anyhow::Error> {
    Ok(tycho::BalanceDeltas {
        balance_deltas: block
            .events::<abi::vault::events::PoolBalanceChanged>(&[&VAULT_ADDRESS])
            .flat_map(|(event, log)| {
                event
                    .tokens
                    .iter()
                    .zip(event.deltas.iter())
                    .filter_map(|(token, delta)| {
                        let component_id: Vec<_> = event.pool_id.into();

                        if store
                            .get_last(format!("pool:{0}", hex::encode(&component_id)))
                            .is_none()
                        {
                            return None;
                        }

                        Some(tycho::BalanceDelta {
                            ord: log.log.ordinal,
                            tx: Some(tycho::Transaction {
                                hash: log.receipt.transaction.hash.clone(),
                                from: log.receipt.transaction.from.clone(),
                                to: log.receipt.transaction.to.clone(),
                                index: Into::<u64>::into(log.receipt.transaction.index),
                            }),
                            token: token.clone(),
                            delta: delta.to_signed_bytes_be(),
                            component_id: component_id.clone(),
                        })
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>(),
    })
}

/// It's significant to include both the `pool_id` and the `token_id` for each balance delta as the
///  store key to ensure that there's a unique balance being tallied for each.
#[substreams::handlers::store]
pub fn store_balance_changes(deltas: tycho::BalanceDeltas, store: StoreAddBigInt) {
    deltas.balance_deltas.iter().for_each(|delta| {
        store.add(
            delta.ord,
            format!(
                "pool:{0}:token:{1}",
                hex::encode(&delta.component_id),
                hex::encode(&delta.token)
            ),
            BigInt::from_signed_bytes_be(&delta.delta),
        );
    });
}

/// This is the main map that handles most of the indexing of this substream.
/// Every contract change is grouped by transaction index via the `transaction_contract_changes`
///  map. Each block of code will extend the `TransactionContractChanges` struct with the
///  cooresponding changes (balance, component, contract), inserting a new one if it doesn't exist.
///  At the very end, the map can easily be sorted by index to ensure the final `BlockContractChanges`
///  is ordered by transactions properly.
#[substreams::handlers::map]
pub fn map_changes(
    block: eth::v2::Block,
    grouped_components: tycho::GroupedTransactionProtocolComponents,
    deltas: tycho::BalanceDeltas,
    components_store: StoreGetInt64,
    balance_store: StoreDeltas, // Note, this map module is using the `deltas` mode for the store.
) -> Result<tycho::BlockContractChanges> {
    // We merge contract changes by transaction (identified by transaction index) making it easy to
    //  sort them at the very end.
    let mut transaction_contract_changes: HashMap<_, tycho::TransactionContractChanges> =
        HashMap::new();

    // `ProtocolComponents` are gathered from `map_pools_created` which just need a bit of work to
    //   convert into `TransactionContractChanges`
    grouped_components
        .tx_components
        .iter()
        .for_each(|tx_component| {
            let tx = tx_component.tx.as_ref().unwrap();

            transaction_contract_changes
                .entry(tx.index)
                .or_insert_with(|| tycho::TransactionContractChanges {
                    tx: Some(tx.clone()),
                    contract_changes: vec![],
                    component_changes: vec![],
                    balance_changes: vec![],
                })
                .component_changes
                .extend_from_slice(&tx_component.components);
        });

    // Balance changes are gathered by the `StoreDelta` based on `PoolBalanceChanged` creating
    //  `BalanceDeltas`. We essentially just process the changes that occured to the `store` this
    //  block. Then, these balance changes are merged onto the existing map of tx contract changes,
    //  inserting a new one if it doesn't exist.
    balance_store
        .deltas
        .into_iter()
        .zip(deltas.balance_deltas)
        .map(|(store_delta, balance_delta)| {
            let pool_id = key::segment_at(&store_delta.key, 1);
            let token_id = key::segment_at(&store_delta.key, 3);
            (
                balance_delta.tx.unwrap(),
                tycho::BalanceChange {
                    token: hex::decode(token_id).expect("Token ID not valid hex"),
                    balance: store_delta.new_value,
                    component_id: hex::decode(pool_id).expect("Token ID not valid hex"),
                },
            )
        })
        // We need to group the balance changes by tx hash for the `TransactionContractChanges` agg
        .group_by(|(tx, _)| TransactionWrapper(tx.clone()))
        .into_iter()
        .for_each(|(tx_wrapped, group)| {
            let tx = tx_wrapped.0;

            transaction_contract_changes
                .entry(tx.index)
                .or_insert_with(|| tycho::TransactionContractChanges {
                    tx: Some(tx.clone()),
                    contract_changes: vec![],
                    component_changes: vec![],
                    balance_changes: vec![],
                })
                .balance_changes
                .extend(group.map(|(_, change)| change));
        });

    // General helper for extracting contract changes. Uses block, our component store which holds
    //  all of our tracked deployed pool addresses, and the map of tx contract changes which we
    //  output into for final processing later.
    extract_contract_changes(&block, components_store, &mut transaction_contract_changes);

    // Process all `transaction_contract_changes` for final output in the `BlockContractChanges`,
    //  sorted by transaction index (the key).
    Ok(tycho::BlockContractChanges {
        block: Some(tycho::Block {
            number: block.number,
            hash: block.hash.clone(),
            parent_hash: block
                .header
                .as_ref()
                .expect("Block header not present")
                .parent_hash
                .clone(),
            ts: block.timestamp_seconds(),
        }),
        changes: transaction_contract_changes
            .drain()
            .sorted_unstable_by_key(|(index, _)| index.clone())
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
