use crate::abi::pool_contract::events::{PoolAdded, TokensTraded};
use anyhow::Result;
use itertools::Itertools;
use std::collections::HashMap;
use substreams::{
    hex,
    pb::substreams::StoreDeltas,
    store::{
        StoreAddBigInt, StoreGet, StoreGetInt64, StoreGetProto, StoreNew, StoreSet, StoreSetString,
    },
};
use substreams_ethereum::{
    pb::{
        eth,
        eth::v2::{Log, TransactionTrace},
    },
    Event,
};
use tycho_substreams::{
    balances::aggregate_balances_changes, contract::extract_contract_changes_builder, prelude::*,
};

pub const FACTORY_ADDRESS: [u8; 20] = hex!("eEF417e1D5CC832e619ae18D2F140De2999dD4fB");
pub const BNT_ADDRESS: [u8; 20] = hex!("1F573D6Fb3F13d689FF844B4cE37794d79a7FF1C");

#[substreams::handlers::map]
pub fn map_components(
    block: eth::v2::Block,
) -> Result<BlockTransactionProtocolComponents, anyhow::Error> {
    Ok(BlockTransactionProtocolComponents {
        tx_components: block
            .transactions()
            .filter_map(|tx| {
                let components = tx
                    .logs_with_calls()
                    .filter(|(_, call)| !call.call.state_reverted)
                    .filter_map(|(log, _)| address_map(&log, &tx))
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

/// Simply stores the `ProtocolComponent`s with the pool address as the key and the pool id as value
#[substreams::handlers::store]
pub fn store_components(map: BlockTransactionProtocolComponents, store: StoreSetString) {
    map.tx_components
        .into_iter()
        .for_each(|tx_pc| {
            tx_pc
                .components
                .into_iter()
                .for_each(|pc| store.set(0, format!("pool:{0}", &pc.id[..42]), &pc.id))
        });
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
        .filter(|log| log.address() == &FACTORY_ADDRESS)
        .flat_map(|log| {
            let mut deltas = Vec::new();

            // Match the `TokensTraded` event from the log
            if let Some(event) = TokensTraded::match_and_decode(log) {
                let source_component_id = address_to_hex(&event.source_token);
                let target_component_id = address_to_hex(&event.target_token);

                // Retrieve source and target components from the store
                if let (Some(_source_component), Some(_target_component)) = (
                    store.get_last(format!("pool:{}", source_component_id)),
                    store.get_last(format!("pool:{}", target_component_id)),
                ) {
                    // Adding balance delta for source token
                    deltas.push(BalanceDelta {
                        ord: log.ordinal(),
                        tx: Some(log.receipt.transaction.into()),
                        token: event.source_token.clone(),
                        delta: event.source_amount.to_signed_bytes_be(),
                        component_id: source_component_id.clone().into_bytes(),
                    });

                    // Adding balance delta for target token
                    deltas.push(BalanceDelta {
                        ord: log.ordinal(),
                        tx: Some(log.receipt.transaction.into()),
                        token: event.target_token.clone(),
                        delta: event
                            .target_amount
                            .neg()
                            .to_signed_bytes_be(),
                        component_id: target_component_id.clone().into_bytes(),
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

/// This is the main map that handles most of the indexing of this substream.
/// Every contract change is grouped by transaction index via the `transaction_contract_changes`
///  map. Each block of code will extend the `TransactionContractChanges` struct with the
///  corresponding changes (balance, component, contract), inserting a new one if it doesn't exist.
///  At the very end, the map can easily be sorted by index to ensure the final
/// `BlockContractChanges`  is ordered by transactions properly.
#[substreams::handlers::map]
pub fn map_protocol_changes(
    block: eth::v2::Block,
    grouped_components: BlockTransactionProtocolComponents,
    deltas: BlockBalanceDeltas,
    components_store: StoreGetInt64,
    balance_store: StoreDeltas, // Note, this map module is using the `deltas` mode for the store.
) -> Result<BlockChanges, anyhow::Error> {
    // We merge contract changes by transaction (identified by transaction index) making it easy to
    //  sort them at the very end.
    let mut transaction_changes: HashMap<_, TransactionChangesBuilder> = HashMap::new();

    // `ProtocolComponents` are gathered from `map_pools_created` which just need a bit of work to
    //   convert into `TransactionChanges`
    grouped_components
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

    // Balance changes are gathered by the `StoreDelta` based on `PoolBalanceChanged` creating
    //  `BlockBalanceDeltas`. We essentially just process the changes that occurred to the `store`
    // this  block. Then, these balance changes are merged onto the existing map of tx contract
    // changes,  inserting a new one if it doesn't exist.
    aggregate_balances_changes(balance_store, deltas)
        .into_iter()
        .for_each(|(_, (tx, balances))| {
            let builder = transaction_changes
                .entry(tx.index)
                .or_insert_with(|| TransactionChangesBuilder::new(&tx));
            balances
                .values()
                .for_each(|token_bc_map| {
                    token_bc_map
                        .values()
                        .for_each(|bc| builder.add_balance_change(bc))
                });
        });

    // Extract and insert any storage changes that happened for any of the components.
    extract_contract_changes_builder(
        &block,
        |addr| {
            components_store
                .get_last(format!("pool:0x{0}", hex::encode(addr)))
                .is_some()
        },
        &mut transaction_changes,
    );

    // Process all `transaction_contract_changes` for final output in the `BlockContractChanges`,
    //  sorted by transaction index (the key).
    let block_changes = BlockChanges {
        block: Some((&block).into()),
        changes: transaction_changes
            .drain()
            .sorted_unstable_by_key(|(index, _)| *index)
            .filter_map(|(_, builder)| builder.build())
            .collect::<Vec<_>>(),
    };

    for change in &block_changes.changes {
        substreams::log::info!("🚨 Balance changes {:?}", change.balance_changes);
        substreams::log::info!("🚨 Component changes {:?}", change.component_changes);
    }
    Ok(block_changes)
}

fn address_to_hex(address: &[u8]) -> String {
    format!("0x{}", hex::encode(address))
}

fn address_map(log: &Log, tx: &TransactionTrace) -> Option<ProtocolComponent> {
    let address = log.address.to_owned();
    if address == FACTORY_ADDRESS {
        PoolAdded::match_and_decode(log).map(|pool_added| {
            substreams::log::info!("🚨 Pool added 0x{:?}", hex::encode(&pool_added.pool));

            ProtocolComponent::at_contract(&pool_added.pool, &(tx.into()))
                .with_tokens(&[pool_added.pool, BNT_ADDRESS.to_vec()])
                .as_swap_type("bancor_v3_pool", ImplementationType::Vm)
        })
    } else {
        None
    }
}
