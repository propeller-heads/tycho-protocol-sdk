use crate::abi;
use substreams_ethereum::{pb::eth::v2::Log, Event};
// Add missing dependency to Cargo.toml
// tycho_substreams = "0.1.0"
use tycho_substreams::prelude::*;

#[allow(unused_imports)]
use num_traits::cast::ToPrimitive;

/// This is the main function that handles the creation of `ProtocolComponent`
pub fn address_map(
    tracked_factory_address: &[u8],
    tracked_factory2_address: &[u8],
    pool_factory_address: &[u8],
    log: &Log,
    tx: &Transaction,
) -> Option<ProtocolComponent> {
    if (*pool_factory_address == *tracked_factory_address || *pool_factory_address == *tracked_factory2_address) && !(abi::factory_contract::events::PoolCreated::match_and_decode(log)).is_none() {
        let pool_created =
            abi::factory_contract::events::PoolCreated::match_and_decode(log).unwrap();

        Some(
            ProtocolComponent::at_contract(&pool_created.pool, tx)
                .with_tokens(&[pool_created.token0, pool_created.token1])
                .as_swap_type("kyberswap_classic_pool", ImplementationType::Vm),
        )
    } else {
        None
    }
}
