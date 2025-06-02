use crate::{modules::utils::Params};
use crate::pb::cowamm::{CowPoolCreation, CowPoolCreations};
use anyhow::{Ok, Result};
use substreams_ethereum::pb::eth::v2::{Block};
use substreams::log::info;
use substreams_helper::hex::Hexable;

//This is the first topic (topics[0]) that is present in COWAMMPoolCreated events 
//sample  = https://etherscan.io/tx/0x124f2fa9c181003529e34c22e7380b505f1f5e18e44c3868560e4ddc724cc191#eventlog
//the remaining part of topics[1] the last topic is the address of the BCowPool

//address of CowPool creations
#[substreams::handlers::map]
pub fn map_cowpool_creations(params: String, block: Block) -> Result<CowPoolCreations> {
    const COWAMM_POOL_CREATED_TOPIC: &str = "0x0d03834d0d86c7f57e877af40e26f176dc31bd637535d4ba153d1ac9de88a7ea";

    let params = Params::parse_from_query(&params)?;
    let factory_address = params.decode_addresses().unwrap();

    let cow_pool_creations = block
        .logs()
        .filter(|log| {     
            log.address() == factory_address && 
            log.topics()
                .get(0)
                .map(|t| t.to_hex()) == Some(COWAMM_POOL_CREATED_TOPIC.to_string())
        })
        .filter_map(|log| {                   
            let address = &log
                .topics()
                .get(1)
                .map(|topic| topic.as_slice()[12..].to_vec())?; // we get the last 20 bytes which is BCowPool address
            
            let tx_hash = log.receipt.transaction.hash.clone();
            Some(CowPoolCreation {
                address: address.clone(), 
                lp_token: address.clone(), //address of lptoken is same as the pool address
                created_tx_hash: tx_hash,
            })
        })
        .collect::<Vec<CowPoolCreation>>();
    Ok(CowPoolCreations { pools: cow_pool_creations })
}


