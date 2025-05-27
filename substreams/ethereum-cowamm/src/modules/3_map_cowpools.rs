use std::collections::HashMap;
use crate::{modules::utils::Params};
use crate::pb::cowamm::{CowPool, CowPools, CowpoolBind, CowPoolBinds, CowPoolCreation, CowPoolCreations};
use anyhow::{Ok, Result};
use ethabi::ethereum_types::Address;
use substreams::{prelude::BigInt};
use hex_literal::hex;
use serde::Deserialize;
use substreams_ethereum::pb::eth::v2::{Block};
use substreams_ethereum::Event;
use tycho_substreams::prelude::*;

#[substreams::handlers::map]
pub fn map_cowpools(creations: CowPoolCreations, binds: CowPoolBinds>) -> Result<CowPool, substreams::errors::Error> {
    
    //combine cow_pool_binds and cow_pool_creations based on the address of the pool
    // like an left join in SQL 

    //hashmap of pool addresses with token and weight
    //should we turn the vec to a hex address? i think that'll be better
    let mut binds_by_address: HashMap<Vec<u8>, Vec<Vec<u8>, u64>> = HashMap::new();

    for bind in binds.binds {
        binds_by_address
            .entry(bind.address.clone())
            .or_default()
            .push((bind.token, bind.weight));  
            // in this hashmap if we have more than one element with the same key, it gets pushed to it 
            // so we'll have Vec<Vec<Vec<u8>, u64>, Vec<Vec<u8>, u64>> for a pool with two tokens 
    }
    // Create pools by joining on address
    let mut pools: Vec<CowPool> = Vec::new();
    for creation in creations.pools {
        if let Some(binds) = binds_by_address.get(&creation.address) {
            if tokens.len() != 2 {
                continue; // skip incomplete pools
            }

            let (token_a, weight_a, token_b, weight_b) = {
                let (first, second) = (&binds[0], &binds[1]);
                if first.0 < second.0 {
                    (first.0.clone(), first.1, second.0.clone(), second.1)
                } else {
                    (second.0.clone(), second.1, first.0.clone(), first.1)
                }
            };

            pools.push(CowPool {
                address: creation.address.clone(),
                token_a,
                token_b,
                lp_token: creation.lp_token.clone(),
                weight_a,
                weight_b,
                fee: 0, 
                created_tx_hash: creation.created_tx_hash.clone(),
            });
        }
    }

    Ok(CowPools { pools })
}

