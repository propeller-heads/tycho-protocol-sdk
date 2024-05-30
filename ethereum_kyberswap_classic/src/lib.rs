mod abi;
mod pb;
use hex_literal::hex;
use pb::contract::v1 as contract;
use substreams::prelude::*;
use substreams::store;
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

const FACTORY_TRACKED_CONTRACT: [u8; 20] = hex!("833e4083b7ae46cea85695c4f7ed25cdad8886de");

fn map_factory_events(blk: &eth::Block, events: &mut contract::Events) {
    events.factory_pool_createds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == FACTORY_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::factory_contract::events::PoolCreated::match_and_decode(log) {
                        return Some(contract::FactoryPoolCreated {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            amp_bps: event.amp_bps.to_u64(),
                            pool: event.pool,
                            token0: event.token0,
                            token1: event.token1,
                            total_pool: event.total_pool.to_string(),
                        });
                    }

                    None
                })
        })
        .collect());
    events.factory_set_fee_configurations.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == FACTORY_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::factory_contract::events::SetFeeConfiguration::match_and_decode(log) {
                        return Some(contract::FactorySetFeeConfiguration {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            fee_to: event.fee_to,
                            government_fee_bps: event.government_fee_bps.to_u64(),
                        });
                    }

                    None
                })
        })
        .collect());
    events.factory_set_fee_to_setters.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == FACTORY_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::factory_contract::events::SetFeeToSetter::match_and_decode(log) {
                        return Some(contract::FactorySetFeeToSetter {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            fee_to_setter: event.fee_to_setter,
                        });
                    }

                    None
                })
        })
        .collect());
}

fn is_declared_dds_address(addr: &Vec<u8>, ordinal: u64, dds_store: &store::StoreGetInt64) -> bool {
    //    substreams::log::info!("Checking if address {} is declared dds address", Hex(addr).to_string());
    if dds_store.get_at(ordinal, Hex(addr).to_string()).is_some() {
        return true;
    }
    return false;
}

fn map_pool_events(
    blk: &eth::Block,
    dds_store: &store::StoreGetInt64,
    events: &mut contract::Events,
) {

    events.pool_approvals.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| is_declared_dds_address(&log.address, log.ordinal, dds_store))
                .filter_map(|log| {
                    if let Some(event) = abi::pool_contract::events::Approval::match_and_decode(log) {
                        return Some(contract::PoolApproval {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            evt_address: Hex(&log.address).to_string(),
                            owner: event.owner,
                            spender: event.spender,
                            value: event.value.to_string(),
                        });
                    }

                    None
                })
        })
        .collect());

    events.pool_burns.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| is_declared_dds_address(&log.address, log.ordinal, dds_store))
                .filter_map(|log| {
                    if let Some(event) = abi::pool_contract::events::Burn::match_and_decode(log) {
                        return Some(contract::PoolBurn {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            evt_address: Hex(&log.address).to_string(),
                            amount0: event.amount0.to_string(),
                            amount1: event.amount1.to_string(),
                            sender: event.sender,
                            to: event.to,
                        });
                    }

                    None
                })
        })
        .collect());

    events.pool_mints.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| is_declared_dds_address(&log.address, log.ordinal, dds_store))
                .filter_map(|log| {
                    if let Some(event) = abi::pool_contract::events::Mint::match_and_decode(log) {
                        return Some(contract::PoolMint {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            evt_address: Hex(&log.address).to_string(),
                            amount0: event.amount0.to_string(),
                            amount1: event.amount1.to_string(),
                            sender: event.sender,
                        });
                    }

                    None
                })
        })
        .collect());

    events.pool_swaps.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| is_declared_dds_address(&log.address, log.ordinal, dds_store))
                .filter_map(|log| {
                    if let Some(event) = abi::pool_contract::events::Swap::match_and_decode(log) {
                        return Some(contract::PoolSwap {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            evt_address: Hex(&log.address).to_string(),
                            amount0_in: event.amount0_in.to_string(),
                            amount0_out: event.amount0_out.to_string(),
                            amount1_in: event.amount1_in.to_string(),
                            amount1_out: event.amount1_out.to_string(),
                            fee_in_precision: event.fee_in_precision.to_string(),
                            sender: event.sender,
                            to: event.to,
                        });
                    }

                    None
                })
        })
        .collect());

    events.pool_syncs.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| is_declared_dds_address(&log.address, log.ordinal, dds_store))
                .filter_map(|log| {
                    if let Some(event) = abi::pool_contract::events::Sync::match_and_decode(log) {
                        return Some(contract::PoolSync {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            evt_address: Hex(&log.address).to_string(),
                            reserve0: event.reserve0.to_string(),
                            reserve1: event.reserve1.to_string(),
                            v_reserve0: event.v_reserve0.to_string(),
                            v_reserve1: event.v_reserve1.to_string(),
                        });
                    }

                    None
                })
        })
        .collect());

    events.pool_transfers.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| is_declared_dds_address(&log.address, log.ordinal, dds_store))
                .filter_map(|log| {
                    if let Some(event) = abi::pool_contract::events::Transfer::match_and_decode(log) {
                        return Some(contract::PoolTransfer {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            evt_address: Hex(&log.address).to_string(),
                            from: event.from,
                            to: event.to,
                            value: event.value.to_string(),
                        });
                    }

                    None
                })
        })
        .collect());

    events.pool_update_emas.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| is_declared_dds_address(&log.address, log.ordinal, dds_store))
                .filter_map(|log| {
                    if let Some(event) = abi::pool_contract::events::UpdateEma::match_and_decode(log) {
                        return Some(contract::PoolUpdateEma {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            evt_address: Hex(&log.address).to_string(),
                            last_block_volume: event.last_block_volume.to_string(),
                            long_ema: event.long_ema.to_string(),
                            short_ema: event.short_ema.to_string(),
                            skip_block: event.skip_block.to_string(),
                        });
                    }

                    None
                })
        })
        .collect());
}


