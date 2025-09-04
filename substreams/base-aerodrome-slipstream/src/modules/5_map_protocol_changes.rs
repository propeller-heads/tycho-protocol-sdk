use std::{collections::HashMap, vec};
use itertools::Itertools;
use substreams::pb::substreams::StoreDeltas;
use substreams::store::{StoreGet, StoreGetProto};
use substreams_ethereum::pb::eth::v2::{self as eth};

use substreams_helper::hex::Hexable;

use crate::{
    events::{get_log_changed_attributes},
    pb::aerodrome::slipstream::Pool,
};

use tycho_substreams::{
    entrypoint::create_entrypoint, models::entry_point_params::TraceData, prelude::*,
};
use tycho_substreams::balances::aggregate_balances_changes;
use tycho_substreams::block_storage::get_block_storage_changes;
use crate::abi::clfactory::functions::GetSwapFee;

#[substreams::handlers::map]
pub fn map_protocol_changes(
    params: String,
    block: eth::Block,
    protocol_components: BlockTransactionProtocolComponents,
    balance_deltas: BlockBalanceDeltas,
    pools_store: StoreGetProto<Pool>,
    balance_store: StoreDeltas,
) -> Result<BlockChanges, substreams::errors::Error> {
    let factory_address = params.as_str();
    let mut transaction_changes: HashMap<_, TransactionChangesBuilder> = HashMap::new();

    protocol_components
        .tx_components
        .iter()
        .for_each(|tx_component| {
            let tx = tx_component.tx.as_ref().unwrap();
            let builder = transaction_changes
                .entry(tx.index)
                .or_insert_with(|| TransactionChangesBuilder::new(tx));

            tx_component
                .components
                .iter()
                .for_each(|c| {
                    builder.add_protocol_component(c);
                    let get_swap_fee_fn = GetSwapFee {
                        pool: c.id.clone().into_bytes(),
                    };
                    // iterate all touched components and add entrypoints for them
                    let trace_data = TraceData::Rpc(RpcTraceData {
                        caller: None,
                        calldata: get_swap_fee_fn.encode(), // factory.getSwapFee(address(this))
                    });
                    let (entrypoint, entrypoint_params) = create_entrypoint(
                        factory_address.as_bytes().to_vec(),
                        "getSwapFee(address)".to_string(),
                        c.id.clone(),
                        trace_data,
                    );
                    builder.add_entrypoint(&entrypoint);
                    builder.add_entrypoint_params(&entrypoint_params);

                });
        });

    aggregate_balances_changes(balance_store, balance_deltas)
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

    for trx in block.transactions() {
        let tx = Transaction {
            to: trx.to.clone(),
            from: trx.from.clone(),
            hash: trx.hash.clone(),
            index: trx.index.into(),
        };
        let mut merged_changes: Vec<EntityChanges> = vec![];
        for (log, call_view) in trx.logs_with_calls() {
            // Skip if the log is not from a known aerodrome slipstream pool.
            if let Some(pool) =
                pools_store.get_last(format!("{}:{}", "Pool", &log.address.to_hex()))
            {
                let changed_attributes = get_log_changed_attributes(
                    log,
                    &call_view.call.storage_changes,
                    pool.address
                        .clone()
                        .as_slice()
                        .try_into()
                        .expect("Pool address is not 20 bytes long"),
                );

                // Create entity changes
                let entity_changes: Vec<EntityChanges> = vec![EntityChanges {
                    component_id: pool.address.clone().to_hex(),
                    attributes: changed_attributes,
                }];

                merged_changes = merge_entity_changes(&merged_changes, &entity_changes);
            } else {
                continue;
            }
        }
        let builder = transaction_changes
            .entry(trx.index as u64)
            .or_insert_with(|| TransactionChangesBuilder::new(&tx));

        merged_changes.iter().for_each(|c| {
            builder.add_entity_change(c);
        })
    }

    let tycho_block: Block = (&block).into();
    let block_storage_changes = get_block_storage_changes(&block);

    let block_entity_changes =
        BlockChanges {
            block: Some(tycho_block),
            changes: transaction_changes
                .drain()
                .sorted_unstable_by_key(|(index, _)| *index)
                .filter_map(|(_, builder)| builder.build())
                .collect::<Vec<_>>(),
            storage_changes: block_storage_changes
        };

    Ok(block_entity_changes)
}

/// Merges new entity changes into an existing collection of entity changes and returns the merged
/// result. For each entity change, if an entity change with the same component_id exists, its
/// attributes are merged. If an attribute with the same name exists, the new attribute replaces the
/// old one.
///
/// Parameters:
/// - `existing_changes`: A reference to a vector of existing entity changes.
/// - `new_changes`: A reference to a vector of new entity changes to be merged.
///
/// Returns:
/// A new `Vec<EntityChanges>` containing the merged entity changes.
fn merge_entity_changes(
    existing_changes: &[EntityChanges],
    new_changes: &Vec<EntityChanges>,
) -> Vec<EntityChanges> {
    let mut changes_map = existing_changes
        .iter()
        .cloned()
        .map(|change| (change.component_id.clone(), change))
        .collect::<HashMap<_, _>>();

    for change in new_changes {
        match changes_map.get_mut(&change.component_id) {
            Some(existing_change) => {
                let mut attributes_map = existing_change
                    .attributes
                    .iter()
                    .cloned()
                    .map(|attr| (attr.name.clone(), attr))
                    .collect::<HashMap<_, _>>();

                for attr in &change.attributes {
                    attributes_map.insert(attr.name.clone(), attr.clone());
                }

                existing_change.attributes = attributes_map.into_values().collect();
            }
            None => {
                changes_map.insert(change.component_id.clone(), change.clone());
            }
        }
    }

    changes_map.into_values().collect()
}
