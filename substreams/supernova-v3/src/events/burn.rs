use substreams_ethereum::pb::eth::v2::StorageChange;
use tycho_substreams::prelude::Attribute;

use crate::{
    abi::algebrapool::events::Burn,
    storage::{constants::TRACKED_SLOTS, pool_storage::UniswapPoolStorage},
};

use super::EventTrait;

impl EventTrait for Burn {
    fn get_changed_attributes(
        &self,
        storage_changes: &[StorageChange],
        _pool_address: &[u8; 20],
    ) -> Vec<Attribute> {
        let pool_storage = UniswapPoolStorage::new(storage_changes);
        let mut attrs = pool_storage.get_changed_attributes(TRACKED_SLOTS.iter().collect());
        // Also extract decoded liquidityDelta for the two ticks this Burn touches.
        attrs.extend(pool_storage.get_ticks_changes(vec![&self.bottom_tick, &self.top_tick]));
        attrs
    }
}
