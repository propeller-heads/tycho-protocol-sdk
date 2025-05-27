use crate::pb::cowamm::{CowPoolCreations,CowpoolBind, CowPoolBinds};
use anyhow::{Ok, Result};
use ethabi::ethereum_types::Address;
use substreams::{prelude::BigInt};
use hex_literal::hex;
use serde::Deserialize;
use substreams_ethereum::pb::eth::v2::{Call,Block, Log,};
use tycho_substreams::prelude::*;

#[substreams::handlers::map]
pub fn map_cowpool_binds(block: Block, creations: CowpoolCreations) -> Vec<CowpoolBind> {
    const BIND_TOPIC: &str = "0xe4e1e53800000000000000000000000000000000000000000000000000000000";

    let creation_addrs: std::collections::HashSet<&Vec<u8>> = creations.creations.iter().map(|c| &c.address).collect();

    let cowpool_binds = block
        .logs()
        .filter(|log| {
            log.topics.get(0).map(|t| hex::encode(t)) == Some(BIND_TOPIC) &&
            creation_addrs.contains(&log.address)
        })
        .filter_map(|log| {
            let data = &log.data;
            if data.len() < 165 { return None; } // is this necessary?
            let token = data.get(80..100)?.to_vec();
            let weight_bytes = data.get(132..164)?;
            let weight = substreams::scalar::BigInt::from_bytes_be(weight_bytes).to_u64(); //change this to an expect?
 
            Some(
               CowPoolBind {
                address: log.address.clone(),
                token,
                weight: weight.to_u64().unwrap_or(0), //is this necessary? or it should just panic
            })
        })
        .collect<Vec<CowPoolBind>>();

     Ok(CowPoolBinds { cowpool_binds })
}

