use anyhow::{Ok, Result};
use tycho_substreams::prelude::*;
use substreams_ethereum::pb::eth::v2::Block;
use substreams::store::{StoreGet, StoreGetString};

#[substreams::handlers::map]
pub fn map_relative_balances(
    _block: Block,
    _store: StoreGetString,
) -> Result<BlockBalanceDeltas, anyhow::Error> {
    
    Ok(BlockBalanceDeltas {
        balance_deltas: vec![],
    })
}