use std::collections::HashMap;
use crate::{modules::utils::Params};
use anyhow::{Ok, Result};
use ethabi::ethereum_types::Address;
use substreams::{prelude::BigInt};
use hex_literal::hex;
use serde::Deserialize;
use crate::pb::cowamm::{CowPool, CowPools};
use substreams_ethereum::pb::eth::v2::{Block};
use substreams_ethereum::Event;
use tycho_substreams::prelude::*;
use crate::abi::{b_cow_factory::events::CowammPoolCreated, b_cow_pool::functions::Bind};
use substreams_helper::{event_handler::EventHandler, hex::Hexable};

fn create_component(
    factory_address: &[u8],
    pool: CowPool,
) -> Option<ProtocolComponent> {

    substreams::log::info!(
        "CowammPoolCreated Event: pool address = {}",
        pool.address.to_hex()
    );

    Some(
        ProtocolComponent::new(&pool.address.to_hex())
        .with_tokens(&[pool.token_a.as_slice(), pool.token_a.as_slice(), pool.lp_token.as_slice()])  //remember the address of the contract is the address of the lp token  
        .with_contracts(&[
            factory_address,
            pool.address.as_slice()
        ])
        .with_attributes(&[  
            ("fee", BigInt::from(0).to_signed_bytes_be()), 
            ("denormalized_weight_a", BigInt::from(pool.weight_a).to_signed_bytes_be()), 
            ("denormalized_weight_b", BigInt::from(pool.weight_b).to_signed_bytes_be()),
        ])
        .as_swap_type("cowamm_pool", ImplementationType::Vm)
    )
    // substreams::log::info!("Constructed ProtocolComponent for pool {}", pool_created.b_co_w_pool.to_hex());
}

#[substreams::handlers::map]
pub fn map_components(params: String, block: Block, pools: CowPools) -> Result<BlockTransactionProtocolComponents> {
    substreams::log::info!("--- map_components called ---");   
    // substreams::log::info!("Received params: {}", params);
    // substreams::log::info!("Block number: {}", block.number);
    let params = Params::parse_from_query(&params)?;
    let factory_address = params.decode_addresses().unwrap();
    
    // Index CowPools by created_tx_hash for fast lookup
    let pool_by_tx_hash: HashMap<Vec<u8>, CowPool> = pools
        .pools
        .into_iter()
        .map(|pool| (pool.created_tx_hash.clone(), pool))
        .collect();

        Ok(BlockTransactionProtocolComponents {
            tx_components: 
              block
                .transactions()
                .filter_map(|tx| {
                    let pool = pool_by_tx_hash.get(&tx.hash)?;

                    let components = create_component(&factory_address, pool.clone())
                        .into_iter()
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
