use crate::{
    abi::pool::events::Burn,
    pb::tycho::evm::aerodrome::Pool,
    storage::{constants::TRACKED_SLOTS, pool_storage::SlipstreamsPoolStorage},
};
use substreams::prelude::BigInt;
use substreams_ethereum::pb::eth::v2::StorageChange;
use substreams_helper::storage_change::StorageChangesFilter;
use tycho_substreams::models::Transaction;

use super::{BalanceDelta, EventTrait};
use tycho_substreams::prelude::Attribute;

impl EventTrait for Burn {
    fn get_changed_attributes(
        &self,
        storage_changes: &[StorageChange],
        pool_address: &[u8; 20],
    ) -> Vec<Attribute> {
        let storage_vec = storage_changes.to_vec();

        let filtered_storage_changes = storage_vec
            .filter_by_address(pool_address)
            .into_iter()
            .cloned()
            .collect();

        let pool_storage = SlipstreamsPoolStorage::new(&filtered_storage_changes);

        let mut changed_attributes =
            pool_storage.get_changed_attributes(TRACKED_SLOTS.to_vec().iter().collect());

        let changed_ticks =
            pool_storage.get_ticks_changes(vec![&self.tick_upper, &self.tick_lower]);

        changed_attributes.extend(changed_ticks);

        let changed_observation_index = changed_attributes
            .iter()
            .find(|attr| attr.name == "observationIndex")
            .map(|attr| attr.value.clone());

        if let Some(observation_index) = changed_observation_index {
            let observation_index = BigInt::from_signed_bytes_be(observation_index.as_slice());
            let changed_observation =
                pool_storage.get_observations_changes(vec![&observation_index]);
            changed_attributes.extend(changed_observation);
        }

        changed_attributes
    }

    fn get_balance_delta(
        &self,
        _tx: &Transaction,
        _pool: &Pool,
        _ordinal: u64,
    ) -> Vec<BalanceDelta> {
        // Burn event balances deltas are accounted for by the Collect event.
        // In the case of a burn, the Collect event amounts will include both the burned amount and
        // the fees earned.
        vec![]
    }
}
