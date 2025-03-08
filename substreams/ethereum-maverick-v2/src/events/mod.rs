use substreams_ethereum::Event;
use substreams_ethereum::pb::eth::v2::{Log, StorageChange};
use tycho_substreams::prelude::*;
use crate::abi::pool::events::{
    PoolSwap,PoolAddLiquidity,PoolRemoveLiquidity,PoolSetVariableFee
};
use crate::pb::maverick::v2::{Pool,BalanceDelta};

pub mod swap;
pub mod add_liquidity;
pub mod remove_liquidity;
pub mod set_variable_fee;

pub trait EventTrait {
    fn get_changed_attributes(
        &self,
        storage_changes: &[StorageChange],
        pool_address: &[u8; 20],
    ) -> Vec<Attribute>;
    fn get_balance_delta(&self, pool: &Pool, ordinal: u64) -> Vec<BalanceDelta>;
}

pub enum EventType {
    PoolSwap(PoolSwap),
    AddLiquidity(PoolAddLiquidity),
    RemoveLiquidity(PoolRemoveLiquidity),
    SetVariableFee(PoolSetVariableFee),
}

impl EventType {
    fn as_event_trait(&self) -> &dyn EventTrait {
        match self {
            EventType::PoolSwap(event) => event,
            EventType::AddLiquidity(event) => event,
            EventType::RemoveLiquidity(event) => event,
            EventType::SetVariableFee(event) => event,
        }
    }
}

pub fn decode_event(event: &Log) -> Option<EventType> {
    [
        PoolSwap::match_and_decode(event).map(EventType::PoolSwap),
        PoolAddLiquidity::match_and_decode(event).map(EventType::AddLiquidity),
        PoolRemoveLiquidity::match_and_decode(event).map(EventType::RemoveLiquidity),
        PoolSetVariableFee::match_and_decode(event).map(EventType::SetVariableFee),
    ]
        .into_iter()
        .find_map(std::convert::identity)
}

/// Gets the changed attributes from the log.
///
/// # Arguments
///
/// * `event` - A reference to the `Log`.
/// * `storage_changes` - A slice of `StorageChange` that indicates the changes in storage.
/// * `pool` - Reference to the `Pool` structure.
///
/// # Returns
///
/// A vector of `Attribute` that represents the changed attributes.
pub fn get_log_changed_attributes(
    event: &Log,
    storage_changes: &[StorageChange],
    pool_address: &[u8; 20],
) -> Vec<Attribute> {
    decode_event(event)
        .map(|e| {
            e.as_event_trait()
                .get_changed_attributes(storage_changes, pool_address)
        })
        .unwrap_or_default()
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
pub fn get_log_changed_balances(event: &Log, pool: &Pool) -> Vec<BalanceDelta> {
    decode_event(event)
        .map(|e| {
            e.as_event_trait()
                .get_balance_delta(pool, event.ordinal)
        })
        .unwrap_or_default()
}
