use crate::{abi::pool::events::PoolSwap, events::EventTrait, pb::maverick::v2::Pool};
use tycho_substreams::prelude::*;

impl EventTrait for PoolSwap {
    fn get_balance_delta(&self, tx: &Transaction, pool: &Pool, ordinal: u64) -> Vec<BalanceDelta> {
        let (token_in, token_out, amount_in, amount_out) = if self.params.1 {
            (&pool.token_a, &pool.token_b, &self.amount_in, &self.amount_out)
        } else {
            (&pool.token_b, &pool.token_a, &self.amount_out, &self.amount_in)
        };

        vec![
            BalanceDelta {
                ord: ordinal,
                tx: Some(tx.clone()),
                token: token_in.clone(),
                delta: amount_in.clone().to_bytes_le().1,
                component_id: pool.address.clone(),
            },
            BalanceDelta {
                ord: ordinal,
                tx: Some(tx.clone()),
                token: token_out.clone(),
                delta: amount_out.clone().to_bytes_le().1,
                component_id: pool.address.clone(),
            },
        ]
    }
}
