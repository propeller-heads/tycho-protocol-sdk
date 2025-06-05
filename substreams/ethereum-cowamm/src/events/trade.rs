use crate::{
    events::Trade,
    events::BalanceEventTrait, pb::cowamm::CowPool,
};
use substreams_helper::hex::Hexable;
use tycho_substreams::prelude::*;

impl BalanceEventTrait for Trade {
    fn get_balance_delta(&self, tx: &Transaction, pool: &CowPool, ordinal: u64) -> Vec<BalanceDelta> {
        let mut changed_balance = Vec::new();

        let sell_token = &self.sell_token;
        let buy_token = &self.buy_token;

        let is_sell_token_in_pool = sell_token == &pool.token_a || sell_token == &pool.token_b;
        let is_buy_token_in_pool = buy_token == &pool.token_a || buy_token == &pool.token_b;

        if is_sell_token_in_pool {
            changed_balance.push(BalanceDelta {
                ord: ordinal,
                tx: Some(tx.clone()),
                token: sell_token.clone(),
                delta: self.sell_amount.clone().to_signed_bytes_be(),
                component_id: pool
                    .address
                    .clone()
                    .to_hex()
                    .as_bytes()
                    .to_vec(),
            });
        }

        if is_buy_token_in_pool {
            changed_balance.push(BalanceDelta {
                ord: ordinal,
                tx: Some(tx.clone()),
                token: buy_token.clone(),
                delta: self.buy_amount.clone().neg().to_signed_bytes_be(),
                component_id:pool
                    .address
                    .clone()
                    .to_hex()
                    .as_bytes()
                    .to_vec(),
            });
        }

        changed_balance
    }
}


