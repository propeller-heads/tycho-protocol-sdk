use crate::{
    abi::pool::events::Swap,
    pb::tycho::evm::aerodrome::Pool,
    storage::{constants::TRACKED_SLOTS, pool_storage::SlipstreamsPoolStorage},
};
use substreams::scalar::BigInt;
use substreams_ethereum::pb::eth::v2::StorageChange;
use substreams_helper::{hex::Hexable, storage_change::StorageChangesFilter};
use tycho_substreams::{models::Transaction, prelude::Attribute};

use super::{BalanceDelta, EventTrait};

impl EventTrait for Swap {
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

    fn get_balance_delta(&self, tx: &Transaction, pool: &Pool, ordinal: u64) -> Vec<BalanceDelta> {
        let create_balance_delta = |token_address: Vec<u8>, amount: BigInt| -> BalanceDelta {
            BalanceDelta {
                ord: ordinal,
                tx: Some(tx.clone()),
                token: token_address,
                delta: amount.to_signed_bytes_be(),
                component_id: pool
                    .address
                    .clone()
                    .to_hex()
                    .as_bytes()
                    .to_vec(),
            }
        };

        vec![
            create_balance_delta(pool.token0.clone(), self.amount0.clone()),
            create_balance_delta(pool.token1.clone(), self.amount1.clone()),
        ]
    }
}