fn db_factory_out(events: &contract::Events, tables: &mut DatabaseChangeTables) {
    // Loop over all the abis events to create table changes
    events.factory_pool_createds.iter().for_each(|evt| {
        tables
            .create_row("factory_pool_created", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("amp_bps", evt.amp_bps)
            .set("pool", Hex(&evt.pool).to_string())
            .set("token0", Hex(&evt.token0).to_string())
            .set("token1", Hex(&evt.token1).to_string())
            .set("total_pool", BigDecimal::from_str(&evt.total_pool).unwrap());
    });
    events.factory_set_fee_configurations.iter().for_each(|evt| {
        tables
            .create_row("factory_set_fee_configuration", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("fee_to", Hex(&evt.fee_to).to_string())
            .set("government_fee_bps", evt.government_fee_bps);
    });
    events.factory_set_fee_to_setters.iter().for_each(|evt| {
        tables
            .create_row("factory_set_fee_to_setter", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("fee_to_setter", Hex(&evt.fee_to_setter).to_string());
    });
}
fn db_pool_out(events: &contract::Events, tables: &mut DatabaseChangeTables) {
    // Loop over all the abis events to create table changes
    events.pool_approvals.iter().for_each(|evt| {
        tables
            .create_row("pool_approval", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("evt_address", &evt.evt_address)
            .set("owner", Hex(&evt.owner).to_string())
            .set("spender", Hex(&evt.spender).to_string())
            .set("value", BigDecimal::from_str(&evt.value).unwrap());
    });
    events.pool_burns.iter().for_each(|evt| {
        tables
            .create_row("pool_burn", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("evt_address", &evt.evt_address)
            .set("amount0", BigDecimal::from_str(&evt.amount0).unwrap())
            .set("amount1", BigDecimal::from_str(&evt.amount1).unwrap())
            .set("sender", Hex(&evt.sender).to_string())
            .set("to", Hex(&evt.to).to_string());
    });
    events.pool_mints.iter().for_each(|evt| {
        tables
            .create_row("pool_mint", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("evt_address", &evt.evt_address)
            .set("amount0", BigDecimal::from_str(&evt.amount0).unwrap())
            .set("amount1", BigDecimal::from_str(&evt.amount1).unwrap())
            .set("sender", Hex(&evt.sender).to_string());
    });
    events.pool_swaps.iter().for_each(|evt| {
        tables
            .create_row("pool_swap", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("evt_address", &evt.evt_address)
            .set("amount0_in", BigDecimal::from_str(&evt.amount0_in).unwrap())
            .set("amount0_out", BigDecimal::from_str(&evt.amount0_out).unwrap())
            .set("amount1_in", BigDecimal::from_str(&evt.amount1_in).unwrap())
            .set("amount1_out", BigDecimal::from_str(&evt.amount1_out).unwrap())
            .set("fee_in_precision", BigDecimal::from_str(&evt.fee_in_precision).unwrap())
            .set("sender", Hex(&evt.sender).to_string())
            .set("to", Hex(&evt.to).to_string());
    });
    events.pool_syncs.iter().for_each(|evt| {
        tables
            .create_row("pool_sync", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("evt_address", &evt.evt_address)
            .set("reserve0", BigDecimal::from_str(&evt.reserve0).unwrap())
            .set("reserve1", BigDecimal::from_str(&evt.reserve1).unwrap())
            .set("v_reserve0", BigDecimal::from_str(&evt.v_reserve0).unwrap())
            .set("v_reserve1", BigDecimal::from_str(&evt.v_reserve1).unwrap());
    });
    events.pool_transfers.iter().for_each(|evt| {
        tables
            .create_row("pool_transfer", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("evt_address", &evt.evt_address)
            .set("from", Hex(&evt.from).to_string())
            .set("to", Hex(&evt.to).to_string())
            .set("value", BigDecimal::from_str(&evt.value).unwrap());
    });
    events.pool_update_emas.iter().for_each(|evt| {
        tables
            .create_row("pool_update_ema", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("evt_address", &evt.evt_address)
            .set("last_block_volume", BigDecimal::from_str(&evt.last_block_volume).unwrap())
            .set("long_ema", BigDecimal::from_str(&evt.long_ema).unwrap())
            .set("short_ema", BigDecimal::from_str(&evt.short_ema).unwrap())
            .set("skip_block", BigDecimal::from_str(&evt.skip_block).unwrap());
    });
}


fn graph_factory_out(events: &contract::Events, tables: &mut EntityChangesTables) {
    // Loop over all the abis events to create table changes
    events.factory_pool_createds.iter().for_each(|evt| {
        tables
            .create_row("factory_pool_created", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("amp_bps", evt.amp_bps)
            .set("pool", Hex(&evt.pool).to_string())
            .set("token0", Hex(&evt.token0).to_string())
            .set("token1", Hex(&evt.token1).to_string())
            .set("total_pool", BigDecimal::from_str(&evt.total_pool).unwrap());
    });
    events.factory_set_fee_configurations.iter().for_each(|evt| {
        tables
            .create_row("factory_set_fee_configuration", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("fee_to", Hex(&evt.fee_to).to_string())
            .set("government_fee_bps", evt.government_fee_bps);
    });
    events.factory_set_fee_to_setters.iter().for_each(|evt| {
        tables
            .create_row("factory_set_fee_to_setter", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("fee_to_setter", Hex(&evt.fee_to_setter).to_string());
    });
}
fn graph_pool_out(events: &contract::Events, tables: &mut EntityChangesTables) {
    // Loop over all the abis events to create table changes
    events.pool_approvals.iter().for_each(|evt| {
        tables
            .create_row("pool_approval", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("evt_address", &evt.evt_address)
            .set("owner", Hex(&evt.owner).to_string())
            .set("spender", Hex(&evt.spender).to_string())
            .set("value", BigDecimal::from_str(&evt.value).unwrap());
    });
    events.pool_burns.iter().for_each(|evt| {
        tables
            .create_row("pool_burn", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("evt_address", &evt.evt_address)
            .set("amount0", BigDecimal::from_str(&evt.amount0).unwrap())
            .set("amount1", BigDecimal::from_str(&evt.amount1).unwrap())
            .set("sender", Hex(&evt.sender).to_string())
            .set("to", Hex(&evt.to).to_string());
    });
    events.pool_mints.iter().for_each(|evt| {
        tables
            .create_row("pool_mint", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("evt_address", &evt.evt_address)
            .set("amount0", BigDecimal::from_str(&evt.amount0).unwrap())
            .set("amount1", BigDecimal::from_str(&evt.amount1).unwrap())
            .set("sender", Hex(&evt.sender).to_string());
    });
    events.pool_swaps.iter().for_each(|evt| {
        tables
            .create_row("pool_swap", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("evt_address", &evt.evt_address)
            .set("amount0_in", BigDecimal::from_str(&evt.amount0_in).unwrap())
            .set("amount0_out", BigDecimal::from_str(&evt.amount0_out).unwrap())
            .set("amount1_in", BigDecimal::from_str(&evt.amount1_in).unwrap())
            .set("amount1_out", BigDecimal::from_str(&evt.amount1_out).unwrap())
            .set("fee_in_precision", BigDecimal::from_str(&evt.fee_in_precision).unwrap())
            .set("sender", Hex(&evt.sender).to_string())
            .set("to", Hex(&evt.to).to_string());
    });
    events.pool_syncs.iter().for_each(|evt| {
        tables
            .create_row("pool_sync", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("evt_address", &evt.evt_address)
            .set("reserve0", BigDecimal::from_str(&evt.reserve0).unwrap())
            .set("reserve1", BigDecimal::from_str(&evt.reserve1).unwrap())
            .set("v_reserve0", BigDecimal::from_str(&evt.v_reserve0).unwrap())
            .set("v_reserve1", BigDecimal::from_str(&evt.v_reserve1).unwrap());
    });
    events.pool_transfers.iter().for_each(|evt| {
        tables
            .create_row("pool_transfer", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("evt_address", &evt.evt_address)
            .set("from", Hex(&evt.from).to_string())
            .set("to", Hex(&evt.to).to_string())
            .set("value", BigDecimal::from_str(&evt.value).unwrap());
    });
    events.pool_update_emas.iter().for_each(|evt| {
        tables
            .create_row("pool_update_ema", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("evt_address", &evt.evt_address)
            .set("last_block_volume", BigDecimal::from_str(&evt.last_block_volume).unwrap())
            .set("long_ema", BigDecimal::from_str(&evt.long_ema).unwrap())
            .set("short_ema", BigDecimal::from_str(&evt.short_ema).unwrap())
            .set("skip_block", BigDecimal::from_str(&evt.skip_block).unwrap());
    });
}
#[substreams::handlers::store]
fn store_factory_pool_created(blk: eth::Block, store: StoreSetInt64) {
    for rcpt in blk.receipts() {
        for log in rcpt
            .receipt
            .logs
            .iter()
            .filter(|log| log.address == FACTORY_TRACKED_CONTRACT)
        {
            if let Some(event) = abi::factory_contract::events::PoolCreated::match_and_decode(log) {
                store.set(log.ordinal, Hex(event.pool).to_string(), &1);
            }
        }
    }
}

#[substreams::handlers::map]
fn map_events(
    blk: eth::Block,
    store_pool: StoreGetInt64,
) -> Result<contract::Events, substreams::errors::Error> {
    let mut events = contract::Events::default();
    map_factory_events(&blk, &mut events);
    map_pool_events(&blk, &store_pool, &mut events);
    Ok(events)
}

#[substreams::handlers::map]
fn db_out(events: contract::Events) -> Result<DatabaseChanges, substreams::errors::Error> {
    // Initialize Database Changes container
    let mut tables = DatabaseChangeTables::new();
    db_factory_out(&events, &mut tables);
    db_pool_out(&events, &mut tables);
    Ok(tables.to_database_changes())
}

#[substreams::handlers::map]
fn graph_out(events: contract::Events) -> Result<EntityChanges, substreams::errors::Error> {
    // Initialize Database Changes container
    let mut tables = EntityChangesTables::new();
    graph_factory_out(&events, &mut tables);
    graph_pool_out(&events, &mut tables);
    Ok(tables.to_entity_changes())
}
