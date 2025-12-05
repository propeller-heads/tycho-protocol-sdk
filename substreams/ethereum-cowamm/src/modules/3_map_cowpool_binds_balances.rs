use crate::{events::get_log_changed_balances, pb::cowamm::{CowPool, CowPoolBinds}};
use anyhow::{Ok, Result};
use substreams::{
    scalar::BigInt,
    {prelude::StoreGetProto, store::StoreGet}
};
use substreams_ethereum::pb::eth::v2::Block;
use substreams_helper::hex::Hexable;
use tycho_substreams::prelude::*;

#[substreams::handlers::map]
pub fn map_cowpool_binds_balances(
    binds: CowPoolBinds
) -> Result<BlockBalanceDeltas, anyhow::Error> {
    //a balance delta is created for each bind to the pool
    let mut balance_deltas = Vec::new();
    let mut tx_deltas = Vec::new();

    for bind in binds.binds.iter() {
        let delta = BalanceDelta {
            ord: bind.ordinal,
            tx: Some(Transaction {
                from: bind.tx.as_ref().unwrap().from.clone(),
                to: bind.tx.as_ref().unwrap().to.clone(),
                hash: bind.tx.as_ref().unwrap().hash.clone(),
                index: bind.tx.as_ref().unwrap().index.clone(),
            }),
            token: bind.token.clone(),
            delta: BigInt::from_unsigned_bytes_be(&bind.amount).to_signed_bytes_be(),
            component_id: bind
                .address
                .clone()
                .to_hex()
                .as_bytes()
                .to_vec(),
        };
        //should we also do a get_log_changed_balances here if there were any arbitrary transfers to the pool 
        //during bind? before CowAMMPoolCreated?
        tx_deltas.push(delta);
    }

    if !tx_deltas.is_empty() {
        balance_deltas.extend(tx_deltas);
    }

    Ok(BlockBalanceDeltas { balance_deltas })
}
