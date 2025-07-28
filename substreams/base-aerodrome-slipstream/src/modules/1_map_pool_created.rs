use std::str::FromStr;

use ethabi::ethereum_types::Address;
use substreams_ethereum::pb::eth::v2::{self as eth};

use substreams_helper::{event_handler::EventHandler, hex::Hexable};

use crate::abi::clfactory::events::PoolCreated;
use tycho_substreams::prelude::*;

#[substreams::handlers::map]
pub fn map_pools_created(
    params: String,
    block: eth::Block,
) -> Result<BlockTransactionProtocolComponents, substreams::errors::Error> {
    let mut new_pools: Vec<TransactionProtocolComponents> = vec![];
    let factory_address = params.as_str();

    get_new_pools(&block, &mut new_pools, factory_address);

    Ok(BlockTransactionProtocolComponents { tx_components: new_pools })
}

// Extract new pools from PoolCreated events
fn get_new_pools(
    block: &eth::Block,
    new_pools: &mut Vec<TransactionProtocolComponents>,
    factory_address: &str,
) {
    // Extract new pools from PoolCreated events
    let mut on_pool_created = |event: PoolCreated, _tx: &eth::TransactionTrace, _log: &eth::Log| {
        let tycho_tx: Transaction = _tx.into();
        let new_pool_component = ProtocolComponent::new(&event.pool.to_hex())
            .with_tokens(&[event.token0.as_slice(), event.token1.as_slice()])
            .with_attributes(&[("pool_address", event.pool.to_hex())])
            .as_swap_type("aerodrome_slipstream_pool", ImplementationType::Custom);
        new_pools.push(TransactionProtocolComponents {
            tx: Some(tycho_tx.clone()),
            components: vec![new_pool_component],
        });
    };

    let mut eh = EventHandler::new(block);

    eh.filter_by_address(vec![Address::from_str(factory_address).unwrap()]);

    eh.on::<PoolCreated, _>(&mut on_pool_created);
    eh.handle_events();
}
