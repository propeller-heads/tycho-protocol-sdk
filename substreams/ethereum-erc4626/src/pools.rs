use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;

const PARAMS_SEPERATOR: &str = "#";

#[derive(Debug, Deserialize, PartialEq)]
pub struct PoolParams {
    pub name: String,
    pub address: String,
    pub contracts: Option<Vec<String>>,
    pub tx_hash: String,
    pub asset: String,
    pub static_attribute_keys: Option<Vec<String>>,
    pub static_attribute_vals: Option<Vec<String>>,
}

impl PoolParams {
    pub fn parse_params(params: &str) -> Result<HashMap<String, Vec<PoolParams>>, anyhow::Error> {
        let mut pools: HashMap<String, Vec<PoolParams>> = HashMap::new();
        for param in params.split(PARAMS_SEPERATOR) {
            let pool: PoolParams = serde_qs::from_str(param)
                .with_context(|| format!("Failed to parse pool params: {param}"))?;
            pools
                .entry(pool.tx_hash.clone())
                .or_default()
                .push(pool);
        }
        Ok(pools)
    }
}
