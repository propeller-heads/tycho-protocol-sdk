mod abi;
mod pb;
use hex_literal::hex;
use pb::contract::v1 as contract;
use substreams::Hex;
use substreams_database_change::pb::database::DatabaseChanges;
use substreams_database_change::tables::Tables as DatabaseChangeTables;
use substreams_entity_change::pb::entity::EntityChanges;
use substreams_entity_change::tables::Tables as EntityChangesTables;
use substreams_ethereum::pb::eth::v2 as eth;
use substreams_ethereum::Event;

#[allow(unused_imports)]
use num_traits::cast::ToPrimitive;
use std::str::FromStr;
use substreams::scalar::BigDecimal;

substreams_ethereum::init!();

const ROCKETVAULT_TRACKED_CONTRACT: [u8; 20] = hex!("3bdc69c4e5e13e52a65f5583c23efb9636b469d6");

fn map_rocketvault_events(blk: &eth::Block, events: &mut contract::Events) {
    events.rocketvault_ether_depositeds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == ROCKETVAULT_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::rocketvault_contract::events::EtherDeposited::match_and_decode(log) {
                        return Some(contract::RocketvaultEtherDeposited {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            amount: event.amount.to_string(),
                            by: event.by,
                            time: event.time.to_string(),
                        });
                    }

                    None
                })
        })
        .collect());
    events.rocketvault_ether_withdrawns.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == ROCKETVAULT_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::rocketvault_contract::events::EtherWithdrawn::match_and_decode(log) {
                        return Some(contract::RocketvaultEtherWithdrawn {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            amount: event.amount.to_string(),
                            by: event.by,
                            time: event.time.to_string(),
                        });
                    }

                    None
                })
        })
        .collect());
    events.rocketvault_token_burneds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == ROCKETVAULT_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::rocketvault_contract::events::TokenBurned::match_and_decode(log) {
                        return Some(contract::RocketvaultTokenBurned {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            amount: event.amount.to_string(),
                            by: Vec::from(event.by),
                            time: event.time.to_string(),
                            token_address: event.token_address,
                        });
                    }

                    None
                })
        })
        .collect());
    events.rocketvault_token_depositeds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == ROCKETVAULT_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::rocketvault_contract::events::TokenDeposited::match_and_decode(log) {
                        return Some(contract::RocketvaultTokenDeposited {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            amount: event.amount.to_string(),
                            by: Vec::from(event.by),
                            time: event.time.to_string(),
                            token_address: event.token_address,
                        });
                    }

                    None
                })
        })
        .collect());
    events.rocketvault_token_transfers.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == ROCKETVAULT_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::rocketvault_contract::events::TokenTransfer::match_and_decode(log) {
                        return Some(contract::RocketvaultTokenTransfer {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            amount: event.amount.to_string(),
                            by: Vec::from(event.by),
                            time: event.time.to_string(),
                            to: Vec::from(event.to),
                            token_address: event.token_address,
                        });
                    }

                    None
                })
        })
        .collect());
    events.rocketvault_token_withdrawns.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == ROCKETVAULT_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::rocketvault_contract::events::TokenWithdrawn::match_and_decode(log) {
                        return Some(contract::RocketvaultTokenWithdrawn {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            amount: event.amount.to_string(),
                            by: Vec::from(event.by),
                            time: event.time.to_string(),
                            token_address: event.token_address,
                        });
                    }

                    None
                })
        })
        .collect());
}

