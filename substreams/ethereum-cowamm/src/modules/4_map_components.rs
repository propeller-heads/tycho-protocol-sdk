use crate::{modules::utils::Params};
use anyhow::{Ok, Result};
use ethabi::ethereum_types::Address;
use substreams::{prelude::BigInt};
use hex_literal::hex;
use serde::Deserialize;
use crate::pb::cowamm::{CowPool, CowPools};
use substreams_ethereum::pb::eth::v2::{Call,Block, Log, TransactionTrace};
use substreams_ethereum::Event;
use tycho_substreams::prelude::*;
use crate::abi::{b_cow_factory::events::CowammPoolCreated, b_cow_pool::functions::Bind};
use substreams_helper::{event_handler::EventHandler, hex::Hexable};

fn getTransactionTrace(block: Block, pool:CowPool) {
   return block
            .transactions()
            .find(|txn| {                   
                pool.created_tx_hash == txn.hash;
            })
}

fn create_component(
    factory_address: &[u8],
    pool: CowPool,
) -> Option<ProtocolComponent> {

    substreams::log::info!(
        "CowammPoolCreated Event: pool address = {}",
        pool.address.to_hex()
    );

    Some(
        ProtocolComponent::new(&pool.address.to_hex()) // &(tx.into()
        .with_tokens(&[pool.token_a.as_slice(), pool.token_a.as_slice(), pool.lp_token.as_slice()])  //remember the address of the contract is the address of the lp token  
        .with_contracts(&[
            factory_address,
            pool.address.as_slice()
        ])
        .with_attributes(&[  
            ("fee", BigInt::from(0).to_signed_bytes_be()), // not 99.999% ? cuz thats what we are calculating it as 
            ("denormalized_weight_a", BigInt::from(pool.weight_a).to_signed_bytes_be()), 
            ("denormalized_weight_b", BigInt::from(pool.weight_b).to_signed_bytes_be()),
        ])
        .as_swap_type("cowamm_pool", ImplementationType::Vm)
    )
    // substreams::log::info!("Constructed ProtocolComponent for pool {}", pool_created.b_co_w_pool.to_hex());
}


pub fn map_components(params: String, pools: CowPools) -> Result<BlockTransactionProtocolComponents> {
    substreams::log::info!("--- map_components called ---");   
    // substreams::log::info!("Received params: {}", params);
    // substreams::log::info!("Block number: {}", block.number);
    let params = Params::parse_from_query(&params)?;
    let factory_address = params.decode_addresses().unwrap();

        Ok(BlockTransactionProtocolComponents {
            tx_components: 
            pools
                .pools
                .iter()
                .filter_map(|pool| {
                   
                    let components = create_component(&factory_address, pool.clone())
                        .into_iter()
                        .collect::<Vec<_>>();
                    if !components.is_empty() { // pool.tx is Option<cowamm::Transaction>
                        Some(TransactionProtocolComponents { tx: pool.tx.into(), components }) 
                    } else {
                        None
                    }            
                })
                .collect::<Vec<_>>(),
    })
}
