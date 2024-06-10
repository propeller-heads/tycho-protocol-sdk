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

const ELASTICFACTORY_TRACKED_CONTRACT: [u8; 20] = hex!("c7a590291e07b9fe9e64b86c58fd8fc764308c4a");

fn map_elasticfactory_events(blk: &eth::Block, events: &mut contract::Events) {
    events.elasticfactory_config_master_updateds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == ELASTICFACTORY_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::elasticfactory_contract::events::ConfigMasterUpdated::match_and_decode(log) {
                        return Some(contract::ElasticfactoryConfigMasterUpdated {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            new_config_master: event.new_config_master,
                            old_config_master: event.old_config_master,
                        });
                    }

                    None
                })
        })
        .collect());
    events.elasticfactory_fee_configuration_updateds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == ELASTICFACTORY_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::elasticfactory_contract::events::FeeConfigurationUpdated::match_and_decode(log) {
                        return Some(contract::ElasticfactoryFeeConfigurationUpdated {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            fee_to: event.fee_to,
                            government_fee_units: event.government_fee_units.to_u64(),
                        });
                    }

                    None
                })
        })
        .collect());
    events.elasticfactory_nft_manager_addeds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == ELASTICFACTORY_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::elasticfactory_contract::events::NftManagerAdded::match_and_decode(log) {
                        return Some(contract::ElasticfactoryNftManagerAdded {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            added: event.added,
                            u_nft_manager: event.u_nft_manager,
                        });
                    }

                    None
                })
        })
        .collect());
    events.elasticfactory_nft_manager_removeds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == ELASTICFACTORY_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::elasticfactory_contract::events::NftManagerRemoved::match_and_decode(log) {
                        return Some(contract::ElasticfactoryNftManagerRemoved {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            removed: event.removed,
                            u_nft_manager: event.u_nft_manager,
                        });
                    }

                    None
                })
        })
        .collect());
    events.elasticfactory_pool_createds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == ELASTICFACTORY_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::elasticfactory_contract::events::PoolCreated::match_and_decode(log) {
                        return Some(contract::ElasticfactoryPoolCreated {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            pool: event.pool,
                            swap_fee_units: event.swap_fee_units.to_u64(),
                            tick_distance: Into::<num_bigint::BigInt>::into(event.tick_distance).to_i64().unwrap(),
                            token0: event.token0,
                            token1: event.token1,
                        });
                    }

                    None
                })
        })
        .collect());
    events.elasticfactory_swap_fee_enableds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == ELASTICFACTORY_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::elasticfactory_contract::events::SwapFeeEnabled::match_and_decode(log) {
                        return Some(contract::ElasticfactorySwapFeeEnabled {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            swap_fee_units: event.swap_fee_units.to_u64(),
                            tick_distance: Into::<num_bigint::BigInt>::into(event.tick_distance).to_i64().unwrap(),
                        });
                    }

                    None
                })
        })
        .collect());
    events.elasticfactory_vesting_period_updateds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == ELASTICFACTORY_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::elasticfactory_contract::events::VestingPeriodUpdated::match_and_decode(log) {
                        return Some(contract::ElasticfactoryVestingPeriodUpdated {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            vesting_period: event.vesting_period.to_u64(),
                        });
                    }

                    None
                })
        })
        .collect());
    events.elasticfactory_whitelist_disableds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == ELASTICFACTORY_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::elasticfactory_contract::events::WhitelistDisabled::match_and_decode(log) {
                        return Some(contract::ElasticfactoryWhitelistDisabled {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                        });
                    }

                    None
                })
        })
        .collect());
    events.elasticfactory_whitelist_enableds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == ELASTICFACTORY_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::elasticfactory_contract::events::WhitelistEnabled::match_and_decode(log) {
                        return Some(contract::ElasticfactoryWhitelistEnabled {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
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
                            owner: event.owner,
                            qty: event.qty.to_string(),
                            qty0: event.qty0.to_string(),
                            qty1: event.qty1.to_string(),
                            tick_lower: Into::<num_bigint::BigInt>::into(event.tick_lower).to_i64().unwrap(),
                            tick_upper: Into::<num_bigint::BigInt>::into(event.tick_upper).to_i64().unwrap(),
                        });
                    }

                    None
                })
        })
        .collect());

    events.pool_burn_r_tokens.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| is_declared_dds_address(&log.address, log.ordinal, dds_store))
                .filter_map(|log| {
                    if let Some(event) = abi::pool_contract::events::BurnRTokens::match_and_decode(log) {
                        return Some(contract::PoolBurnRTokens {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            evt_address: Hex(&log.address).to_string(),
                            owner: event.owner,
                            qty: event.qty.to_string(),
                            qty0: event.qty0.to_string(),
                            qty1: event.qty1.to_string(),
                        });
                    }

                    None
                })
        })
        .collect());

    events.pool_flashes.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| is_declared_dds_address(&log.address, log.ordinal, dds_store))
                .filter_map(|log| {
                    if let Some(event) = abi::pool_contract::events::Flash::match_and_decode(log) {
                        return Some(contract::PoolFlash {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            evt_address: Hex(&log.address).to_string(),
                            paid0: event.paid0.to_string(),
                            paid1: event.paid1.to_string(),
                            qty0: event.qty0.to_string(),
                            qty1: event.qty1.to_string(),
                            recipient: event.recipient,
                            sender: event.sender,
                        });
                    }

                    None
                })
        })
        .collect());

    events.pool_initializes.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| is_declared_dds_address(&log.address, log.ordinal, dds_store))
                .filter_map(|log| {
                    if let Some(event) = abi::pool_contract::events::Initialize::match_and_decode(log) {
                        return Some(contract::PoolInitialize {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            evt_address: Hex(&log.address).to_string(),
                            sqrt_p: event.sqrt_p.to_string(),
                            tick: Into::<num_bigint::BigInt>::into(event.tick).to_i64().unwrap(),
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
                            owner: event.owner,
                            qty: event.qty.to_string(),
                            qty0: event.qty0.to_string(),
                            qty1: event.qty1.to_string(),
                            sender: event.sender,
                            tick_lower: Into::<num_bigint::BigInt>::into(event.tick_lower).to_i64().unwrap(),
                            tick_upper: Into::<num_bigint::BigInt>::into(event.tick_upper).to_i64().unwrap(),
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
                            current_tick: Into::<num_bigint::BigInt>::into(event.current_tick).to_i64().unwrap(),
                            delta_qty0: event.delta_qty0.to_string(),
                            delta_qty1: event.delta_qty1.to_string(),
                            liquidity: event.liquidity.to_string(),
                            recipient: event.recipient,
                            sender: event.sender,
                            sqrt_p: event.sqrt_p.to_string(),
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
}


fn db_elasticfactory_out(events: &contract::Events, tables: &mut DatabaseChangeTables) {
    // Loop over all the abis events to create table changes
    events.elasticfactory_config_master_updateds.iter().for_each(|evt| {
        tables
            .create_row("elasticfactory_config_master_updated", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("new_config_master", Hex(&evt.new_config_master).to_string())
            .set("old_config_master", Hex(&evt.old_config_master).to_string());
    });
    events.elasticfactory_fee_configuration_updateds.iter().for_each(|evt| {
        tables
            .create_row("elasticfactory_fee_configuration_updated", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("fee_to", Hex(&evt.fee_to).to_string())
            .set("government_fee_units", evt.government_fee_units);
    });
    events.elasticfactory_nft_manager_addeds.iter().for_each(|evt| {
        tables
            .create_row("elasticfactory_nft_manager_added", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("added", evt.added)
            .set("u_nft_manager", Hex(&evt.u_nft_manager).to_string());
    });
    events.elasticfactory_nft_manager_removeds.iter().for_each(|evt| {
        tables
            .create_row("elasticfactory_nft_manager_removed", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("removed", evt.removed)
            .set("u_nft_manager", Hex(&evt.u_nft_manager).to_string());
    });
    events.elasticfactory_pool_createds.iter().for_each(|evt| {
        tables
            .create_row("elasticfactory_pool_created", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("pool", Hex(&evt.pool).to_string())
            .set("swap_fee_units", evt.swap_fee_units)
            .set("tick_distance", evt.tick_distance)
            .set("token0", Hex(&evt.token0).to_string())
            .set("token1", Hex(&evt.token1).to_string());
    });
    events.elasticfactory_swap_fee_enableds.iter().for_each(|evt| {
        tables
            .create_row("elasticfactory_swap_fee_enabled", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("swap_fee_units", evt.swap_fee_units)
            .set("tick_distance", evt.tick_distance);
    });
    events.elasticfactory_vesting_period_updateds.iter().for_each(|evt| {
        tables
            .create_row("elasticfactory_vesting_period_updated", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("vesting_period", evt.vesting_period);
    });
    events.elasticfactory_whitelist_disableds.iter().for_each(|evt| {
        tables
            .create_row("elasticfactory_whitelist_disabled", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number);
    });
    events.elasticfactory_whitelist_enableds.iter().for_each(|evt| {
        tables
            .create_row("elasticfactory_whitelist_enabled", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number);
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
            .set("owner", Hex(&evt.owner).to_string())
            .set("qty", BigDecimal::from_str(&evt.qty).unwrap())
            .set("qty0", BigDecimal::from_str(&evt.qty0).unwrap())
            .set("qty1", BigDecimal::from_str(&evt.qty1).unwrap())
            .set("tick_lower", evt.tick_lower)
            .set("tick_upper", evt.tick_upper);
    });
    events.pool_burn_r_tokens.iter().for_each(|evt| {
        tables
            .create_row("pool_burn_r_tokens", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("evt_address", &evt.evt_address)
            .set("owner", Hex(&evt.owner).to_string())
            .set("qty", BigDecimal::from_str(&evt.qty).unwrap())
            .set("qty0", BigDecimal::from_str(&evt.qty0).unwrap())
            .set("qty1", BigDecimal::from_str(&evt.qty1).unwrap());
    });
    events.pool_flashes.iter().for_each(|evt| {
        tables
            .create_row("pool_flash", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("evt_address", &evt.evt_address)
            .set("paid0", BigDecimal::from_str(&evt.paid0).unwrap())
            .set("paid1", BigDecimal::from_str(&evt.paid1).unwrap())
            .set("qty0", BigDecimal::from_str(&evt.qty0).unwrap())
            .set("qty1", BigDecimal::from_str(&evt.qty1).unwrap())
            .set("recipient", Hex(&evt.recipient).to_string())
            .set("sender", Hex(&evt.sender).to_string());
    });
    events.pool_initializes.iter().for_each(|evt| {
        tables
            .create_row("pool_initialize", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("evt_address", &evt.evt_address)
            .set("sqrt_p", BigDecimal::from_str(&evt.sqrt_p).unwrap())
            .set("tick", evt.tick);
    });
    events.pool_mints.iter().for_each(|evt| {
        tables
            .create_row("pool_mint", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("evt_address", &evt.evt_address)
            .set("owner", Hex(&evt.owner).to_string())
            .set("qty", BigDecimal::from_str(&evt.qty).unwrap())
            .set("qty0", BigDecimal::from_str(&evt.qty0).unwrap())
            .set("qty1", BigDecimal::from_str(&evt.qty1).unwrap())
            .set("sender", Hex(&evt.sender).to_string())
            .set("tick_lower", evt.tick_lower)
            .set("tick_upper", evt.tick_upper);
    });
    events.pool_swaps.iter().for_each(|evt| {
        tables
            .create_row("pool_swap", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("evt_address", &evt.evt_address)
            .set("current_tick", evt.current_tick)
            .set("delta_qty0", BigDecimal::from_str(&evt.delta_qty0).unwrap())
            .set("delta_qty1", BigDecimal::from_str(&evt.delta_qty1).unwrap())
            .set("liquidity", BigDecimal::from_str(&evt.liquidity).unwrap())
            .set("recipient", Hex(&evt.recipient).to_string())
            .set("sender", Hex(&evt.sender).to_string())
            .set("sqrt_p", BigDecimal::from_str(&evt.sqrt_p).unwrap());
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
}


fn graph_elasticfactory_out(events: &contract::Events, tables: &mut EntityChangesTables) {
    // Loop over all the abis events to create table changes
    events.elasticfactory_config_master_updateds.iter().for_each(|evt| {
        tables
            .create_row("elasticfactory_config_master_updated", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("new_config_master", Hex(&evt.new_config_master).to_string())
            .set("old_config_master", Hex(&evt.old_config_master).to_string());
    });
    events.elasticfactory_fee_configuration_updateds.iter().for_each(|evt| {
        tables
            .create_row("elasticfactory_fee_configuration_updated", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("fee_to", Hex(&evt.fee_to).to_string())
            .set("government_fee_units", evt.government_fee_units);
    });
    events.elasticfactory_nft_manager_addeds.iter().for_each(|evt| {
        tables
            .create_row("elasticfactory_nft_manager_added", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("added", evt.added)
            .set("u_nft_manager", Hex(&evt.u_nft_manager).to_string());
    });
    events.elasticfactory_nft_manager_removeds.iter().for_each(|evt| {
        tables
            .create_row("elasticfactory_nft_manager_removed", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("removed", evt.removed)
            .set("u_nft_manager", Hex(&evt.u_nft_manager).to_string());
    });
    events.elasticfactory_pool_createds.iter().for_each(|evt| {
        tables
            .create_row("elasticfactory_pool_created", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("pool", Hex(&evt.pool).to_string())
            .set("swap_fee_units", evt.swap_fee_units)
            .set("tick_distance", evt.tick_distance)
            .set("token0", Hex(&evt.token0).to_string())
            .set("token1", Hex(&evt.token1).to_string());
    });
    events.elasticfactory_swap_fee_enableds.iter().for_each(|evt| {
        tables
            .create_row("elasticfactory_swap_fee_enabled", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("swap_fee_units", evt.swap_fee_units)
            .set("tick_distance", evt.tick_distance);
    });
    events.elasticfactory_vesting_period_updateds.iter().for_each(|evt| {
        tables
            .create_row("elasticfactory_vesting_period_updated", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("vesting_period", evt.vesting_period);
    });
    events.elasticfactory_whitelist_disableds.iter().for_each(|evt| {
        tables
            .create_row("elasticfactory_whitelist_disabled", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number);
    });
    events.elasticfactory_whitelist_enableds.iter().for_each(|evt| {
        tables
            .create_row("elasticfactory_whitelist_enabled", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number);
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
            .set("owner", Hex(&evt.owner).to_string())
            .set("qty", BigDecimal::from_str(&evt.qty).unwrap())
            .set("qty0", BigDecimal::from_str(&evt.qty0).unwrap())
            .set("qty1", BigDecimal::from_str(&evt.qty1).unwrap())
            .set("tick_lower", evt.tick_lower)
            .set("tick_upper", evt.tick_upper);
    });
    events.pool_burn_r_tokens.iter().for_each(|evt| {
        tables
            .create_row("pool_burn_r_tokens", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("evt_address", &evt.evt_address)
            .set("owner", Hex(&evt.owner).to_string())
            .set("qty", BigDecimal::from_str(&evt.qty).unwrap())
            .set("qty0", BigDecimal::from_str(&evt.qty0).unwrap())
            .set("qty1", BigDecimal::from_str(&evt.qty1).unwrap());
    });
    events.pool_flashes.iter().for_each(|evt| {
        tables
            .create_row("pool_flash", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("evt_address", &evt.evt_address)
            .set("paid0", BigDecimal::from_str(&evt.paid0).unwrap())
            .set("paid1", BigDecimal::from_str(&evt.paid1).unwrap())
            .set("qty0", BigDecimal::from_str(&evt.qty0).unwrap())
            .set("qty1", BigDecimal::from_str(&evt.qty1).unwrap())
            .set("recipient", Hex(&evt.recipient).to_string())
            .set("sender", Hex(&evt.sender).to_string());
    });
    events.pool_initializes.iter().for_each(|evt| {
        tables
            .create_row("pool_initialize", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("evt_address", &evt.evt_address)
            .set("sqrt_p", BigDecimal::from_str(&evt.sqrt_p).unwrap())
            .set("tick", evt.tick);
    });
    events.pool_mints.iter().for_each(|evt| {
        tables
            .create_row("pool_mint", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("evt_address", &evt.evt_address)
            .set("owner", Hex(&evt.owner).to_string())
            .set("qty", BigDecimal::from_str(&evt.qty).unwrap())
            .set("qty0", BigDecimal::from_str(&evt.qty0).unwrap())
            .set("qty1", BigDecimal::from_str(&evt.qty1).unwrap())
            .set("sender", Hex(&evt.sender).to_string())
            .set("tick_lower", evt.tick_lower)
            .set("tick_upper", evt.tick_upper);
    });
    events.pool_swaps.iter().for_each(|evt| {
        tables
            .create_row("pool_swap", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("evt_address", &evt.evt_address)
            .set("current_tick", evt.current_tick)
            .set("delta_qty0", BigDecimal::from_str(&evt.delta_qty0).unwrap())
            .set("delta_qty1", BigDecimal::from_str(&evt.delta_qty1).unwrap())
            .set("liquidity", BigDecimal::from_str(&evt.liquidity).unwrap())
            .set("recipient", Hex(&evt.recipient).to_string())
            .set("sender", Hex(&evt.sender).to_string())
            .set("sqrt_p", BigDecimal::from_str(&evt.sqrt_p).unwrap());
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
}
#[substreams::handlers::store]
fn store_elasticfactory_pool_created(blk: eth::Block, store: StoreSetInt64) {
    for rcpt in blk.receipts() {
        for log in rcpt
            .receipt
            .logs
            .iter()
            .filter(|log| log.address == ELASTICFACTORY_TRACKED_CONTRACT)
        {
            if let Some(event) = abi::elasticfactory_contract::events::PoolCreated::match_and_decode(log) {
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
    map_elasticfactory_events(&blk, &mut events);
    map_pool_events(&blk, &store_pool, &mut events);
    Ok(events)
}

#[substreams::handlers::map]
fn db_out(events: contract::Events) -> Result<DatabaseChanges, substreams::errors::Error> {
    // Initialize Database Changes container
    let mut tables = DatabaseChangeTables::new();
    db_elasticfactory_out(&events, &mut tables);
    db_pool_out(&events, &mut tables);
    Ok(tables.to_database_changes())
}

#[substreams::handlers::map]
fn graph_out(events: contract::Events) -> Result<EntityChanges, substreams::errors::Error> {
    // Initialize Database Changes container
    let mut tables = EntityChangesTables::new();
    graph_elasticfactory_out(&events, &mut tables);
    graph_pool_out(&events, &mut tables);
    Ok(tables.to_entity_changes())
}
