use crate::{abi, modules::VAULT_ADDRESS};
use abi::{
    stable_pool_factory_contract::{
        events::PoolCreated as StablePoolCreated, functions::Create as StablePoolCreate,
    },
    weigthed_pool_factory_contract::{
        events::PoolCreated as WeightedPoolCreated, functions::Create as WeightedPoolCreate,
    },
};
use substreams::hex;
use substreams_ethereum::{
    pb::eth::v2::{Call, Log, TransactionTrace},
    Event, Function,
};
use tycho_substreams::{attributes::json_serialize_bigint_list, prelude::*};

pub fn address_map(
    pool_factory_address: &[u8],
    log: &Log,
    call: &Call,
    tx: &TransactionTrace,
) -> Option<ProtocolComponent> {
    match *pool_factory_address {
        hex!("201efd508c8DfE9DE1a13c2452863A78CB2a86Cc") => {
            let WeightedPoolCreate {
                tokens: token_config,
                normalized_weights,
                swap_fee_percentage,
                ..
            } = WeightedPoolCreate::match_and_decode(call)?;
            let WeightedPoolCreated { pool } = WeightedPoolCreated::match_and_decode(log)?;
            let tokens = token_config
                .into_iter()
                .map(|t| t.0)
                .collect::<Vec<_>>();

            Some(
                ProtocolComponent::new(
                    &format!("0x{}", hex::encode(pool.to_owned())),
                    &(tx.into()),
                )
                .with_contracts(&[pool, VAULT_ADDRESS.to_vec()])
                .with_tokens(tokens.as_slice())
                .with_attributes(&[
                    ("pool_type", "WeightedPoolFactory".as_bytes()),
                    (
                        "normalized_weights",
                        &json_serialize_bigint_list(normalized_weights.as_slice()),
                    ),
                    ("fee", &swap_fee_percentage.to_signed_bytes_be()),
                    ("manual_updates", &[1u8]),
                ])
                .as_swap_type("balancer_v3_pool", ImplementationType::Vm),
            )
        }
        hex!("DB8d758BCb971e482B2C45f7F8a7740283A1bd3A") => {
            let StablePoolCreate { tokens: token_config, swap_fee_percentage, .. } =
                StablePoolCreate::match_and_decode(call)?;
            let StablePoolCreated { pool } = StablePoolCreated::match_and_decode(log)?;
            let tokens = token_config
                .into_iter()
                .map(|t| t.0)
                .collect::<Vec<_>>();

            Some(
                ProtocolComponent::new(
                    &format!("0x{}", hex::encode(pool.to_owned())),
                    &(tx.into()),
                )
                .with_contracts(&[pool.to_owned(), VAULT_ADDRESS.to_vec()])
                .with_tokens(tokens.as_slice())
                .with_attributes(&[
                    ("pool_type", "StablePoolFactory".as_bytes()),
                    ("bpt", &pool),
                    ("fee", &swap_fee_percentage.to_signed_bytes_be()),
                    ("manual_updates", &[1u8]),
                ])
                .as_swap_type("balancer_v2_pool", ImplementationType::Vm),
            )
        }
        _ => None,
    }
}
