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
use substreams_helper::{hex::Hexable};

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
        
        //calculate normalized weight
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

#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;

    #[test]
    fn test_parse_binds_single_entry() {
        let bind_str = r#"{\"address\":\"9bd702e05b9c97e4a4a3e47df1e0fe7a0c26d2f1\",\"token\":\"def1ca1fb7fbcdc777520aa7f396b4e015f497ab\",\"weight\":\"0000000000000000000000000000000000000000000000000de0b6b3a7640000\"};"#;
        let result = parse_binds(bind_str);

        assert!(result.is_some());
        let binds = result.unwrap();
        assert_eq!(binds.len(), 1);

        assert_eq!(binds[0].address, hex!("9bd702e05b9c97e4a4a3e47df1e0fe7a0c26d2f1"));
        assert_eq!(binds[0].token, hex!("def1ca1fb7fbcdc777520aa7f396b4e015f497ab"));
        assert_eq!(binds[0].weight, hex!("0000000000000000000000000000000000000000000000000de0b6b3a7640000"));
    }

    #[test]
    fn test_parse_binds_multiple_entries() { // change to to an actual proper string lol 
        let bind_str = r#"{\"address\":\"9bd702e05b9c97e4a4a3e47df1e0fe7a0c26d2f1\",\"token\":\"def1ca1fb7fbcdc777520aa7f396b4e015f497ab\",\"weight\":\"0000000000000000000000000000000000000000000000000de0b6b3a7640000\"};{\"address\":\"9bd702e05b9c97e4a4a3e47df1e0fe7a0c26d2f1\",\"token\":\"7f39c581f595b53c5cb19bd0b3f8da6c935e2ca0\",\"weight\":\"0000000000000000000000000000000000000000000000000de0b6b3a7640000\"};"  bind_last : "{\"address\":\"9bd702e05b9c97e4a4a3e47df1e0fe7a0c26d2f1\",\"token\":\"def1ca1fb7fbcdc777520aa7f396b4e015f497ab\",\"weight\":\"0000000000000000000000000000000000000000000000000de0b6b3a7640000\"};{\"address\":\"9bd702e05b9c97e4a4a3e47df1e0fe7a0c26d2f1\",\"token\":\"7f39c581f595b53c5cb19bd0b3f8da6c935e2ca0\",\"weight\":\"0000000000000000000000000000000000000000000000000de0b6b3a7640000\"};"#;
        let result = parse_binds(bind_str);

        assert!(result.is_some());
        let binds = result.unwrap();
        assert_eq!(binds.len(), 2);
        assert_eq!(binds[0].address, hex!("9bd702e05b9c97e4a4a3e47df1e0fe7a0c26d2f1"));
        assert_eq!(binds[0].token, hex!("def1ca1fb7fbcdc777520aa7f396b4e015f497ab"));
        
        assert_eq!(binds[1].address, hex!("9bd702e05b9c97e4a4a3e47df1e0fe7a0c26d2f1"));
        assert_eq!(binds[1].token, hex!("def1ca1fb7fbcdc777520aa7f396b4e015f497ab"));
    }

    #[test]
    fn test_parse_binds_invalid_json() {
        let bind_str = r#"invalid_json"#;
        let result = parse_binds(bind_str);
        assert!(result.is_none());
    }

    #[test]
    fn test_weight_normalization_60_40() {
        let weight_a = substreams::scalar::BigInt::from(600000000000000000u64); // 60 * 1e18
        let weight_b = substreams::scalar::BigInt::from(400000000000000000u64); // 40 * 1e18
        let bone = substreams::scalar::BigInt::from(100000000000000000u64);     // 1e18

        let wa = &weight_a / &bone;
        let wb = &weight_b / &bone;
        let total = &wa + &wb;

        let norm_a = substreams::scalar::BigInt::from(100) * &wa / &total;
        let norm_b = substreams::scalar::BigInt::from(100) * &wb / &total;

        assert_eq!(norm_a.to_u64(), 60);
        assert_eq!(norm_b.to_u64(), 40);
    }

    #[test]
    fn test_weight_normalization_50_50() {
        let weight = substreams::scalar::BigInt::from(500000000000000000u64);
        let bone = substreams::scalar::BigInt::from(100000000000000000u64);

        let wa = &weight / &bone;
        let wb = &weight / &bone;
        let total = &wa + &wb;

        let norm_a = substreams::scalar::BigInt::from(100) * &wa / &total;
        let norm_b = substreams::scalar::BigInt::from(100) * &wb / &total;

        assert_eq!(norm_a.to_u64(), 50);
        assert_eq!(norm_b.to_u64(), 50);
    }

    #[test]
    fn test_weight_normalization_unbalanced() {
        let weight_a = substreams::scalar::BigInt::from(300000000000000000u64); // 30
        let weight_b = substreams::scalar::BigInt::from(700000000000000000u64); // 70
        let bone = substreams::scalar::BigInt::from(100000000000000000u64);     // 1e18

        let wa = &weight_a / &bone;
        let wb = &weight_b / &bone;
        let total = &wa + &wb;

        let norm_a = substreams::scalar::BigInt::from(100) * &wa / &total;
        let norm_b = substreams::scalar::BigInt::from(100) * &wb / &total;

        assert_eq!(norm_a.to_u64(), 30);
        assert_eq!(norm_b.to_u64(), 70);
    }
}
