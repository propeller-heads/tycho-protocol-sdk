use substreams::store::{StoreAppend, Appender};
use substreams_helper::hex::Hexable;
use tycho_substreams::prelude::*;

#[substreams::handlers::store]
pub fn store_pools_per_hook(
    _params: String,
    pools_created: BlockEntityChanges,
    output: StoreAppend<String>,
) {
    let pool_hook_mappings = _track_uniswap_pools_by_hook(pools_created);
    
    for (key, pool_id) in pool_hook_mappings {
        output.append(0, &key, pool_id);
    }
}

// Extracted core logic for easier testing
pub fn _track_uniswap_pools_by_hook(pools_created: BlockEntityChanges) -> Vec<(String, String)> {
    let mut pool_hook_mappings = Vec::new();
    
    for tx_change in pools_created.changes {
        for component_change in tx_change.component_changes {
            // Extract the hook address from the static attributes
            if let Some(hooks_attr) = component_change.static_att.iter().find(|attr| attr.name == "hooks") {
                let hook_address = hooks_attr.value.to_hex();
                let pool_id = component_change.id.clone();
                
                // Store the pool ID under the hook address key
                // We use append mode to maintain a list of pools per hook
                let key = format!("hook:{}", hook_address);
                
                pool_hook_mappings.push((key, pool_id));
            }
        }
    }
    
    pool_hook_mappings
}