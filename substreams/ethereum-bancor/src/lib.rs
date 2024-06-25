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

const POOL_TRACKED_CONTRACT: [u8; 20] = hex!("eef417e1d5cc832e619ae18d2f140de2999dd4fb");

fn map_pool_events(blk: &eth::Block, events: &mut contract::Events) {
    events.pool_admin_changeds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == POOL_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::pool_contract::events::AdminChanged::match_and_decode(log) {
                        return Some(contract::PoolAdminChanged {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            new_admin: event.new_admin,
                            previous_admin: event.previous_admin,
                        });
                    }

                    None
                })
        })
        .collect());
    events.pool_beacon_upgradeds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == POOL_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::pool_contract::events::BeaconUpgraded::match_and_decode(log) {
                        return Some(contract::PoolBeaconUpgraded {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            beacon: event.beacon,
                        });
                    }

                    None
                })
        })
        .collect());
    events.pool_flash_loan_completeds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == POOL_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::pool_contract::events::FlashLoanCompleted::match_and_decode(log) {
                        return Some(contract::PoolFlashLoanCompleted {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            amount: event.amount.to_string(),
                            borrower: event.borrower,
                            fee_amount: event.fee_amount.to_string(),
                            token: event.token,
                        });
                    }

                    None
                })
        })
        .collect());
    events.pool_funds_migrateds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == POOL_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::pool_contract::events::FundsMigrated::match_and_decode(log) {
                        return Some(contract::PoolFundsMigrated {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            amount: event.amount.to_string(),
                            available_amount: event.available_amount.to_string(),
                            context_id: Vec::from(event.context_id),
                            original_amount: event.original_amount.to_string(),
                            provider: event.provider,
                            token: event.token,
                        });
                    }

                    None
                })
        })
        .collect());
    events.pool_min_network_fee_burn_updateds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == POOL_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::pool_contract::events::MinNetworkFeeBurnUpdated::match_and_decode(log) {
                        return Some(contract::PoolMinNetworkFeeBurnUpdated {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            new_min_network_fee_burn: event.new_min_network_fee_burn.to_string(),
                            old_min_network_fee_burn: event.old_min_network_fee_burn.to_string(),
                        });
                    }

                    None
                })
        })
        .collect());
    events.pool_network_fees_burneds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == POOL_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::pool_contract::events::NetworkFeesBurned::match_and_decode(log) {
                        return Some(contract::PoolNetworkFeesBurned {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            amount: event.amount.to_string(),
                            caller: event.caller,
                        });
                    }

                    None
                })
        })
        .collect());
    events.pool_pol_rewards_ppm_updateds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == POOL_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::pool_contract::events::PolRewardsPpmUpdated::match_and_decode(log) {
                        return Some(contract::PoolPolRewardsPpmUpdated {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            new_rewards_ppm: event.new_rewards_ppm.to_u64(),
                            old_rewards_ppm: event.old_rewards_ppm.to_u64(),
                        });
                    }

                    None
                })
        })
        .collect());
    events.pool_pol_withdrawns.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == POOL_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::pool_contract::events::PolWithdrawn::match_and_decode(log) {
                        return Some(contract::PoolPolWithdrawn {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            caller: event.caller,
                            pol_token_amount: event.pol_token_amount.to_string(),
                            token: event.token,
                            user_reward: event.user_reward.to_string(),
                        });
                    }

                    None
                })
        })
        .collect());
    events.pool_pauseds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == POOL_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::pool_contract::events::Paused::match_and_decode(log) {
                        return Some(contract::PoolPaused {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            account: event.account,
                        });
                    }

                    None
                })
        })
        .collect());
    events.pool_pool_addeds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == POOL_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::pool_contract::events::PoolAdded::match_and_decode(log) {
                        return Some(contract::PoolPoolAdded {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            pool: event.pool,
                            pool_collection: event.pool_collection,
                        });
                    }

                    None
                })
        })
        .collect());
    events.pool_pool_collection_addeds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == POOL_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::pool_contract::events::PoolCollectionAdded::match_and_decode(log) {
                        return Some(contract::PoolPoolCollectionAdded {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            pool_collection: event.pool_collection,
                            pool_type: event.pool_type.to_u64(),
                        });
                    }

                    None
                })
        })
        .collect());
    events.pool_pool_collection_removeds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == POOL_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::pool_contract::events::PoolCollectionRemoved::match_and_decode(log) {
                        return Some(contract::PoolPoolCollectionRemoved {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            pool_collection: event.pool_collection,
                            pool_type: event.pool_type.to_u64(),
                        });
                    }

                    None
                })
        })
        .collect());
    events.pool_pool_createds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == POOL_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::pool_contract::events::PoolCreated::match_and_decode(log) {
                        return Some(contract::PoolPoolCreated {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            pool: event.pool,
                            pool_collection: event.pool_collection,
                        });
                    }

                    None
                })
        })
        .collect());
    events.pool_pool_removeds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == POOL_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::pool_contract::events::PoolRemoved::match_and_decode(log) {
                        return Some(contract::PoolPoolRemoved {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            pool: event.pool,
                            pool_collection: event.pool_collection,
                        });
                    }

                    None
                })
        })
        .collect());
    events.pool_role_admin_changeds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == POOL_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::pool_contract::events::RoleAdminChanged::match_and_decode(log) {
                        return Some(contract::PoolRoleAdminChanged {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            new_admin_role: Vec::from(event.new_admin_role),
                            previous_admin_role: Vec::from(event.previous_admin_role),
                            role: Vec::from(event.role),
                        });
                    }

                    None
                })
        })
        .collect());
    events.pool_role_granteds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == POOL_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::pool_contract::events::RoleGranted::match_and_decode(log) {
                        return Some(contract::PoolRoleGranted {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            account: event.account,
                            role: Vec::from(event.role),
                            sender: event.sender,
                        });
                    }

                    None
                })
        })
        .collect());
    events.pool_role_revokeds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == POOL_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::pool_contract::events::RoleRevoked::match_and_decode(log) {
                        return Some(contract::PoolRoleRevoked {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            account: event.account,
                            role: Vec::from(event.role),
                            sender: event.sender,
                        });
                    }

                    None
                })
        })
        .collect());
    events.pool_tokens_tradeds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == POOL_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::pool_contract::events::TokensTraded::match_and_decode(log) {
                        return Some(contract::PoolTokensTraded {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            bnt_amount: event.bnt_amount.to_string(),
                            bnt_fee_amount: event.bnt_fee_amount.to_string(),
                            context_id: Vec::from(event.context_id),
                            source_amount: event.source_amount.to_string(),
                            source_token: event.source_token,
                            target_amount: event.target_amount.to_string(),
                            target_fee_amount: event.target_fee_amount.to_string(),
                            target_token: event.target_token,
                            trader: event.trader,
                        });
                    }

                    None
                })
        })
        .collect());
    events.pool_unpauseds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == POOL_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::pool_contract::events::Unpaused::match_and_decode(log) {
                        return Some(contract::PoolUnpaused {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            account: event.account,
                        });
                    }

                    None
                })
        })
        .collect());
    events.pool_upgradeds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == POOL_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::pool_contract::events::Upgraded::match_and_decode(log) {
                        return Some(contract::PoolUpgraded {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            implementation: event.implementation,
                        });
                    }

                    None
                })
        })
        .collect());
}

