use crate::pb::cowamm::{CowPoolBind, CowPoolBinds};
use anyhow::{Ok, Result};
use substreams_ethereum::pb::eth::v2::Block;
use crate::abi::b_cow_pool::functions::Bind;
use substreams_helper::hex::Hexable;

#[substreams::handlers::map]
pub fn map_cowpool_binds(block: Block) -> Result<CowPoolBinds> {
    const BIND_TOPIC: &str = "0xe4e1e53800000000000000000000000000000000000000000000000000000000";

     let binds = block
        .transaction_traces
        .iter()
        // extract (tx, receipt) pairs; skip tx without receipts
        .filter_map(|tx| tx.receipt.as_ref().map(|receipt| (tx, receipt)))
        // for each (tx, receipt) emit all the matching binds
        .flat_map(|(tx, receipt)| {
            receipt
                .logs
                .iter()
                // topic match
                .filter(|log| {
                    log.topics
                        .get(0)
                        .map(|t| t.to_hex())
                        == Some(BIND_TOPIC.to_string())
                })
                // validate log data and map to CowPoolBind
                .filter_map(move |log| {

                    // Find the call that contains this log by matching addresses and checking calls
                    let call = tx.calls.iter().find(|call| {
                        call.address == log.address && !call.state_reverted
                    })?;

                    let bind = Bind::decode(call).expect("couldn't decode bind");
                    let token = bind.token;
                    let amount = bind.balance.to_signed_bytes_be();
                    let weight = bind.denorm.to_signed_bytes_be();

                    Some(CowPoolBind {
                        address: log.address.clone(),
                        token: token,
                        amount: amount,
                        weight: weight,
                        tx: Some(tx.into()), // full TransactionTrace
                        ordinal: log.ordinal
                    })
                })
        })
        .collect::<Vec<_>>();

    Ok(CowPoolBinds { binds })
}
