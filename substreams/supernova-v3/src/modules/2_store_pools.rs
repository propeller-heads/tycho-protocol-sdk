use substreams::store::{StoreNew, StoreSetIfNotExists, StoreSetIfNotExistsProto};

use crate::store_key::StoreKey;
use tycho_substreams::prelude::*;

#[substreams::handlers::store]
pub fn store_pools(
    pools_created: BlockChanges,
    store: StoreSetIfNotExistsProto<ProtocolComponent>,
) {
    // Store pools. Required so the next steps can match any event to a known pool by their address.
    // We index the ProtocolComponent under every contract address it owns (pool + plugin),
    // so `pool_check` in map_pool_events can recognize storage changes coming from
    // either the pool itself or its plugin.

    for change in pools_created.changes {
        for new_protocol_component in change.component_changes {
            // Primary index: by component id (pool address)
            store.set_if_not_exists(
                0,
                StoreKey::Pool.get_unique_pool_key(&new_protocol_component.id),
                &new_protocol_component,
            );

            // Secondary index: by every other tracked contract address (e.g. plugin),
            // so `extract_contract_changes_builder` will pick up their storage too.
            for contract_addr in &new_protocol_component.contracts {
                let addr_hex = format!("0x{}", hex::encode(contract_addr)).to_lowercase();
                if addr_hex == new_protocol_component.id.to_lowercase() {
                    continue;
                }
                store.set_if_not_exists(
                    0,
                    StoreKey::Pool.get_unique_pool_key(&addr_hex),
                    &new_protocol_component,
                );
            }
        }
    }
}
