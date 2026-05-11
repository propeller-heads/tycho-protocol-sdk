use serde::Deserialize;

#[derive(Deserialize)]
pub struct DeploymentConfig {
    #[serde(with = "hex::serde")]
    pub adapter_address: Vec<u8>,
}