fn db_rocketvault_out(events: &contract::Events, tables: &mut DatabaseChangeTables) {
    // Loop over all the abis events to create table changes
    events.rocketvault_ether_depositeds.iter().for_each(|evt| {
        tables
            .create_row("rocketvault_ether_deposited", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("amount", BigDecimal::from_str(&evt.amount).unwrap())
            .set("by", &evt.by)
            .set("time", BigDecimal::from_str(&evt.time).unwrap());
    });
    events.rocketvault_ether_withdrawns.iter().for_each(|evt| {
        tables
            .create_row("rocketvault_ether_withdrawn", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("amount", BigDecimal::from_str(&evt.amount).unwrap())
            .set("by", &evt.by)
            .set("time", BigDecimal::from_str(&evt.time).unwrap());
    });
    events.rocketvault_token_burneds.iter().for_each(|evt| {
        tables
            .create_row("rocketvault_token_burned", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("amount", BigDecimal::from_str(&evt.amount).unwrap())
            .set("by", Hex(&evt.by).to_string())
            .set("time", BigDecimal::from_str(&evt.time).unwrap())
            .set("token_address", Hex(&evt.token_address).to_string());
    });
    events.rocketvault_token_depositeds.iter().for_each(|evt| {
        tables
            .create_row("rocketvault_token_deposited", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("amount", BigDecimal::from_str(&evt.amount).unwrap())
            .set("by", Hex(&evt.by).to_string())
            .set("time", BigDecimal::from_str(&evt.time).unwrap())
            .set("token_address", Hex(&evt.token_address).to_string());
    });
    events.rocketvault_token_transfers.iter().for_each(|evt| {
        tables
            .create_row("rocketvault_token_transfer", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("amount", BigDecimal::from_str(&evt.amount).unwrap())
            .set("by", Hex(&evt.by).to_string())
            .set("time", BigDecimal::from_str(&evt.time).unwrap())
            .set("to", Hex(&evt.to).to_string())
            .set("token_address", Hex(&evt.token_address).to_string());
    });
    events.rocketvault_token_withdrawns.iter().for_each(|evt| {
        tables
            .create_row("rocketvault_token_withdrawn", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("amount", BigDecimal::from_str(&evt.amount).unwrap())
            .set("by", Hex(&evt.by).to_string())
            .set("time", BigDecimal::from_str(&evt.time).unwrap())
            .set("token_address", Hex(&evt.token_address).to_string());
    });
}


fn graph_rocketvault_out(events: &contract::Events, tables: &mut EntityChangesTables) {
    // Loop over all the abis events to create table changes
    events.rocketvault_ether_depositeds.iter().for_each(|evt| {
        tables
            .create_row("rocketvault_ether_deposited", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("amount", BigDecimal::from_str(&evt.amount).unwrap())
            .set("by", &evt.by)
            .set("time", BigDecimal::from_str(&evt.time).unwrap());
    });
    events.rocketvault_ether_withdrawns.iter().for_each(|evt| {
        tables
            .create_row("rocketvault_ether_withdrawn", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("amount", BigDecimal::from_str(&evt.amount).unwrap())
            .set("by", &evt.by)
            .set("time", BigDecimal::from_str(&evt.time).unwrap());
    });
    events.rocketvault_token_burneds.iter().for_each(|evt| {
        tables
            .create_row("rocketvault_token_burned", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("amount", BigDecimal::from_str(&evt.amount).unwrap())
            .set("by", Hex(&evt.by).to_string())
            .set("time", BigDecimal::from_str(&evt.time).unwrap())
            .set("token_address", Hex(&evt.token_address).to_string());
    });
    events.rocketvault_token_depositeds.iter().for_each(|evt| {
        tables
            .create_row("rocketvault_token_deposited", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("amount", BigDecimal::from_str(&evt.amount).unwrap())
            .set("by", Hex(&evt.by).to_string())
            .set("time", BigDecimal::from_str(&evt.time).unwrap())
            .set("token_address", Hex(&evt.token_address).to_string());
    });
    events.rocketvault_token_transfers.iter().for_each(|evt| {
        tables
            .create_row("rocketvault_token_transfer", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("amount", BigDecimal::from_str(&evt.amount).unwrap())
            .set("by", Hex(&evt.by).to_string())
            .set("time", BigDecimal::from_str(&evt.time).unwrap())
            .set("to", Hex(&evt.to).to_string())
            .set("token_address", Hex(&evt.token_address).to_string());
    });
    events.rocketvault_token_withdrawns.iter().for_each(|evt| {
        tables
            .create_row("rocketvault_token_withdrawn", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("amount", BigDecimal::from_str(&evt.amount).unwrap())
            .set("by", Hex(&evt.by).to_string())
            .set("time", BigDecimal::from_str(&evt.time).unwrap())
            .set("token_address", Hex(&evt.token_address).to_string());
    });
}

#[substreams::handlers::map]
fn map_events(blk: eth::Block) -> Result<contract::Events, substreams::errors::Error> {
    let mut events = contract::Events::default();
    map_rocketvault_events(&blk, &mut events);
    Ok(events)
}

#[substreams::handlers::map]
fn db_out(events: contract::Events) -> Result<DatabaseChanges, substreams::errors::Error> {
    // Initialize Database Changes container
    let mut tables = DatabaseChangeTables::new();
    db_rocketvault_out(&events, &mut tables);
    Ok(tables.to_database_changes())
}

#[substreams::handlers::map]
fn graph_out(events: contract::Events) -> Result<EntityChanges, substreams::errors::Error> {
    // Initialize Database Changes container
    let mut tables = EntityChangesTables::new();
    graph_rocketvault_out(&events, &mut tables);
    Ok(tables.to_entity_changes())
}
