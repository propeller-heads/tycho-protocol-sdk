use serde::Deserialize;
use substreams_ethereum::Event;
use substreams_ethereum::pb::eth::v2::{Call, Log, TransactionTrace};
use tiny_keccak::{Hasher, Keccak};
use tycho_substreams::models::{ChangeType, FinancialType, ImplementationType, ProtocolComponent, ProtocolType};
use tycho_substreams::prelude::Attribute;

#[derive(Deserialize)]
pub struct DeploymentConfig {
    #[serde(with = "hex::serde")]
    pub core: Vec<u8>,
    #[serde(with = "hex::serde")]
    pub oracle: Vec<u8>,
}

/// Potentially constructs a new ProtocolComponent given a call
///
/// This method is given each individual call within a transaction, the corresponding
/// logs emitted during that call as well as the full transaction trace.
///
/// If this call creates a component in your protocol please contstruct and return it
/// here. Otherwise, simply return None.
pub fn maybe_create_component(
    call: &Call,
    log: &Log,
    _tx: &TransactionTrace,
    config: &DeploymentConfig,
) -> Option<ProtocolComponent> {
    if call.address == config.core {
        if let Some(pi) = crate::abi::core::events::PoolInitialized::match_and_decode(log) {
            let pool_id = hash_pool_key(&pi.pool_key);

            return Some(ProtocolComponent {
                id: pool_id,
                tokens: vec![pi.pool_key.0.clone(), pi.pool_key.1.clone()],
                contracts: if config.oracle == pi.pool_key.4 {
                    vec![
                        config.oracle.clone(),
                        config.core.clone(),
                    ]
                } else {
                    vec![config.core.clone()]
                },
                change: ChangeType::Creation.into(),
                protocol_type: Some(ProtocolType {
                    name: "EKUBO".to_string(),
                    financial_type: FinancialType::Swap.into(),
                    implementation_type: ImplementationType::Vm.into(),
                    attribute_schema: vec![],
                }),
                static_att: vec![
                    Attribute {
                        change: ChangeType::Creation.into(),
                        name: "token0".to_string(),
                        value: pi.pool_key.0,
                    },
                    Attribute {
                        change: ChangeType::Creation.into(),
                        name: "token1".to_string(),
                        value: pi.pool_key.1,
                    },
                    Attribute {
                        change: ChangeType::Creation.into(),
                        name: "fee".to_string(),
                        value: pi.pool_key.2.to_bytes_be().1,
                    },
                    Attribute {
                        change: ChangeType::Creation.into(),
                        name: "tick_spacing".to_string(),
                        value: pi.pool_key.3.to_bytes_be().1,
                    },
                    Attribute {
                        change: ChangeType::Creation.into(),
                        name: "extension".to_string(),
                        value: pi.pool_key.4,
                    },
                ],
            });
        }
    };

    None
}


pub type PoolKey = (
    Vec<u8>,
    Vec<u8>,
    substreams::scalar::BigInt,
    substreams::scalar::BigInt,
    Vec<u8>,
);

pub fn hash_pool_key(pool_key: &PoolKey) -> String {
    let mut hasher = Keccak::v256();
    hasher.update(pool_key.0.as_slice());
    hasher.update(pool_key.1.as_slice());
    hasher.update(pool_key.2.to_signed_bytes_be().as_slice());
    hasher.update(pool_key.3.to_signed_bytes_be().as_slice());
    hasher.update(pool_key.4.as_slice());

    let mut output = [0u8; 32];
    hasher.finalize(&mut output);
    hex::encode(output)
}