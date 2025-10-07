use anyhow::{anyhow, Result};
use serde::Deserialize;
use tiny_keccak::{Hasher, Keccak};

#[derive(Debug, Deserialize)]
pub struct Params {
    pub dpp_factory: String,
    pub dsp_factory: String,
    pub dvm_factory: String,
    pub gsp_factory: String,
}

pub struct DodoFactories {
    pub dpp_factory: [u8; 20],
    pub dsp_factory: [u8; 20],
    pub dvm_factory: [u8; 20],
    pub gsp_factory: [u8; 20],
}

impl Params {
    pub fn parse_from_query(input: &str) -> Result<Self> {
        serde_qs::from_str(input).map_err(|e| anyhow!("Failed to parse query params: {}", e))
    }

    pub fn decode_addresses(&self) -> Result<DodoFactories> {
        let dpp_factory =
            hex::decode(&self.dpp_factory).map_err(|e| anyhow!("Invalid factory hex: {}", e))?;
        let dsp_factory =
            hex::decode(&self.dsp_factory).map_err(|e| anyhow!("Invalid factory hex: {}", e))?;
        let dvm_factory =
            hex::decode(&self.dvm_factory).map_err(|e| anyhow!("Invalid factory hex: {}", e))?;
        let gsp_factory =
            hex::decode(&self.gsp_factory).map_err(|e| anyhow!("Invalid factory hex: {}", e))?;
        if dpp_factory.len() != 20 ||
            dsp_factory.len() != 20 ||
            dvm_factory.len() != 20 ||
            gsp_factory.len() != 20
        {
            return Err(anyhow!("Addresses must be 20 bytes"));
        }
        Ok(DodoFactories {
            dpp_factory: dpp_factory.try_into().unwrap(),
            dsp_factory: dsp_factory.try_into().unwrap(),
            dvm_factory: dvm_factory.try_into().unwrap(),
            gsp_factory: gsp_factory.try_into().unwrap(),
        })
    }
}

pub fn fn_selector(signature: &str) -> [u8; 4] {
    let mut hasher = Keccak::v256();
    let mut hash = [0u8; 32];
    hasher.update(signature.as_bytes());
    hasher.finalize(&mut hash);
    let mut selector = [0u8; 4];
    selector.copy_from_slice(&hash[..4]);
    selector
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;

    #[test]
    fn test_sell_shares_selector() {
        let sig = "sellShares(uint256,address,uint256,uint256,bytes,uint256)";
        let expected = hex!("b56ceaa6");
        assert_eq!(fn_selector(sig), expected);
    }

    #[test]
    fn test_buy_shares_selector() {
        let sig = "buyShares(address)";
        let expected = hex!("4c85b425");
        assert_eq!(fn_selector(sig), expected);
    }
}
