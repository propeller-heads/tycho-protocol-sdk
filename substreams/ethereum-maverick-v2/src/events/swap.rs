use substreams_ethereum::pb::eth::v2::StorageChange;
use tycho_substreams::prelude::Attribute;
use crate::abi::pool::events::PoolSwap;
use crate::events::EventTrait;
use crate::pb::maverick::v2::{BalanceDelta, Pool};

impl EventTrait for PoolSwap{
    fn get_changed_attributes(&self, storage_changes: &[StorageChange], pool_address: &[u8; 20]) -> Vec<Attribute> {
        vec![]
    }

    fn get_balance_delta(&self, pool: &Pool, ordinal: u64) -> Vec<BalanceDelta> {
        let (token_in, token_out, amount_in, amount_out) = if self.params.1 {
            (&pool.token_a, &pool.token_b, &self.amount_in, &self.amount_out)
        } else {
            (&pool.token_b, &pool.token_a, &self.amount_out, &self.amount_in)
        };

        vec![
            BalanceDelta {
                token_address: token_in.clone(),
                amount: amount_in.clone().to_bytes_le().1,
                sign: false,
                pool_address: pool.address.clone(),
                ordinal,
            },
            BalanceDelta {
                token_address: token_out.clone(),
                amount: amount_out.clone().to_bytes_le().1,
                sign: true,
                pool_address: pool.address.clone(),
                ordinal,
            },
        ]
    }
}