use crate::pb::cowamm::{CowPoolBind, CowPoolBinds};
use anyhow::{Ok, Result};
use substreams_ethereum::pb::eth::v2::Block;
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
                    let data = &log.data;
                    if data.len() < 165 {
                        return None;
                    }
                    let token = data.get(80..100)?.to_vec();
                    let amount = data.get(100..132)?.to_vec();
                    let weight = data.get(132..164)?.to_vec();
                    substreams::log::info!("THIS IS THE amount: {}", substreams::Hex(amount.clone())); 

                    //check if the address is the same one (9bd702....)
                    substreams::log::info!("THIS IS THE address: {}", substreams::Hex(log.address.clone()));
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
