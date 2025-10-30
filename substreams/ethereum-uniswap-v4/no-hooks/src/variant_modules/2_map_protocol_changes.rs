use substreams::pb::substreams::StoreDeltas;
use substreams_ethereum::pb::eth::v2::{self as eth};
use tycho_substreams::prelude::*;

use ethereum_uniswap_v4_shared::utils::protocol_changes::collect_transaction_changes;

use crate::pb::uniswap::v4::{Events, LiquidityChanges, TickDeltas};

#[substreams::handlers::map]
pub fn map_protocol_changes(
    block: eth::Block,
    created_pools: BlockEntityChanges,
    events: Events,
    balances_map_deltas: BlockBalanceDeltas,
    balances_store_deltas: StoreDeltas,
    ticks_map_deltas: TickDeltas,
    ticks_store_deltas: StoreDeltas,
    pool_liquidity_changes: LiquidityChanges,
    pool_liquidity_store_deltas: StoreDeltas,
) -> Result<BlockChanges, substreams::errors::Error> {
    let changes = collect_transaction_changes(
        created_pools,
        events,
        balances_map_deltas,
        balances_store_deltas,
        ticks_map_deltas,
        ticks_store_deltas,
        pool_liquidity_changes,
        pool_liquidity_store_deltas,
    );

    Ok(BlockChanges { block: Some((&block).into()), changes, storage_changes: vec![] })
}
