use std::str::FromStr;

use ethabi::ethereum_types::Address;
use substreams::{store::{StoreSetIfNotExistsString, StoreSetIfNotExists, StoreNew}};
use substreams_ethereum::pb::eth::v2::{self as eth};
use substreams_helper::event_handler::EventHandler;
use substreams_helper::hex::Hexable;
use ethereum_uniswap_v4_shared::abi::euler_swap_factory::events::PoolDeployed;

#[substreams::handlers::store]
pub fn store_euler_hooks(
    params: String,
    block: eth::Block,
    output: StoreSetIfNotExistsString,
) {
    let euler_factory_address = params.as_str();
    let euler_hooks = _track_euler_hooks(&block, euler_factory_address);
    
    for (hook_key, euler_account_value) in euler_hooks {
        output.set_if_not_exists(0, &hook_key, &euler_account_value);
    }
}

pub fn _track_euler_hooks(
    block: &eth::Block,
    euler_factory_address: &str,
) -> Vec<(String, String)> {
    let mut euler_hooks = Vec::new();

    {
        let mut on_pool_deployed = |event: PoolDeployed, _tx: &eth::TransactionTrace, _log: &eth::Log| {
            // Store the relationship between the deployed hook (pool) address and the Euler pool info
            // Key: hook_address, Value: euler_account (could be expanded to include more data)
            let hook_key = event.pool.to_hex();
            let euler_account_value = event.euler_account.to_hex();
            
            euler_hooks.push((hook_key, euler_account_value));
        };

        let mut eh = EventHandler::new(block);
        eh.filter_by_address(vec![Address::from_str(euler_factory_address).unwrap()]);
        eh.on::<PoolDeployed, _>(&mut on_pool_deployed);
        eh.handle_events();
    }
    
    euler_hooks
}