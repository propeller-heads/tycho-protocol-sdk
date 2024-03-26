use crate::abi;
use substreams::{hex, scalar::BigInt};
use substreams_ethereum::{
    pb::eth::v2::{Call, Log},
    Event, Function,
};
use tycho_substreams::prelude::*;

/// This trait defines some helpers for serializing and deserializing `Vec<BigInt` which is needed
///  to be able to encode the `normalized_weights` and `weights` `Attribute`s. This should also be
///  handled by any downstream application.
trait SerializableVecBigInt {
    fn serialize_bytes(&self) -> Vec<u8>;
    #[allow(dead_code)]
    fn deserialize_bytes(bytes: &[u8]) -> Vec<BigInt>;
}

impl SerializableVecBigInt for Vec<BigInt> {
    fn serialize_bytes(&self) -> Vec<u8> {
        self.iter()
            .flat_map(|big_int| big_int.to_signed_bytes_be())
            .collect()
    }
    fn deserialize_bytes(bytes: &[u8]) -> Vec<BigInt> {
        bytes
            .chunks_exact(32)
            .map(BigInt::from_signed_bytes_be)
            .collect::<Vec<BigInt>>()
    }
}

/// This is the main function that handles the creation of `ProtocolComponent`s with `Attribute`s
///  based on the specific factory address. There's 3 factory groups that are represented here:
///  - Weighted Pool Factories
///  - Linear Pool Factories
///  - Stable Pool Factories
/// (Balancer does have a bit more (esp. in the deprecated section) that could be implemented as
///  desired.)
/// We use the specific ABIs to decode both the log event and corresponding call to gather
///  `PoolCreated` event information alongside the `Create` call data that provide us details to
///  fulfill both the required details + any extra `Attributes`
/// Ref: https://docs.balancer.fi/reference/contracts/deployment-addresses/mainnet.html
pub fn address_map(
    pool_factory_address: &[u8],
    log: &Log,
    call: &Call,
    tx: &Transaction,
) -> Option<ProtocolComponent> {
    match *pool_factory_address {
        hex!("43eC799eAdd63848443E2347C49f5f52e8Fe0F6f") => {
            let create_call =
                abi::weighted_pool_factory::functions::Create::match_and_decode(call)?;
            let pool_created =
                abi::weighted_pool_factory::events::PoolCreated::match_and_decode(log)?;

            Some(
                ProtocolComponent::at_contract(&pool_created.pool, tx)
                    .with_tokens(&create_call.tokens)
                    .as_swap_type("fraxswap_pair", ImplementationType::Vm),
            )
        }
        _ => None,
    }
}
