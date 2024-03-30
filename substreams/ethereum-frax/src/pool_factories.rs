use crate::abi;
use substreams::{hex, scalar::BigInt};
use substreams_ethereum::{
    pb::eth::v2::{Call, Log},
    Event,
};
// Add missing dependency to Cargo.toml
// tycho_substreams = "0.1.0"
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

#[allow(unused_imports)]
use num_traits::cast::ToPrimitive;

const FACTORY_TRACKED_CONTRACT: [u8; 20] = hex!("43ec799eadd63848443e2347c49f5f52e8fe0f6f");

/// This is the main function that handles the creation of `ProtocolComponent`s generated by the
/// factory address Frax Swap has only one factory contract deployed, ref: https://docs.frax.finance/smart-contracts/fraxswap
pub fn address_map(
    pool_factory_address: &[u8],
    log: &Log,
    _call: &Call,
    tx: &Transaction,
) -> Option<ProtocolComponent> {
    let tracked_factory_address = FACTORY_TRACKED_CONTRACT.to_vec();
    if *pool_factory_address == tracked_factory_address {
        let pool_created =
            abi::factory_contract::events::PairCreated::match_and_decode(log).unwrap();

        Some(
            ProtocolComponent::at_contract(&pool_created.pair, tx)
                .with_tokens(&[pool_created.token0, pool_created.token1])
                // .with_attributes(&[("placeholder", "".as_bytes())]) // @todo: identify attributes
                .as_swap_type("frax_pool", ImplementationType::Vm),
        )
    } else {
        None
    }
}
