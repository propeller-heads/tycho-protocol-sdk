use std::collections::HashMap;
use tycho_substreams::prelude::*;

#[substreams::handlers::map]
pub fn map_combined_enriched_block_changes(
    euler_enriched_changes: BlockChanges,
    angstrom_enriched_changes: BlockChanges,
) -> Result<BlockChanges, substreams::errors::Error> {
    let combined_changes = merge_block_changes(euler_enriched_changes, angstrom_enriched_changes);
    Ok(combined_changes)
}

/// Merges two BlockChanges into a single BlockChanges by combining their transaction changes
fn merge_block_changes(
    euler_changes: BlockChanges,
    angstrom_changes: BlockChanges,
) -> BlockChanges {
    // Use the block from the first input (they should be the same)
    let block = euler_changes
        .block
        .clone()
        .unwrap_or_else(|| {
            angstrom_changes
                .block
                .clone()
                .expect("At least one BlockChanges must have a block")
        });

    // Collect all transaction changes by transaction index
    let mut transaction_changes: HashMap<u64, TransactionChangesBuilder> = HashMap::new();

    // Process Euler enriched changes
    for tx_change in euler_changes.changes {
        let tx_index = tx_change
            .tx
            .as_ref()
            .map(|t| t.index)
            .unwrap_or(0u64);
        let tx = tx_change
            .tx
            .as_ref()
            .expect("Transaction must be present");

        let builder = transaction_changes
            .entry(tx_index)
            .or_insert_with(|| TransactionChangesBuilder::new(tx));

        for component_change in tx_change.component_changes {
            builder.add_protocol_component(&component_change);
        }
        for entity_change in tx_change.entity_changes {
            builder.add_entity_change(&entity_change);
        }
        for balance_change in tx_change.balance_changes {
            builder.add_balance_change(&balance_change);
        }
    }

    // Process Angstrom enriched changes
    for tx_change in angstrom_changes.changes {
        let tx_index = tx_change
            .tx
            .as_ref()
            .map(|t| t.index)
            .unwrap_or(0u64);
        let tx = tx_change
            .tx
            .as_ref()
            .expect("Transaction must be present");

        let builder = transaction_changes
            .entry(tx_index)
            .or_insert_with(|| TransactionChangesBuilder::new(tx));

        // Add all changes from this transaction
        for component_change in tx_change.component_changes {
            builder.add_protocol_component(&component_change);
        }
        for entity_change in tx_change.entity_changes {
            builder.add_entity_change(&entity_change);
        }
        for balance_change in tx_change.balance_changes {
            builder.add_balance_change(&balance_change);
        }
    }

    // Build the final sorted transaction changes
    let mut sorted_changes: Vec<_> = transaction_changes
        .into_iter()
        .collect();
    sorted_changes.sort_by_key(|(index, _)| *index);

    let final_changes: Vec<TransactionChanges> = sorted_changes
        .into_iter()
        .filter_map(|(_, builder)| builder.build())
        .collect();

    // Merge storage changes from both inputs
    let mut all_storage_changes = euler_changes.storage_changes;
    all_storage_changes.extend(angstrom_changes.storage_changes);

    BlockChanges {
        block: Some(block),
        changes: final_changes,
        storage_changes: all_storage_changes,
    }
}