fn db_pool_out(events: &contract::Events, tables: &mut DatabaseChangeTables) {
    // Loop over all the abis events to create table changes
    events.pool_admin_changeds.iter().for_each(|evt| {
        tables
            .create_row("pool_admin_changed", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("new_admin", Hex(&evt.new_admin).to_string())
            .set("previous_admin", Hex(&evt.previous_admin).to_string());
    });
    events.pool_beacon_upgradeds.iter().for_each(|evt| {
        tables
            .create_row("pool_beacon_upgraded", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("beacon", Hex(&evt.beacon).to_string());
    });
    events.pool_flash_loan_completeds.iter().for_each(|evt| {
        tables
            .create_row("pool_flash_loan_completed", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("amount", BigDecimal::from_str(&evt.amount).unwrap())
            .set("borrower", Hex(&evt.borrower).to_string())
            .set("fee_amount", BigDecimal::from_str(&evt.fee_amount).unwrap())
            .set("token", Hex(&evt.token).to_string());
    });
    events.pool_funds_migrateds.iter().for_each(|evt| {
        tables
            .create_row("pool_funds_migrated", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("amount", BigDecimal::from_str(&evt.amount).unwrap())
            .set("available_amount", BigDecimal::from_str(&evt.available_amount).unwrap())
            .set("context_id", Hex(&evt.context_id).to_string())
            .set("original_amount", BigDecimal::from_str(&evt.original_amount).unwrap())
            .set("provider", Hex(&evt.provider).to_string())
            .set("token", Hex(&evt.token).to_string());
    });
    events.pool_min_network_fee_burn_updateds.iter().for_each(|evt| {
        tables
            .create_row("pool_min_network_fee_burn_updated", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("new_min_network_fee_burn", BigDecimal::from_str(&evt.new_min_network_fee_burn).unwrap())
            .set("old_min_network_fee_burn", BigDecimal::from_str(&evt.old_min_network_fee_burn).unwrap());
    });
    events.pool_network_fees_burneds.iter().for_each(|evt| {
        tables
            .create_row("pool_network_fees_burned", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("amount", BigDecimal::from_str(&evt.amount).unwrap())
            .set("caller", Hex(&evt.caller).to_string());
    });
    events.pool_pol_rewards_ppm_updateds.iter().for_each(|evt| {
        tables
            .create_row("pool_pol_rewards_ppm_updated", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("new_rewards_ppm", evt.new_rewards_ppm)
            .set("old_rewards_ppm", evt.old_rewards_ppm);
    });
    events.pool_pol_withdrawns.iter().for_each(|evt| {
        tables
            .create_row("pool_pol_withdrawn", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("caller", Hex(&evt.caller).to_string())
            .set("pol_token_amount", BigDecimal::from_str(&evt.pol_token_amount).unwrap())
            .set("token", Hex(&evt.token).to_string())
            .set("user_reward", BigDecimal::from_str(&evt.user_reward).unwrap());
    });
    events.pool_pauseds.iter().for_each(|evt| {
        tables
            .create_row("pool_paused", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("account", Hex(&evt.account).to_string());
    });
    events.pool_pool_addeds.iter().for_each(|evt| {
        tables
            .create_row("pool_pool_added", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("pool", Hex(&evt.pool).to_string())
            .set("pool_collection", Hex(&evt.pool_collection).to_string());
    });
    events.pool_pool_collection_addeds.iter().for_each(|evt| {
        tables
            .create_row("pool_pool_collection_added", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("pool_collection", Hex(&evt.pool_collection).to_string())
            .set("pool_type", evt.pool_type);
    });
    events.pool_pool_collection_removeds.iter().for_each(|evt| {
        tables
            .create_row("pool_pool_collection_removed", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("pool_collection", Hex(&evt.pool_collection).to_string())
            .set("pool_type", evt.pool_type);
    });
    events.pool_pool_createds.iter().for_each(|evt| {
        tables
            .create_row("pool_pool_created", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("pool", Hex(&evt.pool).to_string())
            .set("pool_collection", Hex(&evt.pool_collection).to_string());
    });
    events.pool_pool_removeds.iter().for_each(|evt| {
        tables
            .create_row("pool_pool_removed", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("pool", Hex(&evt.pool).to_string())
            .set("pool_collection", Hex(&evt.pool_collection).to_string());
    });
    events.pool_role_admin_changeds.iter().for_each(|evt| {
        tables
            .create_row("pool_role_admin_changed", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("new_admin_role", Hex(&evt.new_admin_role).to_string())
            .set("previous_admin_role", Hex(&evt.previous_admin_role).to_string())
            .set("role", Hex(&evt.role).to_string());
    });
    events.pool_role_granteds.iter().for_each(|evt| {
        tables
            .create_row("pool_role_granted", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("account", Hex(&evt.account).to_string())
            .set("role", Hex(&evt.role).to_string())
            .set("sender", Hex(&evt.sender).to_string());
    });
    events.pool_role_revokeds.iter().for_each(|evt| {
        tables
            .create_row("pool_role_revoked", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("account", Hex(&evt.account).to_string())
            .set("role", Hex(&evt.role).to_string())
            .set("sender", Hex(&evt.sender).to_string());
    });
    events.pool_tokens_tradeds.iter().for_each(|evt| {
        tables
            .create_row("pool_tokens_traded", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("bnt_amount", BigDecimal::from_str(&evt.bnt_amount).unwrap())
            .set("bnt_fee_amount", BigDecimal::from_str(&evt.bnt_fee_amount).unwrap())
            .set("context_id", Hex(&evt.context_id).to_string())
            .set("source_amount", BigDecimal::from_str(&evt.source_amount).unwrap())
            .set("source_token", Hex(&evt.source_token).to_string())
            .set("target_amount", BigDecimal::from_str(&evt.target_amount).unwrap())
            .set("target_fee_amount", BigDecimal::from_str(&evt.target_fee_amount).unwrap())
            .set("target_token", Hex(&evt.target_token).to_string())
            .set("trader", Hex(&evt.trader).to_string());
    });
    events.pool_unpauseds.iter().for_each(|evt| {
        tables
            .create_row("pool_unpaused", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("account", Hex(&evt.account).to_string());
    });
    events.pool_upgradeds.iter().for_each(|evt| {
        tables
            .create_row("pool_upgraded", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("implementation", Hex(&evt.implementation).to_string());
    });
}


fn graph_pool_out(events: &contract::Events, tables: &mut EntityChangesTables) {
    // Loop over all the abis events to create table changes
    events.pool_admin_changeds.iter().for_each(|evt| {
        tables
            .create_row("pool_admin_changed", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("new_admin", Hex(&evt.new_admin).to_string())
            .set("previous_admin", Hex(&evt.previous_admin).to_string());
    });
    events.pool_beacon_upgradeds.iter().for_each(|evt| {
        tables
            .create_row("pool_beacon_upgraded", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("beacon", Hex(&evt.beacon).to_string());
    });
    events.pool_flash_loan_completeds.iter().for_each(|evt| {
        tables
            .create_row("pool_flash_loan_completed", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("amount", BigDecimal::from_str(&evt.amount).unwrap())
            .set("borrower", Hex(&evt.borrower).to_string())
            .set("fee_amount", BigDecimal::from_str(&evt.fee_amount).unwrap())
            .set("token", Hex(&evt.token).to_string());
    });
    events.pool_funds_migrateds.iter().for_each(|evt| {
        tables
            .create_row("pool_funds_migrated", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("amount", BigDecimal::from_str(&evt.amount).unwrap())
            .set("available_amount", BigDecimal::from_str(&evt.available_amount).unwrap())
            .set("context_id", Hex(&evt.context_id).to_string())
            .set("original_amount", BigDecimal::from_str(&evt.original_amount).unwrap())
            .set("provider", Hex(&evt.provider).to_string())
            .set("token", Hex(&evt.token).to_string());
    });
    events.pool_min_network_fee_burn_updateds.iter().for_each(|evt| {
        tables
            .create_row("pool_min_network_fee_burn_updated", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("new_min_network_fee_burn", BigDecimal::from_str(&evt.new_min_network_fee_burn).unwrap())
            .set("old_min_network_fee_burn", BigDecimal::from_str(&evt.old_min_network_fee_burn).unwrap());
    });
    events.pool_network_fees_burneds.iter().for_each(|evt| {
        tables
            .create_row("pool_network_fees_burned", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("amount", BigDecimal::from_str(&evt.amount).unwrap())
            .set("caller", Hex(&evt.caller).to_string());
    });
    events.pool_pol_rewards_ppm_updateds.iter().for_each(|evt| {
        tables
            .create_row("pool_pol_rewards_ppm_updated", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("new_rewards_ppm", evt.new_rewards_ppm)
            .set("old_rewards_ppm", evt.old_rewards_ppm);
    });
    events.pool_pol_withdrawns.iter().for_each(|evt| {
        tables
            .create_row("pool_pol_withdrawn", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("caller", Hex(&evt.caller).to_string())
            .set("pol_token_amount", BigDecimal::from_str(&evt.pol_token_amount).unwrap())
            .set("token", Hex(&evt.token).to_string())
            .set("user_reward", BigDecimal::from_str(&evt.user_reward).unwrap());
    });
    events.pool_pauseds.iter().for_each(|evt| {
        tables
            .create_row("pool_paused", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("account", Hex(&evt.account).to_string());
    });
    events.pool_pool_addeds.iter().for_each(|evt| {
        tables
            .create_row("pool_pool_added", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("pool", Hex(&evt.pool).to_string())
            .set("pool_collection", Hex(&evt.pool_collection).to_string());
    });
    events.pool_pool_collection_addeds.iter().for_each(|evt| {
        tables
            .create_row("pool_pool_collection_added", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("pool_collection", Hex(&evt.pool_collection).to_string())
            .set("pool_type", evt.pool_type);
    });
    events.pool_pool_collection_removeds.iter().for_each(|evt| {
        tables
            .create_row("pool_pool_collection_removed", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("pool_collection", Hex(&evt.pool_collection).to_string())
            .set("pool_type", evt.pool_type);
    });
    events.pool_pool_createds.iter().for_each(|evt| {
        tables
            .create_row("pool_pool_created", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("pool", Hex(&evt.pool).to_string())
            .set("pool_collection", Hex(&evt.pool_collection).to_string());
    });
    events.pool_pool_removeds.iter().for_each(|evt| {
        tables
            .create_row("pool_pool_removed", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("pool", Hex(&evt.pool).to_string())
            .set("pool_collection", Hex(&evt.pool_collection).to_string());
    });
    events.pool_role_admin_changeds.iter().for_each(|evt| {
        tables
            .create_row("pool_role_admin_changed", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("new_admin_role", Hex(&evt.new_admin_role).to_string())
            .set("previous_admin_role", Hex(&evt.previous_admin_role).to_string())
            .set("role", Hex(&evt.role).to_string());
    });
    events.pool_role_granteds.iter().for_each(|evt| {
        tables
            .create_row("pool_role_granted", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("account", Hex(&evt.account).to_string())
            .set("role", Hex(&evt.role).to_string())
            .set("sender", Hex(&evt.sender).to_string());
    });
    events.pool_role_revokeds.iter().for_each(|evt| {
        tables
            .create_row("pool_role_revoked", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("account", Hex(&evt.account).to_string())
            .set("role", Hex(&evt.role).to_string())
            .set("sender", Hex(&evt.sender).to_string());
    });
    events.pool_tokens_tradeds.iter().for_each(|evt| {
        tables
            .create_row("pool_tokens_traded", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("bnt_amount", BigDecimal::from_str(&evt.bnt_amount).unwrap())
            .set("bnt_fee_amount", BigDecimal::from_str(&evt.bnt_fee_amount).unwrap())
            .set("context_id", Hex(&evt.context_id).to_string())
            .set("source_amount", BigDecimal::from_str(&evt.source_amount).unwrap())
            .set("source_token", Hex(&evt.source_token).to_string())
            .set("target_amount", BigDecimal::from_str(&evt.target_amount).unwrap())
            .set("target_fee_amount", BigDecimal::from_str(&evt.target_fee_amount).unwrap())
            .set("target_token", Hex(&evt.target_token).to_string())
            .set("trader", Hex(&evt.trader).to_string());
    });
    events.pool_unpauseds.iter().for_each(|evt| {
        tables
            .create_row("pool_unpaused", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("account", Hex(&evt.account).to_string());
    });
    events.pool_upgradeds.iter().for_each(|evt| {
        tables
            .create_row("pool_upgraded", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("implementation", Hex(&evt.implementation).to_string());
    });
}

#[substreams::handlers::map]
fn map_events(blk: eth::Block) -> Result<contract::Events, substreams::errors::Error> {
    let mut events = contract::Events::default();
    map_pool_events(&blk, &mut events);
    Ok(events)
}

#[substreams::handlers::map]
fn db_out(events: contract::Events) -> Result<DatabaseChanges, substreams::errors::Error> {
    // Initialize Database Changes container
    let mut tables = DatabaseChangeTables::new();
    db_pool_out(&events, &mut tables);
    Ok(tables.to_database_changes())
}

#[substreams::handlers::map]
fn graph_out(events: contract::Events) -> Result<EntityChanges, substreams::errors::Error> {
    // Initialize Database Changes container
    let mut tables = EntityChangesTables::new();
    graph_pool_out(&events, &mut tables);
    Ok(tables.to_entity_changes())
}
