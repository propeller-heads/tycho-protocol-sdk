use std::str::FromStr;
use anyhow::{Ok, Result};
use substreams_ethereum::pb::eth::v2::{Block, Log, TransactionTrace};
use tycho_substreams::prelude::*;
use ethabi::ethereum_types::Address;

use crate::abi::factory::events::PoolCreated;
use substreams_helper::event_handler::EventHandler;

#[substreams::handlers::map]
pub fn map_components(
    params: String,
    block: Block
) -> Result<BlockTransactionProtocolComponents> {
    let mut new_pools: Vec<TransactionProtocolComponents> = vec![];
    let factory_address = params.as_str();

    get_new_pools(factory_address, &block, &mut new_pools);

    Ok(BlockTransactionProtocolComponents {
        tx_components: new_pools,
    })
}

fn get_new_pools(
    factory_address: &str,
    block: &Block,
    new_pools: &mut Vec<TransactionProtocolComponents>,
)  {
    let mut on_pool_created = |event: PoolCreated, _tx: &TransactionTrace, _log: &Log| {
        let tycho_tx: Transaction = _tx.into();

        let new_pool_component =
            ProtocolComponent::new(&format!("0x{}", hex::encode(&event.pool_address)))
                .with_tokens(&[event.token_a.as_slice(), event.token_b.as_slice()])
                .with_contracts(&[&event.pool_address])
                .with_attributes(&[
                    ("pool_type", "WeightedPoolFactoryV1".as_bytes()),
                    ("fee_a_in", &event.fee_a_in.to_signed_bytes_be()),
                    ("fee_b_in", &event.fee_b_in.to_signed_bytes_be()),
                    ("tick_spacing", &event.tick_spacing.to_signed_bytes_be()),
                    ("kinds", &event.kinds.to_signed_bytes_be()),
                ])
                .as_swap_type("maverick_v2_pool", ImplementationType::Vm);
        
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