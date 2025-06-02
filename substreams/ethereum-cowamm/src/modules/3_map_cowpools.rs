use serde_json;
use crate::{modules::utils::Params};
use serde::{Deserialize, Serialize};
use crate::pb::cowamm::{CowPool, CowPools, CowPoolBind, CowPoolCreations};
use anyhow::{Ok, Result};
use substreams::{
    pb::substreams::StoreDeltas,
    store::{StoreGet, StoreGetString},
};
use substreams::log::info;

#[derive(Debug, Deserialize, Serialize)]
struct CowPoolBindJson {
    address: String,
    token: String,
    weight: String,
}

fn parse_binds(bind_str: &str) -> Option<Vec<CowPoolBind>> {
    let bind_strs: Vec<&str> = bind_str.split(';').collect();
    let mut binds = Vec::new();
    for bind in bind_strs {
        let bind = bind.trim();
        // Skip empty strings (which can happen if there are extra semicolons)
        if bind.is_empty() {
            continue;
        }
        // Wrap the bind in square brackets to create an array of JSON objects
        let formatted_str = format!("[{}]", bind.replace("};", "},"));
        let parsed: Vec<CowPoolBindJson> = serde_json::from_str(&formatted_str).ok()?;
        for bind_json in parsed {
            let cow_bind = CowPoolBind {
                address: hex::decode(&bind_json.address).expect("Invalid hex for address"),
                token: hex::decode(&bind_json.token).expect("Invalid hex for token"),
                weight: hex::decode(&bind_json.weight).expect("Invalid hex for weight"),
            };
            binds.push(cow_bind);
        }
    }
    if binds.is_empty() {
        None
    } else {
        Some(binds)
    }
}



#[substreams::handlers::map]
pub fn map_cowpools(creations: CowPoolCreations, binds: StoreGetString) -> Result<CowPools, substreams::errors::Error> {
    let mut pools: Vec<CowPool> = Vec::new();

    let creations = &creations;
    let binds = &binds;
    
    for creation in creations.pools.iter() {
        let base_key = hex::encode(&creation.address);
        let bind_first = match binds.get_first(&base_key) {
            Some(data) => data,
            None => continue, // skip if no bind found
        };

        let parsed_binds = match parse_binds(&bind_first) {
            Some(binds) if binds.len() == 2 => binds,
            _ => continue, // skip if parsing fails or not enough binds
        };

        let bind1 = &parsed_binds[0];
        let bind2 = &parsed_binds[1];

        let (token_a, weight_a, token_b, weight_b) = if bind1.token < bind2.token {
            (&bind1.token, &bind1.weight, &bind2.token, &bind2.weight)
        } else {
            (&bind2.token, &bind2.weight, &bind1.token, &bind1.weight)
        };
        
        let w1 = substreams::scalar::BigInt::from_unsigned_bytes_be(&weight_a);   
        let w2 = substreams::scalar::BigInt::from_unsigned_bytes_be(&weight_b);  

        let BONE = substreams::scalar::BigInt::from(100000000000000000u64); 

        let weight_a_scaled_down = &w1 / &BONE; 
        let weight_b_scaled_down = &w2 / &BONE;
        let total = &weight_a_scaled_down + &weight_b_scaled_down;
 
        let normalized_weight_a = substreams::scalar::BigInt::from(100) * weight_a_scaled_down / (&total);
        let normalized_weight_b = substreams::scalar::BigInt::from(100) * weight_b_scaled_down / (&total);

        pools.push(CowPool {
            address: creation.address.clone(), 
            token_a: token_a.clone(),
            token_b: token_b.clone(),
            lp_token: creation.lp_token.clone(),
            weight_a: normalized_weight_a.to_u64(),
            weight_b: normalized_weight_b.to_u64(),
            fee: 0,
            created_tx_hash: creation.created_tx_hash.clone(),
        });
    }

    Ok(CowPools { pools })
}


//unit test for parsing bind_str 

//unit test for weight calculation