use crate::pb::cowamm::{CowPoolBind, CowPoolBinds};
use anyhow::{Ok, Result};
use substreams_ethereum::pb::eth::v2::{Block};
use substreams_helper::hex::Hexable;


#[substreams::handlers::map]
pub fn map_cowpool_binds(block: Block) -> Result<CowPoolBinds> {
    const BIND_TOPIC: &str = "0xe4e1e53800000000000000000000000000000000000000000000000000000000";

    let cowpool_binds = block
    .logs()
    .filter(|log| {
        log.topics()
        .get(0)
        .map(|t| t.to_hex()) == Some(BIND_TOPIC.to_string())
    })
        .filter_map(|log| {
            let data = &log.data();
            let address = log.address().to_vec(); 
            if data.len() < 165 { return None; } 
            let token = data.get(80..100)?.to_vec();
            let weight_bytes = data.get(132..164)?;
            Some(
               CowPoolBind {
                address: address,
                token,
                weight: weight_bytes.to_vec(), 
                //is it here we would get the liquidity? but the issue is the liq when the pool was bound is not the same as subsequently
                
            })
        })
        .collect::<Vec<CowPoolBind>>();

     Ok(CowPoolBinds { binds: cowpool_binds })
}

