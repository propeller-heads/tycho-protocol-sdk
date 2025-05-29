use std::collections::HashMap;
use crate::{modules::utils::Params};
use crate::pb::cowamm::{CowPool, CowPools, CowPoolBind, CowPoolBinds, CowPoolCreation, CowPoolCreations};
use anyhow::{Ok, Result};
use ethabi::ethereum_types::Address;
use substreams::{prelude::BigInt};
use hex_literal::hex;
use serde::Deserialize;
use tycho_substreams::prelude::*;

#[substreams::handlers::map]
pub fn map_cowpools(creations: CowPoolCreations, binds: CowPoolBinds) -> Result<CowPools, substreams::errors::Error> {
    
    //combine cow_pool_binds and cow_pool_creations based on the address of the pool
    // like an left join in SQL 

    //hashmap of pool addresses with token and its weight
    //should we turn the vec to a hex address? i think that'll be better
    let mut binds_by_address: HashMap<Vec<u8>, Vec<(Vec<u8>, u64)>> = HashMap::new();

    for bind in binds.binds {
        binds_by_address
            .entry(bind.address.clone())
            .or_default()
            .push((bind.token, bind.weight));  
            // in this hashmap if we have more than one element with the same key, it gets pushed to it 
            // so we'll have Vec<Vec<u8> Vec<(Vec<u8>, u64)>>> for a pool with two tokens 
    }

    // map the tx_hash to its TransactionTrace
    let mut txs_by_hash = HashMap<Vec<u8>, Transaction> = HashMap::new();
    for creation in creation.pools {
        txs_by_hash
            .entry(creation.created_tx_hash.clone())
            .or_default()
            .push(creation.tx);  
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
                // tx: creation.tx, // why is it an Option?
                created_tx_hash: creation.created_tx_hash.clone(),
            });

        }
    }

    Ok(CowPoolsWithTx { pools , txs})
}

