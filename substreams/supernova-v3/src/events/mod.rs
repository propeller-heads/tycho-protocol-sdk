use substreams_ethereum::{
    pb::eth::v2::{Log, StorageChange},
    Event,
};
use tycho_substreams::prelude::Attribute;

pub mod burn;
pub mod mint;
pub mod swap;
pub mod initialize;

use crate::abi::algebrapool::events::{Burn, Mint, Swap, Initialize};

/// A trait for extracting changed attributes from an event based on storage changes.
pub trait EventTrait {
    fn get_changed_attributes(
        &self,
        storage_changes: &[StorageChange],
        pool_address: &[u8; 20],
    ) -> Vec<Attribute>;
}

pub enum EventType {
    Initialize(Initialize),
    Swap(Swap),
    Mint(Mint),
    Burn(Burn),
}

impl EventType {
    fn as_event_trait(&self) -> &dyn EventTrait {
        match self {
            EventType::Initialize(e) => e,
            EventType::Swap(e) => e,
            EventType::Mint(e) => e,
            EventType::Burn(e) => e,
        }
    }
}

pub fn decode_event(log: &Log) -> Option<EventType> {
    if let Some(e) = Initialize::match_and_decode(log) {
        return Some(EventType::Initialize(e));
    }
    if let Some(e) = Swap::match_and_decode(log) {
        return Some(EventType::Swap(e));
    }
    if let Some(e) = Mint::match_and_decode(log) {
        return Some(EventType::Mint(e));
    }
    if let Some(e) = Burn::match_and_decode(log) {
        return Some(EventType::Burn(e));
    }
    None
}

/// Gets the changed attributes from the log based on storage changes.
pub fn get_log_changed_attributes(
    log: &Log,
    storage_changes: &[StorageChange],
    pool_address: &[u8; 20],
) -> Vec<Attribute> {
    decode_event(log)
        .map(|e| {
            e.as_event_trait()
                .get_changed_attributes(storage_changes, pool_address)
        })
        .unwrap_or_default()
}
