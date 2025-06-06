use crate::{
    abi::b_cow_pool::events::{LogJoin, LogExit},
    abi::gpv2_settlement::events::{Trade},  
    pb::cowamm::CowPool,
};
use substreams_ethereum::{pb::eth::v2::Log, Event};
use tycho_substreams::prelude::*;

pub mod join_pool;
pub mod exit_pool;
pub mod trade;
/// A trait for extracting changed balance from an event.
pub trait BalanceEventTrait {
     /// Get all balance deltas from the event.
    ///
    /// # Arguments
    ///
    /// * `tx` - Reference to the `Transaction`.
    /// * `pool` - Reference to the `Pool`.
    /// * `ordinal` - The ordinal number of the event. This is used by the balance store to sort the
    ///
    /// # Returns
    ///
    /// A vector of `BalanceDelta` that represents the balance deltas.
    fn get_balance_delta(&self, tx: &Transaction, pool: &CowPool, ordinal:u64) -> Vec<BalanceDelta>;
}

/// Represent every events of a Maverick pool.
pub enum EventType {
    JoinPool(LogJoin),
    ExitPool(LogExit),
    Trade(Trade),
}

impl EventType {
    fn as_event_trait(&self) -> &dyn BalanceEventTrait {
        match self {
            EventType::JoinPool(event) => event,
            EventType::ExitPool(event) => event,
            EventType::Trade(event) => event,  
        }
    }
}

/// Decodes the event from the log.
///
/// # Arguments
///
/// * `event` - A reference to the `Log`.
///
/// # Returns
///
/// An `Option` that contains the `EventType` if the event is recognized.
pub fn decode_event(event: &Log) -> Option<EventType> {
    [
        LogJoin::match_and_decode(event).map(EventType::JoinPool),
        LogExit::match_and_decode(event).map(EventType::ExitPool),
        Trade::match_and_decode(event).map(EventType::Trade),
    ]
    .into_iter()
    .find_map(std::convert::identity)
}

/// Gets the changed balances from the log.
///
/// # Arguments
///
/// * `tx` - Reference to the `Transaction`.
/// * `event` - Reference to the `Log`.
/// * `pool` - Reference to the `CowPool`.
///
/// # Returns
///
/// A vector of `BalanceDelta` that represents
pub fn get_log_changed_balances(tx: &Transaction, event: &Log, pool: &CowPool) -> Vec<BalanceDelta> {
    decode_event(event)
        .map(|e| {
            e.as_event_trait()
                .get_balance_delta(tx, pool, event.ordinal)
        })
        .unwrap_or_default()
}