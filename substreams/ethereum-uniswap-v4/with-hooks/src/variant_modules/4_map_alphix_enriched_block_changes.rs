use substreams::store::{StoreGet, StoreGetInt64};
use substreams_helper::hex::Hexable;
use tycho_substreams::prelude::*;

/// Enriches protocol changes by adding `hook_identifier: "alphix_v1"` to pools
/// that use Alphix hooks.
///
/// This module reads the Alphix hooks store and tags matching pool creations
/// with the hook identifier, which the downstream Tycho DCI uses to apply
/// Alphix-specific external liquidity metadata collection.
#[substreams::handlers::map]
pub fn map_alphix_enriched_block_changes(
    block_changes: BlockChanges,
    alphix_hooks_store: StoreGetInt64,
) -> Result<BlockChanges, substreams::errors::Error> {
    let enriched = enrich_block_changes(block_changes, &alphix_hooks_store);
    Ok(enriched)
}

pub fn enrich_block_changes<T: StoreGet<i64>>(
    mut protocol_changes: BlockChanges,
    alphix_hooks_store: &T,
) -> BlockChanges {
    for tx_changes in &mut protocol_changes.changes {
        for component in &mut tx_changes.component_changes {
            if component.change == i32::from(ChangeType::Creation) {
                if let Some(hooks_attr) = component
                    .static_att
                    .iter()
                    .find(|attr| attr.name == "hooks")
                {
                    let hook_address = hooks_attr.value.to_hex();

                    if alphix_hooks_store
                        .get_last(&hook_address)
                        .is_some()
                    {
                        component
                            .static_att
                            .push(Attribute {
                                name: "hook_identifier".to_string(),
                                value: "alphix_v1".as_bytes().to_vec(),
                                change: ChangeType::Creation.into(),
                            });
                    }
                }
            }
        }
    }

    protocol_changes
}
