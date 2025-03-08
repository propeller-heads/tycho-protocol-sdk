use anyhow::Result;
use substreams::{
    store::{StoreGet},
};
use substreams::prelude::{StoreGetBigInt, StoreGetProto};
use substreams_ethereum::pb::eth::v2::Block;
use tycho_substreams::prelude::*;
use crate::pb::maverick::v2::{BalanceDeltas, Pool};

#[substreams::handlers::map]
pub fn map_protocol_changes(
    block: Block,
    _protocol_components: BlockTransactionProtocolComponents,
    _balance_deltas: BalanceDeltas,
    _pool_store: StoreGetProto<Pool>,
    _balance_store: StoreGetBigInt, // Note, this map module is using the `deltas` mode for the store.
) -> Result<BlockChanges> {
    Ok(BlockChanges {
        block: Some((&block).into()),
        changes: vec![],
    })
}