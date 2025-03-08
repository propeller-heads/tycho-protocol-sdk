use substreams_ethereum::pb::eth::v2::StorageChange;
use tycho_substreams::models::Attribute;
use crate::abi::pool::events::PoolSetVariableFee;
use crate::events::EventTrait;
use crate::pb::maverick::v2::{BalanceDelta, Pool};

impl EventTrait for PoolSetVariableFee{
    fn get_changed_attributes(&self, storage_changes: &[StorageChange], pool_address: &[u8; 20]) -> Vec<Attribute> {
        vec![]
    }

    fn get_balance_delta(&self, pool: &Pool, ordinal: u64) -> Vec<BalanceDelta> {
        vec![]
    }
}