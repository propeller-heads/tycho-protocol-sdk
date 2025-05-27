use std::collections::HashMap;
use std::str::FromStr;
use crate::{modules::utils::Params};
use crate::pb::cowamm::{CowPool, CowPools, CowpoolBind, CowPoolBinds, CowPoolCreation, CowPoolCreations};
use anyhow::{Ok, Result};
use ethabi::ethereum_types::Address;
use substreams::{prelude::BigInt};
use hex_literal::hex;
use serde::Deserialize;
use substreams_ethereum::pb::eth::v2::{Call,Block, Log, TransactionTrace};
use substreams_ethereum::Event;
use tycho_substreams::prelude::*;
use crate::abi::{b_cow_factory::events::CowammPoolCreated, b_cow_pool::functions::Bind};
use substreams_helper::{event_handler::EventHandler, hex::Hexable};


fn build_component(
    pool:CowPool
) -> Option<ProtocolComponent> {
            
    let params = Params::parse_from_query(&params)?;
    let factory_address = params.decode_addresses().unwrap();

    substreams::log::info!(
        "CowammPoolCreated Event: pool address = {}",
        pool.address.to_hex()
    );

    Some(
        ProtocolComponent::new(&pool.to_hex(), &(tx.into()))
        .with_tokens(&[pool.token_a.as_slice(), pool.token_a.as_slice(), pool.lp_token])  //remember the address of the contract is the address of the lp token  
        .with_contracts(&[
            factory_address,
            pool.address.as_slice()
        ])
        .with_attributes(&[  
            ("fee", BigInt::from(0).to_signed_bytes_be()), // not 99.999% ? cuz thats what we are calculating it as 
            ("denormalized_weight_a", pool.weight_a.to_signed_bytes_be()), 
            ("denormalized_weight_b", pool.weight_b.to_signed_bytes_be()),
        ])
        .as_swap_type("cowamm_pool", ImplementationType::Vm)
    )
    // substreams::log::info!("Constructed ProtocolComponent for pool {}", pool_created.b_co_w_pool.to_hex());
}


pub fn map_component(params: String, CowPools: pools) -> Result<> {
    substreams::log::info!("--- map_components called ---");   
    substreams::log::info!("Received params: {}", params);
    substreams::log::info!("Block number: {}", block.number);

        Ok(BlockTransactionProtocolComponents {
            tx_components: 
            pools
                .iter()
                .filter_map(|pool|
                    let components = pools
                        .iter()
                        .map(|pool| {
                            build_component(pool)
                        })
                        .collect::<Vec<_>>();
            
                    if !components.is_empty() {
                        Some(TransactionProtocolComponents { tx: Some(pool.created_tx_hash), components }) 
                    } else {
                        None
                    }            
                )
                .collect::<Vec<_>>(),
    })
}
