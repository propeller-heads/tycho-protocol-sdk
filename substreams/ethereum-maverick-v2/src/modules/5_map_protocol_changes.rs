use anyhow::Result;
use substreams::{
    pb::substreams::StoreDeltas,
    store::{StoreGet, StoreGetString},
};
use substreams_ethereum::pb::eth::v2::Block;
use tycho_substreams::prelude::*;


#[substreams::handlers::map]
pub fn map_protocol_changes(
    block: Block,
    _protocol_components: BlockTransactionProtocolComponents,
    _deltas: BlockBalanceDeltas,
    _components_store: StoreGetString,
    _balance_store: StoreDeltas, // Note, this map module is using the `deltas` mode for the store.
) -> Result<BlockChanges> {
    Ok(BlockChanges {
        block: Some((&block).into()),
        changes: vec![],
    })
}