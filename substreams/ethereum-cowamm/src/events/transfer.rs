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

        // when a user redeems tokens, they send the tokens to the null address, effectively burning them and thats a negative delta
        // https://etherscan.io/tx/0xc139e807a155b0ca1fdd5e870350fd623801671289982ab6300007cc7556c5a8#eventlog#250
        if event.address == pool.address && event.topics.get(1).unwrap()[12..] == NULL_ADDRESS {
            changed_balances.push(BalanceDelta {
                ord: event.ordinal,
                tx: Some(tx.clone()),
                token: pool.address.clone(),
                delta: self
                    .value
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
        //joining a pool, lp_tokens are minted to the pool, from the null address, and then transferred to the user so thats a positive delta
        //https://etherscan.io/tx/0x8cf1aa1902994eeaa59b886c848af57d89fec7170c66ef68a541fbc5759e5077#eventlog#387
        else if event.address == pool.address
            && event.topics.get(2).unwrap()[12..] == NULL_ADDRESS
        {
            changed_balances.push(BalanceDelta {
                ord: event.ordinal,
                tx: Some(tx.clone()),
                token: pool.address.clone(),
                delta: self.value.to_signed_bytes_be(),
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
