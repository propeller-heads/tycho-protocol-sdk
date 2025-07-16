use anyhow::{anyhow, Result};
use serde::Deserialize;
use crate::abi::b_cow_pool::functions;
use substreams::scalar::BigInt;
use substreams::Hex;

#[derive(Debug, Deserialize)]
pub struct Params {
    pub factory_address: String,
}

impl Params {
    pub fn parse_from_query(input: &str) -> Result<Self> {
        serde_qs::from_str(input).map_err(|e| anyhow!("Failed to parse query params: {}", e))
    }

    pub fn decode_addresses(&self) -> Result<[u8; 20]> {
        let factory_address =
            hex::decode(&self.factory_address).map_err(|e| anyhow!("Invalid factory address hex: {}", e))?;

        if factory_address.len() != 20 {
            return Err(anyhow!("factory address must be 20 bytes"));
        }

        Ok(factory_address.try_into().unwrap())
    }
}

pub fn get_lp_token_supply(token_address: String) -> Vec<u8> {
    // Try to decode the hex string into bytes
    let Ok(token_address_vec) = Hex::decode(token_address) else {
        substreams::log::info!("Failed to decode token address");
        return vec![];
    };

    // Try to call totalSupply and convert to bytes if successful
     functions::TotalSupply {}
        .call(token_address_vec)
        .map_or_else(|| vec![], |total_supply| total_supply.to_signed_bytes_be())
}