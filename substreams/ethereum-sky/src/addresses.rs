use anyhow::Result;
use substreams::hex;

#[derive(Debug, Clone)]
pub struct SkyAddresses {
    // Components
    pub sdai: [u8; 20],
    pub dai_usds_converter: [u8; 20],
    pub dai_lite_psm: [u8; 20],
    pub usds_psm_wrapper: [u8; 20],
    pub susds: [u8; 20],
    pub mkr_sky_converter: [u8; 20],
    // Tokens
    pub dai: [u8; 20],
    pub usds: [u8; 20],
    pub usdc: [u8; 20],
    pub mkr: [u8; 20],
    pub sky: [u8; 20],
    // Liquidity holders
    pub dai_lite_psm_usdc_holder: [u8; 20],
}

impl SkyAddresses {
    pub fn from_params(params: &str) -> Result<Self> {
        let addresses: Vec<&str> = params
            .split(',')
            .map(str::trim)
            .collect();

        if addresses.len() != 12 {
            anyhow::bail!("Expected 12 addresses, got {}", addresses.len());
        }

        Ok(Self {
            sdai: hex::decode(addresses[0])?.try_into()?,
            dai: hex::decode(addresses[1])?.try_into()?,
            dai_usds_converter: hex::decode(addresses[2])?.try_into()?,
            dai_lite_psm: hex::decode(addresses[3])?.try_into()?,
            usds_psm_wrapper: hex::decode(addresses[4])?.try_into()?,
            susds: hex::decode(addresses[5])?.try_into()?,
            mkr_sky_converter: hex::decode(addresses[6])?.try_into()?,
            usds: hex::decode(addresses[7])?.try_into()?,
            usdc: hex::decode(addresses[8])?.try_into()?,
            mkr: hex::decode(addresses[9])?.try_into()?,
            sky: hex::decode(addresses[10])?.try_into()?,
            dai_lite_psm_usdc_holder: hex::decode(addresses[11])?.try_into()?,
        })
    }
}
