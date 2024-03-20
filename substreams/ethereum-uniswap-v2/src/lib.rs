use substreams::handlers::map;
use substreams_ethereum::pb::eth::v2::Block;
use tycho_substreams::prelude::*;
use substreams::prelude::*;

mod pb;
mod pair_factory;
mod modules;

#[map]
pub fn map_changes(block: Block) -> Result<BlockContractChanges, substreams::errors::Error> {
    modules::map_changes(block)
}

#[substreams::handlers::store]
pub fn store_pools(
    output: tycho::BlockContractChanges,
    pool_store: substreams::store::StoreSetProto<ProtocolComponent>,
) {
    modules::store_pools(output, pool_store)
}