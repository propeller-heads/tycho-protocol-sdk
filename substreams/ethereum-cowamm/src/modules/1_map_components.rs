
use anyhow::{Ok, Result};
use ethabi::ethereum_types::Address;
use serde::Deserialize;
use substreams_ethereum::pb::eth::v2::{Block, Log, TransactionTrace};
use tycho_substreams::prelude::*;

use crate::{abi::b_cow_factory::events::CowammPoolCreated, modules::utils::Params};
use substreams_helper::{event_handler::EventHandler, hex::Hexable};

#[derive(Debug, Deserialize)]
struct Params {
    factory_address: String,
}

//get the first two bind function call traces so we can get the two tokens that were bound 
// to the pool
fn get_tokens_for_pool(
    tx: &TransactionTrace,
    pool_address: &[u8],
) -> Vec<abi::b_cow_pool::functions::Bind> {
     tx.logs_with_calls()
        .filter(|(_, call)| call.address == pool_address)
        .filter_map(|(_, call)| abi::b_cow_pool::functions::Bind::match_and_decode(call))
        .take(2)
        .unwrap()
        .cloned()
        .collect<Vec<_>>()
}

#[substreams::handlers::map]
pub fn map_components(params: String, block: Block) -> Result<BlockTransactionProtocolComponents> {
    let mut new_pools: Vec<TransactionProtocolComponents> = vec![];
  
    let params: Params = serde_qs::from_str(params.as_str()).expect("Unable to deserialize params");

    get_pools(params, &block, &mut new_pools);
  
    Ok(BlockTransactionProtocolComponents { tx_components: new_pools })
}

//By listening for the CowammPoolCreated event on the factory contract, we can get all the pools created
//for CowAmms , the tokens are not created as an attribute of the pool, rather a bind() method has to be called
// to bind specific weights of a particular token 
fn get_pools(
    params: Params,
    block: &Block,
    new_pools: &mut Vec<TransactionProtocolComponents>,
) {
    let factory_address = params.factory_address;
    
    let mut on_pool_created = |event: CowammPoolCreated , _tx: &TransactionTrace, _log: &Log| {
        let tokens = get_tokens_for_pool();

        let tycho_tx: Transaction = _tx.into();
        let contracts = vec![
            factory_address.as_slice(),
            event.b_co_w_pool.as_slice(), 
        ]; 
        let new_pool_component = ProtocolComponent::new(&event.b_co_w_pool.to_hex())
            .with_tokens(&[tokens[0].token.as_slice(), tokens[1].token.as_slice()])
            .with_contracts(&contracts)
            .with_attributes(&[ 
                ("swap_fee", 99.9999.to_signed_bytes_be()),
                ("denormalized_weight_a", &token_a_binding.denorm.to_signed_bytes_be()), //todo
                ("denormalized_weight_b", &token_b_binding.denorm.to_signed_bytes_be()),
            ])
            .as_swap_type("cowamm_pool", ImplementationType::Vm);

        new_pools.push(TransactionProtocolComponents {
            tx: Some(tycho_tx.clone()),
            components: vec![new_pool_component],
        });
    };

    let mut eh = EventHandler::new(block);

    eh.filter_by_address(vec![Address::from_slice(&factory_address)]); 

    eh.on::<CowammPoolCreated, _>(&mut on_pool_created);
    eh.handle_events();
}
