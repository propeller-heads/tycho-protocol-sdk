use crate::abi;
use substreams_database_change::change::AsString;
use substreams_ethereum::{pb::eth::v2::Log, Event};
// Add missing dependency to Cargo.toml
// tycho_substreams = "0.1.0"
use tycho_substreams::prelude::*;

#[allow(unused_imports)]
use num_traits::cast::ToPrimitive;

pub fn address_map(
    tracked_factory_address: &[u8],
    pool_factory_address: &[u8],
    log: &Log,
    tx: &Transaction,
) -> Option<ProtocolComponent> {
    if *pool_factory_address == *tracked_factory_address {
        abi::elasticfactory_contract::events::PoolCreated::match_and_decode(log).map(
            |pool_created| {
                ProtocolComponent::at_contract(&pool_created.pool, tx)
                    .with_tokens(&[pool_created.token0, pool_created.token1])
                    .with_attributes(&[("Tick", pool_created.tick_distance.as_string())])
                    .as_swap_type("kyberswap_elastic_pool", ImplementationType::Vm)
            },
        )
    } else {
        None
    }
}
