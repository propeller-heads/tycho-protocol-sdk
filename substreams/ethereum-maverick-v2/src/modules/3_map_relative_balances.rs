use std::str::FromStr;
use anyhow::{Ok, Result};
use ethabi::ethereum_types::Address;
use substreams::prelude::StoreGetProto;
use tycho_substreams::prelude::*;
use substreams_ethereum::pb::eth::v2::{Block};
use substreams::store::{StoreGet, StoreGetString};
use substreams_helper::hex::Hexable;
use crate::events::get_log_changed_balances;
use crate::pb::maverick::v2::{BalanceDeltas, Pool};

#[substreams::handlers::map]
pub fn map_relative_balances(
    block: Block,
    pools_store: StoreGetProto<Pool>,
) -> Result<BalanceDeltas, anyhow::Error> {
    let mut balance_deltas = Vec::new();
    for trx in block.transactions() {
        let mut tx_deltas = Vec::new();
        for log in trx
            .calls
            .iter()
            .filter(|call| !call.state_reverted)
            .flat_map(|call| &call.logs)
        {
            if let Some(pool) =
                pools_store.get_last(format!("{}:{}", "Pool", &log.address.to_hex()))
            {
                tx_deltas.extend(get_log_changed_balances(log, &pool))
            } else {
                continue;
            }
        }
        if !tx_deltas.is_empty() {
            balance_deltas.extend(tx_deltas);
        }
    }
    Ok(BalanceDeltas {deltas: balance_deltas })
}