use crate::{abi::b_cow_pool::events::Transfer, events::BalanceEventTrait, pb::cowamm::CowPool};
use substreams_ethereum::{pb::eth::v2::Log, Event};
use substreams_helper::hex::Hexable;
use tycho_substreams::prelude::*;

impl BalanceEventTrait for Transfer {
    fn get_balance_delta(
        &self,
        tx: &Transaction,
        pool: &CowPool,
        event: &Log,
    ) -> Vec<BalanceDelta> {
        let mut changed_balances: Vec<BalanceDelta> = vec![];
        const NULL_ADDRESS: [u8; 20] = [0u8; 20];
        //what causes the lp token supply to change is either -> minting or burning
        //topics[1]: from address (padded to 32 bytes)
        //topics[2]: to address (padded to 32 bytes)
        let from =  &event.topics.get(1).unwrap()[12..];
        let to = &event.topics.get(2).unwrap()[12..];
                //joining a pool, lp_tokens are minted to the pool, from the null address, and so total lp supply increases
                if event.address == pool.address && from == NULL_ADDRESS {
                    changed_balances.push(BalanceDelta {
                        ord: event.ordinal,
                tx: Some(tx.clone()),
                token: pool.address.clone(),
                delta: self
                    .value
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
            
            // when a user redeems their lp tokens, they send the tokens to the null address, burning them and thats a negative delta
        else if event.address == pool.address
            && to == NULL_ADDRESS
        {
            changed_balances.push(BalanceDelta {
                ord: event.ordinal,
                tx: Some(tx.clone()),
                token: pool.address.clone(),
                delta: self.value.neg().to_signed_bytes_be(),
                component_id: pool
                    .address
                    .clone()
                    .to_hex()
                    .as_bytes()
                    .to_vec(),
            })
        }
        //tracking arbitrary token transfers (edge case)
        //when a user transfers token_a to the pool it should be a positive delta
        else if event.address == pool.token_a
            && to == pool.address
        {
            changed_balances.push(BalanceDelta {
                ord: event.ordinal,
                tx: Some(tx.clone()),
                token: pool.token_a.clone(),
                delta: self.value.to_signed_bytes_be(),
                component_id: pool
                    .address
                    .clone()
                    .to_hex()
                    .as_bytes()
                    .to_vec(),
            })
        }
        //when a user transfers token_b to the pool it should be a positive delta
        else if event.address == pool.token_b
            && to == pool.address
        {
            changed_balances.push(BalanceDelta {
                ord: event.ordinal,
                tx: Some(tx.clone()),
                token: pool.token_b.clone(),
                delta: self.value.to_signed_bytes_be(),
                component_id: pool
                    .address
                    .clone()
                    .to_hex()
                    .as_bytes()
                    .to_vec(),
            })
        }
        //not case for negative delta of normal token transfer because the only way to 
        //transfer tokens out of CowAMM pool is to make a Trade via cowprotocol or ExitPool,
        //which are separate events whose cases have been covered 
        changed_balances
    }
}
