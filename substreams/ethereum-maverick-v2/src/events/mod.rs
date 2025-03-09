use crate::{
    abi::pool::events::{PoolAddLiquidity, PoolRemoveLiquidity, PoolSetVariableFee, PoolSwap},
    pb::maverick::v2::Pool,
};
use substreams_ethereum::{pb::eth::v2::Log, Event};
use tycho_substreams::prelude::*;

pub mod add_liquidity;
pub mod remove_liquidity;
pub mod swap;

pub trait EventTrait {
    fn get_balance_delta(&self, tx: &Transaction, pool: &Pool, ordinal: u64) -> Vec<BalanceDelta>;
}

pub enum EventType {
    PoolSwap(PoolSwap),
    AddLiquidity(PoolAddLiquidity),
    RemoveLiquidity(PoolRemoveLiquidity),
}

impl EventType {
    fn as_event_trait(&self) -> &dyn EventTrait {
        match self {
            EventType::PoolSwap(event) => event,
            EventType::AddLiquidity(event) => event,
            EventType::RemoveLiquidity(event) => event,
        }
    }
}

pub fn decode_event(event: &Log) -> Option<EventType> {
    [
        PoolSwap::match_and_decode(event).map(EventType::PoolSwap),
        PoolAddLiquidity::match_and_decode(event).map(EventType::AddLiquidity),
        PoolRemoveLiquidity::match_and_decode(event).map(EventType::RemoveLiquidity),
    ]
    .into_iter()
    .find_map(std::convert::identity)
}

/// Gets the changed balances from the log.
///
/// # Arguments
///
/// * `event` - A reference to the `Log`.
/// * `pool` - Reference to the `Pool` structure.
///
/// # Returns
///
/// A vector of `BalanceDelta` that represents
pub fn get_log_changed_balances(tx: &Transaction, event: &Log, pool: &Pool) -> Vec<BalanceDelta> {
    decode_event(event)
        .map(|e| {
            e.as_event_trait()
                .get_balance_delta(tx, pool, event.ordinal)
        })
        .unwrap_or_default()
}
