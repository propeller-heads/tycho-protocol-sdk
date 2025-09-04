use crate::{
    abi,
    modules::VAULT_ADDRESS,
    utils::{json_serialize_mapping_tokens, MappingToken, Params},
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
    pb::eth::v2::{Call, Log},
    Event, Function,
};
use tycho_substreams::{
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

/// Collects the `MappingToken` instances corresponding to a list of tokens.
///
/// For each token in `tokens`, this function looks up its mapping in `params`.
/// If a mapping is found, it is included; otherwise, a default `MappingToken` is used.
///
/// # Arguments
/// * `tokens` - A reference to a list of tokens to map.
/// * `params` - The `Params` instance used to look up token mappings.
///
/// # Returns
/// A `Vec<MappingToken>` containing the mapped tokens for each input token.
pub fn collect_mapping_tokens(tokens: &TokenConfig, params: &Params) -> Vec<MappingToken> {
    tokens
        .iter()
        .map(|token| {
            params
                .get_mapping_token(&token.0)
                .unwrap_or_default()
        })
        .collect()
}

pub fn address_map(
    pool_factory_address: &[u8],
    params: &Params,
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
            let mapping_tokens = collect_mapping_tokens(&token_config, params);

            // We only include the tokens listed in token_config in the pool's token list.
            // Mapping tokens (underlying/unwrapped tokens) are NOT added to the pool tokens
            // as their balances are managed globally in the vault contract, not per-pool.
            // All mapping tokens are stored separately in mapping_tokens_bytes.
            let tokens: Vec<_> = token_config
                .into_iter()
                .map(|t| t.0)
                .chain(
                    mapping_tokens
                        .iter()
                        .flat_map(|m| m.addresses.iter().cloned()),
                )
                .collect();

            let normalized_weights_bytes =
                json_serialize_bigint_list(normalized_weights.as_slice());
            let fee_bytes = swap_fee_percentage.to_signed_bytes_be();
            let rate_providers_bytes = json_serialize_address_list(rate_providers.as_slice());
            let mapping_tokens_bytes = json_serialize_mapping_tokens(mapping_tokens.as_slice());

            let mut attributes = vec![
                ("pool_type", "WeightedPoolFactory".as_bytes()),
                ("normalized_weights", &normalized_weights_bytes),
                ("fee", &fee_bytes),
                ("manual_updates", &[1u8]),
            ];

            if !rate_providers.is_empty() {
                attributes.push(("rate_providers", &rate_providers_bytes));
            }

            if !mapping_tokens.is_empty() {
                attributes.push(("mapping_tokens", &mapping_tokens_bytes));
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
            let mapping_tokens = collect_mapping_tokens(&token_config, params);
            // We only include the tokens listed in token_config in the pool's token list.
            // Mapping tokens (underlying/unwrapped tokens) are NOT added to the pool tokens
            // as their balances are managed globally in the vault contract, not per-pool.
            // All mapping tokens are stored separately in mapping_tokens_bytes.
            let tokens = token_config
                .into_iter()
                .map(|t| t.0)
                .collect::<Vec<_>>();

            let fee_bytes = swap_fee_percentage.to_signed_bytes_be();
            let rate_providers_bytes = json_serialize_address_list(rate_providers.as_slice());
            let mapping_tokens_bytes = json_serialize_mapping_tokens(mapping_tokens.as_slice());
            let mut attributes = vec![
                ("pool_type", "StablePoolFactory".as_bytes()),
                ("bpt", &pool),
                ("fee", &fee_bytes),
                ("manual_updates", &[1u8]),
            ];

            if !rate_providers.is_empty() {
                attributes.push(("rate_providers", &rate_providers_bytes));
            }

            if !mapping_tokens_bytes.is_empty() {
                attributes.push(("mapping_tokens", &mapping_tokens_bytes));
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
