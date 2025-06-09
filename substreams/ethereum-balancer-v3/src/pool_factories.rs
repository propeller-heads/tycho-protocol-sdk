use crate::{
    abi::{self, vault_contract::events::LiquidityAddedToBuffer},
    modules::VAULT_ADDRESS,
};
use abi::{
    stable_pool_factory_contract::{
        events::PoolCreated as StablePoolCreated, functions::Create as StablePoolCreate,
    },
    weighted_pool_factory_contract::{
        events::PoolCreated as WeightedPoolCreated, functions::Create as WeightedPoolCreate,
    },
};
use substreams::{hex, scalar::BigInt};
use substreams_ethereum::{
    pb::eth::v2::{Call, Log, TransactionTrace},
    Event, Function,
};
use tycho_substreams::{
    abi::erc20::events::Transfer,
    attributes::{json_serialize_address_list, json_serialize_bigint_list},
    prelude::*,
};

// Token config: (token_address, rate, rate_provider_address, is_exempt_from_yield_fees)
type TokenConfig = Vec<(Vec<u8>, substreams::scalar::BigInt, Vec<u8>, bool)>;

pub fn collect_rate_providers(tokens: &TokenConfig) -> Vec<Vec<u8>> {
    tokens
        .iter()
        .filter(|token| token.1 == BigInt::from(1)) // WITH_RATE == 1
        .map(|token| token.2.clone())
        .collect::<Vec<_>>()
}

pub fn address_map(
    pool_factory_address: &[u8],
    log: &Log,
    call: &Call,
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
            let rate_providers = collect_rate_providers(&token_config);

            let tokens = token_config
                .into_iter()
                .map(|t| t.0)
                .collect::<Vec<_>>();

            let normalized_weights_bytes =
                json_serialize_bigint_list(normalized_weights.as_slice());
            let fee_bytes = swap_fee_percentage.to_signed_bytes_be();
            let rate_providers_bytes = json_serialize_address_list(rate_providers.as_slice());

            let mut attributes = vec![
                ("pool_type", "WeightedPoolFactory".as_bytes()),
                ("normalized_weights", &normalized_weights_bytes),
                ("fee", &fee_bytes),
                ("manual_updates", &[1u8]),
            ];

            if !rate_providers.is_empty() {
                attributes.push(("rate_providers", &rate_providers_bytes));
            }

            Some(
                ProtocolComponent::new(&format!("0x{}", hex::encode(&pool)))
                    .with_contracts(&[pool, VAULT_ADDRESS.to_vec()])
                    .with_tokens(tokens.as_slice())
                    .with_attributes(&attributes)
                    .as_swap_type("balancer_v3_pool", ImplementationType::Vm),
            )
        }
        hex!("B9d01CA61b9C181dA1051bFDd28e1097e920AB14") => {
            let StablePoolCreate { tokens: token_config, swap_fee_percentage, .. } =
                StablePoolCreate::match_and_decode(call)?;
            let StablePoolCreated { pool } = StablePoolCreated::match_and_decode(log)?;
            let rate_providers = collect_rate_providers(&token_config);
            let tokens = token_config
                .into_iter()
                .map(|t| t.0)
                .collect::<Vec<_>>();

            let fee_bytes = swap_fee_percentage.to_signed_bytes_be();
            let rate_providers_bytes = json_serialize_address_list(rate_providers.as_slice());

            let mut attributes = vec![
                ("pool_type", "StablePoolFactory".as_bytes()),
                ("bpt", &pool),
                ("fee", &fee_bytes),
                ("manual_updates", &[1u8]),
            ];

            if !rate_providers.is_empty() {
                attributes.push(("rate_providers", &rate_providers_bytes));
            }

            Some(
                ProtocolComponent::new(&format!("0x{}", hex::encode(&pool)))
                    .with_contracts(&[pool.to_owned(), VAULT_ADDRESS.to_vec()])
                    .with_tokens(tokens.as_slice())
                    .with_attributes(&attributes)
                    .as_swap_type("balancer_v3_pool", ImplementationType::Vm),
            )
        }
        _ => None,
    }
}

#[allow(dead_code)]
fn find_underlying_token(tx: &TransactionTrace, underlying_amount: BigInt) -> Option<Vec<u8>> {
    tx.receipt()
        .logs()
        .filter_map(|log| {
            if let Some(Transfer { to, value, .. }) = Transfer::match_and_decode(log) {
                if to == VAULT_ADDRESS && value == underlying_amount {
                    return Some(log.address().to_vec());
                }
            }
            None
        })
        .next()
}

// this adds a buffer protocol component, they are internal pool managed by balancer but they can be
// used as a vault-type component for swapping for instance USDT <-> waETHUSDT (aave wrapped USDT)
#[allow(dead_code)]
pub fn buffer_map(log: &Log, tx: &TransactionTrace) -> Option<ProtocolComponent> {
    LiquidityAddedToBuffer::match_and_decode(log).map(
        |LiquidityAddedToBuffer { wrapped_token, amount_underlying, .. }| {
            let underlying_token = find_underlying_token(tx, amount_underlying).unwrap(); // must exist
            ProtocolComponent::new(&format!("0x{}", hex::encode(&wrapped_token)))
                .with_contracts(&[wrapped_token.to_vec(), VAULT_ADDRESS.to_vec()])
                .with_tokens(&[wrapped_token.as_slice(), underlying_token.as_slice()])
                .with_attributes(&[("pool_type", "buffer".as_bytes()), ("erc4626", &[1u8])])
                .as_swap_type("balancer_v3_pool", ImplementationType::Vm)
        },
    )
}
