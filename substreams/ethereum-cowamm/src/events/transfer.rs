use crate::{
    abi::b_cow_pool::events::Transfer, events::BalanceEventTrait, pb::cowamm::CowPool,
};
use substreams_ethereum::{pb::eth::v2::Log, Event};
use substreams_helper::hex::Hexable;
use tycho_substreams::prelude::*;

impl BalanceEventTrait for Transfer {
    fn get_balance_delta(&self, tx: &Transaction, pool: &CowPool, event: &Log) -> Vec<BalanceDelta> {
        let mut changed_balances: Vec<BalanceDelta> = vec![];
        //Exiting a pool, lp_tokens are transferred from the pool to the user, so thats a negative delta 
        if event.address == pool.address && tx.from == pool.address {
            changed_balances.push(BalanceDelta {
                ord: event.ordinal,
                tx: Some(tx.clone()),
                token: pool.address.clone(),
                delta: self.value
                        .neg()
                        .clone()
                        .to_signed_bytes_be(),
                component_id: pool
                    .address
                    .clone()
                    .to_hex()
                    .as_bytes()
                    .to_vec(),
            })
        }

        //Exiting a pool, lp_tokens are transferred from the user to the pool, so thats a positive delta
        if event.address == pool.address && tx.to == pool.address {
            changed_balances.push(BalanceDelta {
                ord:event.ordinal,
                tx: Some(tx.clone()),
                token: pool.address.clone(),
                delta: self.value
                        .clone()
                        .to_signed_bytes_be(),
                component_id: pool
                    .address
                    .clone()
                    .to_hex()
                    .as_bytes()
                    .to_vec(),
            })
        }
        changed_balances
    }
}
