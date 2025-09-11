use crate::{abi, abi::vault_admin::events::LiquidityAddedToBuffer, modules::VAULT_ADDRESS};
use abi::{
    stable_pool_factory_contract::{
        events::PoolCreated as StablePoolCreated, functions::Create as StablePoolCreate,
    },
    vault_admin::functions::InitializeBuffer,
    weighted_pool_factory_contract::{
        events::PoolCreated as WeightedPoolCreated, functions::Create as WeightedPoolCreate,
    },
};
use keccak_hash::keccak;
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

const WEIGHTED_POOL_FACTORY: [u8; 20] = hex!("201efd508c8DfE9DE1a13c2452863A78CB2a86Cc");
const STABLE_POOL_FACTORY: [u8; 20] = hex!("B9d01CA61b9C181dA1051bFDd28e1097e920AB14");

pub fn address_map(factory_address: &[u8], log: &Log, call: &Call) -> Option<ProtocolComponent> {
    if log.address == VAULT_ADDRESS {
        if let (Some(buffer), Some(_)) = (
            InitializeBuffer::match_and_decode(call),
            LiquidityAddedToBuffer::match_and_decode(log),
        ) {
            let wrapped_token = buffer.wrapped_token;
            let underlying_token = find_underlying_token(call, &wrapped_token)?;
            return Some(create_buffer_component(wrapped_token, underlying_token));
        }
    }

    if factory_address == WEIGHTED_POOL_FACTORY {
        create_weighted_pool_component(call, log)
    } else if factory_address == STABLE_POOL_FACTORY {
        create_stable_pool_component(call, log)
    } else {
        None
    }
}

fn create_buffer_component(wrapped_token: Vec<u8>, underlying_token: Vec<u8>) -> ProtocolComponent {
    let tokens_data = [&wrapped_token[..], &underlying_token[..]].concat();
    let component_id = keccak(&tokens_data).as_bytes().to_vec();

    let attributes = vec![("pool_type", "LiquidityBuffer".as_bytes()), ("manual_updates", &[1u8])];

    ProtocolComponent::new(&format!("0x{}", hex::encode(&component_id)))
        .with_contracts(&[VAULT_ADDRESS.to_vec()])
        .with_tokens(&[wrapped_token, underlying_token])
        .with_attributes(&attributes)
        .as_swap_type("balancer_v3_pool", ImplementationType::Vm)
}

fn create_weighted_pool_component(call: &Call, log: &Log) -> Option<ProtocolComponent> {
    let WeightedPoolCreate {
        tokens: token_config, normalized_weights, swap_fee_percentage, ..
    } = WeightedPoolCreate::match_and_decode(call)?;

    let WeightedPoolCreated { pool } = WeightedPoolCreated::match_and_decode(log)?;

    let rate_providers = collect_rate_providers(&token_config);
    let tokens: Vec<_> = token_config
        .into_iter()
        .map(|t| t.0)
        .collect();

    let normalized_weights_bytes = json_serialize_bigint_list(&normalized_weights);
    let fee_bytes = swap_fee_percentage.to_signed_bytes_be();
    let rate_providers_bytes = json_serialize_address_list(&rate_providers);

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
            .with_tokens(&tokens)
            .with_attributes(&attributes)
            .as_swap_type("balancer_v3_pool", ImplementationType::Vm),
    )
}

fn create_stable_pool_component(call: &Call, log: &Log) -> Option<ProtocolComponent> {
    let StablePoolCreate { tokens: token_config, swap_fee_percentage, .. } =
        StablePoolCreate::match_and_decode(call)?;

    let StablePoolCreated { pool } = StablePoolCreated::match_and_decode(log)?;

    let rate_providers = collect_rate_providers(&token_config);
    let tokens: Vec<_> = token_config
        .into_iter()
        .map(|t| t.0)
        .collect();

    let fee_bytes = swap_fee_percentage.to_signed_bytes_be();
    let rate_providers_bytes = json_serialize_address_list(&rate_providers);

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
            .with_tokens(&tokens)
            .with_attributes(&attributes)
            .as_swap_type("balancer_v3_pool", ImplementationType::Vm),
    )
}

fn find_underlying_token(call: &Call, wrapped_token: &[u8]) -> Option<Vec<u8>> {
    let buffer_asset_key = get_storage_key_for_buffer_asset(wrapped_token);
    call.storage_changes
        .iter()
        .find(|e| e.key == buffer_asset_key)
        .map(|e| e.new_value[12..32].to_vec())
}

// token_addr -> keccak256(abi.encode(token_address, 14)) as 14 is the order in which
// _bufferAssets are declared
fn get_storage_key_for_buffer_asset(token_address: &[u8]) -> Vec<u8> {
    let mut input = [0u8; 64];
    input[12..32].copy_from_slice(token_address);
    input[63] = 14u8;
    let result = keccak(input.as_slice())
        .as_bytes()
        .to_vec();
    result
}
