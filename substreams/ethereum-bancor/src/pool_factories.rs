use crate::{abi, consts};
use substreams_ethereum::{
    pb::eth::v2::{Call, Log},
    Event,
};
// Add missing dependency to Cargo.toml
// tycho_substreams = "0.1.0"
use tycho_substreams::prelude::*;

#[allow(unused_imports)]

pub fn address_map(
    pool_factory_address: &[u8],
    log: &Log,
    _call: &Call,
    tx: &Transaction,
) -> Option<ProtocolComponent> {
    let mut found = false;

    for i in 0..consts::FACTORIES.len() {
        if !consts::FACTORIES[i].is_empty() &&
            consts::FACTORIES[i] == pool_factory_address &&
            pool_factory_address == _call.address.as_slice()
        {
            found = true;
            break;
        }
    }

    if found {
        let pool_created = abi::pool_contract::events::PoolAdded::match_and_decode(log).unwrap();

        Some(
            ProtocolComponent::at_contract(&pool_created.pool, tx)
                .as_swap_type("bancor_pool", ImplementationType::Vm),
        )
    } else {
        None
    }
}
