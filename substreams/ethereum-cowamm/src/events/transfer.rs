use crate::{
    abi::b_cow_pool::events::Transfer, events::BalanceEventTrait, pb::cowamm::CowPool,
};
use substreams_ethereum::{pb::eth::v2::Log, Event};
use substreams_helper::hex::Hexable;
use tycho_substreams::prelude::*;

impl BalanceEventTrait for Transfer {
    fn get_balance_delta(&self, tx: &Transaction, pool: &CowPool, event: &Log) -> Vec<BalanceDelta> {
        let mut changed_balances: Vec<BalanceDelta> = vec![];
        if self.to != pool.address {
            return changed_balances;
        }
        //joining a pool, lp_tokens are transferred from the pool to the user, so thats a negative delta 
        if event.address == pool.address && tx.from == pool.address {
            changed_balances.push(BalanceDelta {
                ord: event.ordinal,
                tx: Some(tx.clone()),
                token: pool.address.clone(),
                delta: self.value
                        .neg()
                        // .clone()
                        .to_signed_bytes_be(),
                component_id: pool
                    .address
                    .clone()
                    .to_hex()
                    .as_bytes()
                    .to_vec(),
            })
        }

        //Track any lp_tokens transferred from the user to the pool, not redemption, just to be sure so thats a positive delta
        // now the thing is that for each "changed balance array" the lenght is 1 for each instance, so its not like the thing is getting added twice, its happening in another txn 
        // buts its not its the same txn 
        // oh when token are redeemed they are not sent to the pool, they are burnt 

        else if event.address == pool.address && tx.to == pool.address && tx.from.to_hex() != "0x0000000000000000000000000000000000000000" {
            substreams::log::info!("pool to address {} 2", tx.to.to_hex());
            substreams::log::info!("pool from address {} 2", tx.from.to_hex());
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
