use ethabi::ethereum_types::Address;
use substreams_ethereum::pb::eth::v2::StorageChange;
use tycho_substreams::models::{Attribute};
use crate::abi::pool::events::PoolAddLiquidity;
use crate::events::EventTrait;
use crate::pb::maverick::v2::{BalanceDelta, Pool};

impl EventTrait for PoolAddLiquidity{
    fn get_changed_attributes(&self, storage_changes: &[StorageChange], pool_address: &[u8; 20]) -> Vec<Attribute> {
        vec![]
    }

    fn get_balance_delta(&self, pool: &Pool, ordinal: u64) -> Vec<BalanceDelta> {
        let changed_balance = vec![
            BalanceDelta{
                token_address: pool.token_a.clone(),
                amount: self.token_a_amount.clone().to_bytes_le().1,
                sign: true,
                pool_address: pool.address.clone(),
                ordinal,
            },
            BalanceDelta{
                token_address: pool.token_b.clone(),
                amount: self.token_b_amount.clone().to_bytes_le().1,
                sign: true,
                pool_address: pool.address.clone(),
                ordinal,
            }
        ];
        changed_balance
    }
}