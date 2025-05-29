use std::collections::HashSet;
use crate::pb::cowamm::{CowPoolCreations,CowPoolBind, CowPoolBinds};
use anyhow::{Ok, Result};
use substreams_ethereum::pb::eth::v2::{Block};

#[substreams::handlers::map]
pub fn map_cowpool_binds(block: Block, creations: CowPoolCreations) -> Result<CowPoolBinds> {
    const BIND_TOPIC: &str = "0xe4e1e53800000000000000000000000000000000000000000000000000000000";

    let creation_addrs: HashSet<&Vec<u8>> = creations.pools.iter().map(|c| &c.address).collect();

    let cowpool_binds = block
        .logs()
        .filter(|log| {
            log.topics()
            .get(0)
            .map(|t| hex::encode(t)).as_ref() == Some(&BIND_TOPIC.to_string()) &&
            creation_addrs.contains(&log.address().to_vec())
        })
        .filter_map(|log| {
            let data = &log.data();
            if data.len() < 165 { return None; } // is this necessary?
            let token = data.get(80..100)?.to_vec();
            let weight_bytes = data.get(132..164)?;
            let weight = substreams::scalar::BigInt::from_signed_bytes_be(weight_bytes).to_u64(); //change this to an expect? //
 
            Some(
               CowPoolBind {
                address: log.address().to_vec(),
                token,
                weight: weight, 
            })
        })
        .collect::<Vec<CowPoolBind>>();

     Ok(CowPoolBinds { binds: cowpool_binds })
}

