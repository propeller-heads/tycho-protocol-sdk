use std::iter;

use itertools::Itertools;
use serde::Deserialize;
use substreams_ethereum::Event;
use substreams_ethereum::pb::eth::v2::{Call, Log, TransactionTrace};
use tycho_substreams::models::{ChangeType, FinancialType, ImplementationType, ProtocolComponent, ProtocolType};
use tycho_substreams::prelude::Attribute;

use crate::identifiers::{pool_id_to_component_id, PoolKey};

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
            let pool_key = PoolKey::from(pi.pool_key);

            let contracts = iter
                ::once(config.core.clone())
                .chain((config.oracle == pool_key.config.extension).then(|| config.oracle.clone()))
                .collect_vec();

            return Some(ProtocolComponent {
                id: pool_id_to_component_id(pi.pool_id),
                tokens: vec![pool_key.token0.clone(), pool_key.token1.clone()],
                contracts,
                change: ChangeType::Creation.into(),
                protocol_type: Some(ProtocolType {
                    name: "ekubo".to_string(),
                    financial_type: FinancialType::Swap.into(),
                    implementation_type: ImplementationType::Vm.into(),
                    attribute_schema: vec![],
                }),
                // NOTE: Order of attributes matters (used in store_pool_details)
                static_att: vec![
                    Attribute {
                        change: ChangeType::Creation.into(),
                        name: "token0".to_string(),
                        value: pool_key.token0,
                    },
                    Attribute {
                        change: ChangeType::Creation.into(),
                        name: "token1".to_string(),
                        value: pool_key.token1,
                    },
                    Attribute {
                        change: ChangeType::Creation.into(),
                        name: "fee".to_string(),
                        value: pool_key.config.fee,
                    },
                    Attribute {
                        change: ChangeType::Creation.into(),
                        name: "tick_spacing".to_string(),
                        value: pool_key.config.tick_spacing,
                    },
                    Attribute {
                        change: ChangeType::Creation.into(),
                        name: "extension".to_string(),
                        value: pool_key.config.extension,
                    },
                ],
            });
        }
    };

    None
}
