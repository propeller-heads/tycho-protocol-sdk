use std::collections::HashMap;
use crate::{modules::utils::Params};
use crate::pb::cowamm::{CowPool, CowPools, CowPoolBinds, CowPoolCreations};
use anyhow::{Ok, Result};

#[substreams::handlers::map]
pub fn map_cowpools(creations: CowPoolCreations, binds: CowPoolBinds) -> Result<CowPools, substreams::errors::Error> {

    //hashmap of pool addresses with each token and its weight
    let mut binds_by_address: HashMap<Vec<u8>, Vec<(Vec<u8>, u64)>> = HashMap::new();

    for bind in binds.binds {
        binds_by_address
            .entry(bind.address.clone())
            .or_default()
            .push((bind.token, bind.weight));  
    }

    // Create pools by joining on address
    let mut pools: Vec<CowPool> = Vec::new();

    for creation in creations.pools {
        if let Some(binds) = binds_by_address.get(&creation.address) {
            if binds.len() != 2 {
                continue; // skip incomplete pools if any 
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

