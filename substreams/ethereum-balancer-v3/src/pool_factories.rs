use crate::{abi, modules::VAULT_ADDRESS};
use abi::{
    stable_pool_factory_contract::{
        events::PoolCreated as StablePoolCreated, functions::Create as StablePoolCreate,
    },
    vault_contract::events::LiquidityAddedToBuffer,
    weigthed_pool_factory_contract::{
        events::PoolCreated as WeightedPoolCreated, functions::Create as WeightedPoolCreate,
    },
};
use substreams::{hex, log};
use substreams_ethereum::{
    Event, Function,
    pb::eth::v2::{Call, Log, TransactionTrace},
};
use tycho_substreams::{
    abi::erc20::events::Transfer as ERC20Transfer, attributes::json_serialize_bigint_list,
    prelude::*,
};

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

            log::info!("weighted pool created: {:?}", pool);

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
        hex!("B9d01CA61b9C181dA1051bFDd28e1097e920AB14") => {
            let StablePoolCreate { tokens: token_config, swap_fee_percentage, .. } =
                StablePoolCreate::match_and_decode(call)?;
            let StablePoolCreated { pool } = StablePoolCreated::match_and_decode(log)?;
            let tokens = token_config
                .into_iter()
                .map(|t| t.0)
                .collect::<Vec<_>>();

            log::info!("stable pool created: {:?}", pool);

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
                .as_swap_type("balancer_v3_pool", ImplementationType::Vm),
            )
        }
        _ => None,
    }
}

pub fn map_buffer_components(log: &Log, tx: &TransactionTrace) -> Option<ProtocolComponent> {
    LiquidityAddedToBuffer::match_and_decode(log).map(
        |LiquidityAddedToBuffer { wrapped_token,.. }| {
            let underlying_token = tx
                .logs_with_calls()
                .find_map(|(l, _)| {
                    ERC20Transfer::match_and_decode(l).and_then(|transfer| {
                        if transfer.to == VAULT_ADDRESS {
                            Some(l.address.to_owned())
                        } else {
                            None
                        }
                    })
                })
                .expect("There should always be a transfer to the vault for both wrapped and underlying token");

            log::info!("mapping buffer components: {:?}", wrapped_token);

            ProtocolComponent::new(
                &format!("0x{}", hex::encode(wrapped_token.to_owned())),
                &(tx.into()),
            )
            .with_tokens(&[wrapped_token.as_slice(), underlying_token.as_slice()])
            .with_contracts(&[VAULT_ADDRESS])
            .with_attributes(&[("pool_type", "Buffer".as_bytes())])
            .as_swap_type("balancer_v3_pool", ImplementationType::Vm)
        },
    )
}
