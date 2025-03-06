pub fn pool_id_to_component_id<T: AsRef<[u8]>>(pool_id: T) -> String {
    format!("0x{}", hex::encode(pool_id))
}

pub struct PoolConfig {
    pub fee: Vec<u8>,
    pub tick_spacing: Vec<u8>,
    pub extension: Vec<u8>,
}

impl From<[u8; 32]> for PoolConfig {
    fn from(value: [u8; 32]) -> Self {
        Self {
            tick_spacing: value[28..32].into(),
            fee: value[20..28].into(),
            extension: value[..20].into(),
        }
    }
}

pub struct PoolKey {
    pub token0: Vec<u8>,
    pub token1: Vec<u8>,
    pub config: PoolConfig,
}

impl From<(Vec<u8>, Vec<u8>, [u8; 32])> for PoolKey {
    fn from(value: (Vec<u8>, Vec<u8>, [u8; 32])) -> Self {
        Self {
            token0: value.0,
            token1: value.1,
            config: value.2.into(),
        }
    }
}
