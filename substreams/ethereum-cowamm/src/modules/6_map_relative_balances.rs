use crate::{events::get_log_changed_balances, pb::cowamm::{CowPool, CowPoolBind}};
use anyhow::{Ok, Result};
use substreams::{
    scalar::BigInt,
    {prelude::StoreGetProto, store::StoreGet}
};
use substreams_ethereum::pb::eth::v2::Block;
use substreams_helper::hex::Hexable;
use tycho_substreams::prelude::*;

#[substreams::handlers::map]
pub fn map_relative_balances(
    block: Block,
    binds_balance_deltas: BlockBalanceDeltas,
    pools_store: StoreGetProto<CowPool>,

) -> Result<BlockBalanceDeltas, anyhow::Error> {
    let mut balance_deltas = Vec::new();
    //we'll have to combine it with these deltas here 
    for trx in block.transactions() {
        let mut tx_deltas = Vec::new();
        for log in trx
            .calls
            .iter()
            .filter(|call| !call.state_reverted)
            .flat_map(|call| &call.logs)
        {
            if let Some(pool) = pools_store.get_last(format!("Pool:{}", &log.address.to_hex())) {
                //create a balance delta for the amounts added to the pool during the bind method call
                //so if a bind method call was of this pool, we create a delta for it here

                //the problem with this is that we are going to record both bind balance delta changes
                //in the wrong block and our rpc test MIGHT give the error, because the component balance 
                // at that block will not be right 

                //so this means we'll set the ordinal to 0 because its the first balance change that happens 
                //to the pool
                // substreams::log::info!("THIS IS THE log.address {:?}", &log.address);
                // substreams::log::info!("THIS IS THE log.address in hex {}", hex::encode(&log.address));
                // substreams::log::info!("THIS IS THE RES {:?}", res);

                // if let Some(bind) = binds_store.get_last(hex::encode(&log.address)) {
                //     substreams::log::info!("We are in bind right now {}", hex::encode(&log.address));
                //     substreams::log::info!("We are in bind right now {:?}", bind);

                    
                //     let delta = BalanceDelta {
                //         ord: 0,
                //         tx: Some(Transaction {
                //             from: bind.tx.as_ref().unwrap().from.clone(),
                //             to: bind.tx.as_ref().unwrap().to.clone(),
                //             hash: bind.tx.as_ref().unwrap().hash.clone(),
                //             index: bind.tx.as_ref().unwrap().index.clone(),
                //         }),
                //         token: bind.token.clone(), //
                //         delta: BigInt::from_unsigned_bytes_be(&bind.amount).to_signed_bytes_be(),
                //         component_id: pool
                //             .address
                //             .clone()
                //             .to_hex()
                //             .as_bytes()
                //             .to_vec(),
                //     };

                //     tx_deltas.extend(vec![delta]);
                // }
                tx_deltas.extend(get_log_changed_balances(&trx.into(), log, &pool));
            } else {
                continue;
            }
        }
        if !tx_deltas.is_empty() {
            balance_deltas.extend(tx_deltas);

        }
    }

    let mut combined = binds_balance_deltas.balance_deltas.clone();
    combined.extend(balance_deltas.clone());

    Ok(BlockBalanceDeltas { balance_deltas: combined })
}
