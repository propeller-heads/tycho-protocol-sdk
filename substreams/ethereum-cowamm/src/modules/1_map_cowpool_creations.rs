use crate::{modules::utils::Params};
use crate::pb::cowamm::{CowPoolCreation, CowPoolCreations};
use anyhow::{Ok, Result};
use substreams_ethereum::pb::eth::v2::{Block};
use substreams_helper::hex::Hexable;


#[substreams::handlers::map]
pub fn map_cowpool_creations(params: String, block: Block) -> Result<CowPoolCreations> {
    const COWAMM_POOL_CREATED_TOPIC: &str = "0x0d03834d0d86c7f57e877af40e26f176dc31bd637535d4ba153d1ac9de88a7ea";

    let params = Params::parse_from_query(&params)?;
    let factory_address = params.decode_addresses().expect("unable to extract factory address");

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
            // filter again , get the token supply of the pool, if it is zero, ignore it and done index it
            // but now the problem is that from the data it actually shows it has some supply, because its getting its supply from the start of the block, when it was initialized, thats why their supplies are the same 
            // so we don't bother about it imo, its value will be taken care of in the balances changes 
            //what we want to do now is to check if lp token balance tracking through the balance is right - check with the transaction hash
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


