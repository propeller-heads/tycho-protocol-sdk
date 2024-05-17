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

const STAKEDFRAX_TRACKED_CONTRACT: [u8; 20] = hex!("a663b02cf0a4b149d2ad41910cb81e23e1c41c32");

fn map_stakedfrax_events(blk: &eth::Block, events: &mut contract::Events) {
    events.stakedfrax_approvals.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == STAKEDFRAX_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::stakedfrax_contract::events::Approval::match_and_decode(log) {
                        return Some(contract::StakedfraxApproval {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            amount: event.amount.to_string(),
                            owner: event.owner,
                            spender: event.spender,
                        });
                    }

                    None
                })
        })
        .collect());
    events.stakedfrax_deposits.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == STAKEDFRAX_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::stakedfrax_contract::events::Deposit::match_and_decode(log) {
                        return Some(contract::StakedfraxDeposit {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            assets: event.assets.to_string(),
                            caller: event.caller,
                            owner: event.owner,
                            shares: event.shares.to_string(),
                        });
                    }

                    None
                })
        })
        .collect());
    events.stakedfrax_distribute_rewards.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == STAKEDFRAX_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::stakedfrax_contract::events::DistributeRewards::match_and_decode(log) {
                        return Some(contract::StakedfraxDistributeRewards {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            rewards_to_distribute: event.rewards_to_distribute.to_string(),
                        });
                    }

                    None
                })
        })
        .collect());
    events.stakedfrax_set_max_distribution_per_second_per_assets.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == STAKEDFRAX_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::stakedfrax_contract::events::SetMaxDistributionPerSecondPerAsset::match_and_decode(log) {
                        return Some(contract::StakedfraxSetMaxDistributionPerSecondPerAsset {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            new_max: event.new_max.to_string(),
                            old_max: event.old_max.to_string(),
                        });
                    }

                    None
                })
        })
        .collect());
    events.stakedfrax_sync_rewards.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == STAKEDFRAX_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::stakedfrax_contract::events::SyncRewards::match_and_decode(log) {
                        return Some(contract::StakedfraxSyncRewards {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            cycle_end: event.cycle_end.to_u64(),
                            last_sync: event.last_sync.to_u64(),
                            reward_cycle_amount: event.reward_cycle_amount.to_string(),
                        });
                    }

                    None
                })
        })
        .collect());
    events.stakedfrax_timelock_transfer_starteds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == STAKEDFRAX_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::stakedfrax_contract::events::TimelockTransferStarted::match_and_decode(log) {
                        return Some(contract::StakedfraxTimelockTransferStarted {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            new_timelock: event.new_timelock,
                            previous_timelock: event.previous_timelock,
                        });
                    }

                    None
                })
        })
        .collect());
    events.stakedfrax_timelock_transferreds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == STAKEDFRAX_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::stakedfrax_contract::events::TimelockTransferred::match_and_decode(log) {
                        return Some(contract::StakedfraxTimelockTransferred {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            new_timelock: event.new_timelock,
                            previous_timelock: event.previous_timelock,
                        });
                    }

                    None
                })
        })
        .collect());
    events.stakedfrax_transfers.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == STAKEDFRAX_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::stakedfrax_contract::events::Transfer::match_and_decode(log) {
                        return Some(contract::StakedfraxTransfer {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            amount: event.amount.to_string(),
                            from: event.from,
                            to: event.to,
                        });
                    }

                    None
                })
        })
        .collect());
    events.stakedfrax_withdraws.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == STAKEDFRAX_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::stakedfrax_contract::events::Withdraw::match_and_decode(log) {
                        return Some(contract::StakedfraxWithdraw {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            assets: event.assets.to_string(),
                            caller: event.caller,
                            owner: event.owner,
                            receiver: event.receiver,
                            shares: event.shares.to_string(),
                        });
                    }

                    None
                })
        })
        .collect());
}

fn map_stakedfrax_calls(blk: &eth::Block, calls: &mut contract::Calls) {
    calls.stakedfrax_call_accept_transfer_timelocks.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == STAKEDFRAX_TRACKED_CONTRACT && abi::stakedfrax_contract::functions::AcceptTransferTimelock::match_call(call))
                .filter_map(|call| {
                    match abi::stakedfrax_contract::functions::AcceptTransferTimelock::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::StakedfraxAcceptTransferTimelockCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.stakedfrax_call_approves.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == STAKEDFRAX_TRACKED_CONTRACT && abi::stakedfrax_contract::functions::Approve::match_call(call))
                .filter_map(|call| {
                    match abi::stakedfrax_contract::functions::Approve::decode(call) {
                        Ok(decoded_call) => {
                            let output_param0 = match abi::stakedfrax_contract::functions::Approve::output(&call.return_data) {
                                Ok(output_param0) => {output_param0}
                                Err(_) => Default::default(),
                            };
                            
                            Some(contract::StakedfraxApproveCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                amount: decoded_call.amount.to_string(),
                                output_param0: output_param0,
                                spender: decoded_call.spender,
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.stakedfrax_call_deposits.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == STAKEDFRAX_TRACKED_CONTRACT && abi::stakedfrax_contract::functions::Deposit::match_call(call))
                .filter_map(|call| {
                    match abi::stakedfrax_contract::functions::Deposit::decode(call) {
                        Ok(decoded_call) => {
                            let output__shares = match abi::stakedfrax_contract::functions::Deposit::output(&call.return_data) {
                                Ok(output__shares) => {output__shares}
                                Err(_) => Default::default(),
                            };
                            
                            Some(contract::StakedfraxDepositCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                output__shares: output__shares.to_string(),
                                u_assets: decoded_call.u_assets.to_string(),
                                u_receiver: decoded_call.u_receiver,
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.stakedfrax_call_deposit_with_signatures.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == STAKEDFRAX_TRACKED_CONTRACT && abi::stakedfrax_contract::functions::DepositWithSignature::match_call(call))
                .filter_map(|call| {
                    match abi::stakedfrax_contract::functions::DepositWithSignature::decode(call) {
                        Ok(decoded_call) => {
                            let output__shares = match abi::stakedfrax_contract::functions::DepositWithSignature::output(&call.return_data) {
                                Ok(output__shares) => {output__shares}
                                Err(_) => Default::default(),
                            };
                            
                            Some(contract::StakedfraxDepositWithSignatureCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                output__shares: output__shares.to_string(),
                                u_approve_max: decoded_call.u_approve_max,
                                u_assets: decoded_call.u_assets.to_string(),
                                u_deadline: decoded_call.u_deadline.to_string(),
                                u_r: Vec::from(decoded_call.u_r),
                                u_receiver: decoded_call.u_receiver,
                                u_s: Vec::from(decoded_call.u_s),
                                u_v: decoded_call.u_v.to_u64(),
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.stakedfrax_call_mints.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == STAKEDFRAX_TRACKED_CONTRACT && abi::stakedfrax_contract::functions::Mint::match_call(call))
                .filter_map(|call| {
                    match abi::stakedfrax_contract::functions::Mint::decode(call) {
                        Ok(decoded_call) => {
                            let output__assets = match abi::stakedfrax_contract::functions::Mint::output(&call.return_data) {
                                Ok(output__assets) => {output__assets}
                                Err(_) => Default::default(),
                            };
                            
                            Some(contract::StakedfraxMintCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                output__assets: output__assets.to_string(),
                                u_receiver: decoded_call.u_receiver,
                                u_shares: decoded_call.u_shares.to_string(),
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.stakedfrax_call_permits.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == STAKEDFRAX_TRACKED_CONTRACT && abi::stakedfrax_contract::functions::Permit::match_call(call))
                .filter_map(|call| {
                    match abi::stakedfrax_contract::functions::Permit::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::StakedfraxPermitCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                deadline: decoded_call.deadline.to_string(),
                                owner: decoded_call.owner,
                                r: Vec::from(decoded_call.r),
                                s: Vec::from(decoded_call.s),
                                spender: decoded_call.spender,
                                v: decoded_call.v.to_u64(),
                                value: decoded_call.value.to_string(),
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.stakedfrax_call_redeems.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == STAKEDFRAX_TRACKED_CONTRACT && abi::stakedfrax_contract::functions::Redeem::match_call(call))
                .filter_map(|call| {
                    match abi::stakedfrax_contract::functions::Redeem::decode(call) {
                        Ok(decoded_call) => {
                            let output__assets = match abi::stakedfrax_contract::functions::Redeem::output(&call.return_data) {
                                Ok(output__assets) => {output__assets}
                                Err(_) => Default::default(),
                            };
                            
                            Some(contract::StakedfraxRedeemCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                output__assets: output__assets.to_string(),
                                u_owner: decoded_call.u_owner,
                                u_receiver: decoded_call.u_receiver,
                                u_shares: decoded_call.u_shares.to_string(),
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.stakedfrax_call_renounce_timelocks.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == STAKEDFRAX_TRACKED_CONTRACT && abi::stakedfrax_contract::functions::RenounceTimelock::match_call(call))
                .filter_map(|call| {
                    match abi::stakedfrax_contract::functions::RenounceTimelock::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::StakedfraxRenounceTimelockCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.stakedfrax_call_set_max_distribution_per_second_per_assets.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == STAKEDFRAX_TRACKED_CONTRACT && abi::stakedfrax_contract::functions::SetMaxDistributionPerSecondPerAsset::match_call(call))
                .filter_map(|call| {
                    match abi::stakedfrax_contract::functions::SetMaxDistributionPerSecondPerAsset::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::StakedfraxSetMaxDistributionPerSecondPerAssetCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                u_max_distribution_per_second_per_asset: decoded_call.u_max_distribution_per_second_per_asset.to_string(),
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.stakedfrax_call_sync_rewards_and_distributions.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == STAKEDFRAX_TRACKED_CONTRACT && abi::stakedfrax_contract::functions::SyncRewardsAndDistribution::match_call(call))
                .filter_map(|call| {
                    match abi::stakedfrax_contract::functions::SyncRewardsAndDistribution::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::StakedfraxSyncRewardsAndDistributionCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.stakedfrax_call_transfers.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == STAKEDFRAX_TRACKED_CONTRACT && abi::stakedfrax_contract::functions::Transfer::match_call(call))
                .filter_map(|call| {
                    match abi::stakedfrax_contract::functions::Transfer::decode(call) {
                        Ok(decoded_call) => {
                            let output_param0 = match abi::stakedfrax_contract::functions::Transfer::output(&call.return_data) {
                                Ok(output_param0) => {output_param0}
                                Err(_) => Default::default(),
                            };
                            
                            Some(contract::StakedfraxTransferCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                amount: decoded_call.amount.to_string(),
                                output_param0: output_param0,
                                to: decoded_call.to,
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.stakedfrax_call_transfer_froms.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == STAKEDFRAX_TRACKED_CONTRACT && abi::stakedfrax_contract::functions::TransferFrom::match_call(call))
                .filter_map(|call| {
                    match abi::stakedfrax_contract::functions::TransferFrom::decode(call) {
                        Ok(decoded_call) => {
                            let output_param0 = match abi::stakedfrax_contract::functions::TransferFrom::output(&call.return_data) {
                                Ok(output_param0) => {output_param0}
                                Err(_) => Default::default(),
                            };
                            
                            Some(contract::StakedfraxTransferFromCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                amount: decoded_call.amount.to_string(),
                                from: decoded_call.from,
                                output_param0: output_param0,
                                to: decoded_call.to,
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.stakedfrax_call_transfer_timelocks.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == STAKEDFRAX_TRACKED_CONTRACT && abi::stakedfrax_contract::functions::TransferTimelock::match_call(call))
                .filter_map(|call| {
                    match abi::stakedfrax_contract::functions::TransferTimelock::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::StakedfraxTransferTimelockCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                u_new_timelock: decoded_call.u_new_timelock,
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.stakedfrax_call_withdraws.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == STAKEDFRAX_TRACKED_CONTRACT && abi::stakedfrax_contract::functions::Withdraw::match_call(call))
                .filter_map(|call| {
                    match abi::stakedfrax_contract::functions::Withdraw::decode(call) {
                        Ok(decoded_call) => {
                            let output__shares = match abi::stakedfrax_contract::functions::Withdraw::output(&call.return_data) {
                                Ok(output__shares) => {output__shares}
                                Err(_) => Default::default(),
                            };
                            
                            Some(contract::StakedfraxWithdrawCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                output__shares: output__shares.to_string(),
                                u_assets: decoded_call.u_assets.to_string(),
                                u_owner: decoded_call.u_owner,
                                u_receiver: decoded_call.u_receiver,
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
}

fn db_stakedfrax_out(events: &contract::Events, tables: &mut DatabaseChangeTables) {
    // Loop over all the abis events to create table changes
    events.stakedfrax_approvals.iter().for_each(|evt| {
        tables
            .create_row("stakedfrax_approval", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("amount", BigDecimal::from_str(&evt.amount).unwrap())
            .set("owner", Hex(&evt.owner).to_string())
            .set("spender", Hex(&evt.spender).to_string());
    });
    events.stakedfrax_deposits.iter().for_each(|evt| {
        tables
            .create_row("stakedfrax_deposit", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("assets", BigDecimal::from_str(&evt.assets).unwrap())
            .set("caller", Hex(&evt.caller).to_string())
            .set("owner", Hex(&evt.owner).to_string())
            .set("shares", BigDecimal::from_str(&evt.shares).unwrap());
    });
    events.stakedfrax_distribute_rewards.iter().for_each(|evt| {
        tables
            .create_row("stakedfrax_distribute_rewards", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("rewards_to_distribute", BigDecimal::from_str(&evt.rewards_to_distribute).unwrap());
    });
    events.stakedfrax_set_max_distribution_per_second_per_assets.iter().for_each(|evt| {
        tables
            .create_row("stakedfrax_set_max_distribution_per_second_per_asset", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("new_max", BigDecimal::from_str(&evt.new_max).unwrap())
            .set("old_max", BigDecimal::from_str(&evt.old_max).unwrap());
    });
    events.stakedfrax_sync_rewards.iter().for_each(|evt| {
        tables
            .create_row("stakedfrax_sync_rewards", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("cycle_end", evt.cycle_end)
            .set("last_sync", evt.last_sync)
            .set("reward_cycle_amount", BigDecimal::from_str(&evt.reward_cycle_amount).unwrap());
    });
    events.stakedfrax_timelock_transfer_starteds.iter().for_each(|evt| {
        tables
            .create_row("stakedfrax_timelock_transfer_started", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("new_timelock", Hex(&evt.new_timelock).to_string())
            .set("previous_timelock", Hex(&evt.previous_timelock).to_string());
    });
    events.stakedfrax_timelock_transferreds.iter().for_each(|evt| {
        tables
            .create_row("stakedfrax_timelock_transferred", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("new_timelock", Hex(&evt.new_timelock).to_string())
            .set("previous_timelock", Hex(&evt.previous_timelock).to_string());
    });
    events.stakedfrax_transfers.iter().for_each(|evt| {
        tables
            .create_row("stakedfrax_transfer", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("amount", BigDecimal::from_str(&evt.amount).unwrap())
            .set("from", Hex(&evt.from).to_string())
            .set("to", Hex(&evt.to).to_string());
    });
    events.stakedfrax_withdraws.iter().for_each(|evt| {
        tables
            .create_row("stakedfrax_withdraw", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("assets", BigDecimal::from_str(&evt.assets).unwrap())
            .set("caller", Hex(&evt.caller).to_string())
            .set("owner", Hex(&evt.owner).to_string())
            .set("receiver", Hex(&evt.receiver).to_string())
            .set("shares", BigDecimal::from_str(&evt.shares).unwrap());
    });
}
fn db_stakedfrax_calls_out(calls: &contract::Calls, tables: &mut DatabaseChangeTables) {
    // Loop over all the abis calls to create table changes
    calls.stakedfrax_call_accept_transfer_timelocks.iter().for_each(|call| {
        tables
            .create_row("stakedfrax_call_accept_transfer_timelock", [("call_tx_hash", call.call_tx_hash.to_string()),("call_ordinal", call.call_ordinal.to_string())])
            .set("call_block_time", call.call_block_time.as_ref().unwrap())
            .set("call_block_number", call.call_block_number)
            .set("call_success", call.call_success);
    });
    calls.stakedfrax_call_approves.iter().for_each(|call| {
        tables
            .create_row("stakedfrax_call_approve", [("call_tx_hash", call.call_tx_hash.to_string()),("call_ordinal", call.call_ordinal.to_string())])
            .set("call_block_time", call.call_block_time.as_ref().unwrap())
            .set("call_block_number", call.call_block_number)
            .set("call_success", call.call_success)
            .set("amount", BigDecimal::from_str(&call.amount).unwrap())
            .set("output_param0", call.output_param0)
            .set("spender", Hex(&call.spender).to_string());
    });
    calls.stakedfrax_call_deposits.iter().for_each(|call| {
        tables
            .create_row("stakedfrax_call_deposit", [("call_tx_hash", call.call_tx_hash.to_string()),("call_ordinal", call.call_ordinal.to_string())])
            .set("call_block_time", call.call_block_time.as_ref().unwrap())
            .set("call_block_number", call.call_block_number)
            .set("call_success", call.call_success)
            .set("output__shares", BigDecimal::from_str(&call.output__shares).unwrap())
            .set("u_assets", BigDecimal::from_str(&call.u_assets).unwrap())
            .set("u_receiver", Hex(&call.u_receiver).to_string());
    });
    calls.stakedfrax_call_deposit_with_signatures.iter().for_each(|call| {
        tables
            .create_row("stakedfrax_call_deposit_with_signature", [("call_tx_hash", call.call_tx_hash.to_string()),("call_ordinal", call.call_ordinal.to_string())])
            .set("call_block_time", call.call_block_time.as_ref().unwrap())
            .set("call_block_number", call.call_block_number)
            .set("call_success", call.call_success)
            .set("output__shares", BigDecimal::from_str(&call.output__shares).unwrap())
            .set("u_approve_max", call.u_approve_max)
            .set("u_assets", BigDecimal::from_str(&call.u_assets).unwrap())
            .set("u_deadline", BigDecimal::from_str(&call.u_deadline).unwrap())
            .set("u_r", Hex(&call.u_r).to_string())
            .set("u_receiver", Hex(&call.u_receiver).to_string())
            .set("u_s", Hex(&call.u_s).to_string())
            .set("u_v", call.u_v);
    });
    calls.stakedfrax_call_mints.iter().for_each(|call| {
        tables
            .create_row("stakedfrax_call_mint", [("call_tx_hash", call.call_tx_hash.to_string()),("call_ordinal", call.call_ordinal.to_string())])
            .set("call_block_time", call.call_block_time.as_ref().unwrap())
            .set("call_block_number", call.call_block_number)
            .set("call_success", call.call_success)
            .set("output__assets", BigDecimal::from_str(&call.output__assets).unwrap())
            .set("u_receiver", Hex(&call.u_receiver).to_string())
            .set("u_shares", BigDecimal::from_str(&call.u_shares).unwrap());
    });
    calls.stakedfrax_call_permits.iter().for_each(|call| {
        tables
            .create_row("stakedfrax_call_permit", [("call_tx_hash", call.call_tx_hash.to_string()),("call_ordinal", call.call_ordinal.to_string())])
            .set("call_block_time", call.call_block_time.as_ref().unwrap())
            .set("call_block_number", call.call_block_number)
            .set("call_success", call.call_success)
            .set("deadline", BigDecimal::from_str(&call.deadline).unwrap())
            .set("owner", Hex(&call.owner).to_string())
            .set("r", Hex(&call.r).to_string())
            .set("s", Hex(&call.s).to_string())
            .set("spender", Hex(&call.spender).to_string())
            .set("v", call.v)
            .set("value", BigDecimal::from_str(&call.value).unwrap());
    });
    calls.stakedfrax_call_redeems.iter().for_each(|call| {
        tables
            .create_row("stakedfrax_call_redeem", [("call_tx_hash", call.call_tx_hash.to_string()),("call_ordinal", call.call_ordinal.to_string())])
            .set("call_block_time", call.call_block_time.as_ref().unwrap())
            .set("call_block_number", call.call_block_number)
            .set("call_success", call.call_success)
            .set("output__assets", BigDecimal::from_str(&call.output__assets).unwrap())
            .set("u_owner", Hex(&call.u_owner).to_string())
            .set("u_receiver", Hex(&call.u_receiver).to_string())
            .set("u_shares", BigDecimal::from_str(&call.u_shares).unwrap());
    });
    calls.stakedfrax_call_renounce_timelocks.iter().for_each(|call| {
        tables
            .create_row("stakedfrax_call_renounce_timelock", [("call_tx_hash", call.call_tx_hash.to_string()),("call_ordinal", call.call_ordinal.to_string())])
            .set("call_block_time", call.call_block_time.as_ref().unwrap())
            .set("call_block_number", call.call_block_number)
            .set("call_success", call.call_success);
    });
    calls.stakedfrax_call_set_max_distribution_per_second_per_assets.iter().for_each(|call| {
        tables
            .create_row("stakedfrax_call_set_max_distribution_per_second_per_asset", [("call_tx_hash", call.call_tx_hash.to_string()),("call_ordinal", call.call_ordinal.to_string())])
            .set("call_block_time", call.call_block_time.as_ref().unwrap())
            .set("call_block_number", call.call_block_number)
            .set("call_success", call.call_success)
            .set("u_max_distribution_per_second_per_asset", BigDecimal::from_str(&call.u_max_distribution_per_second_per_asset).unwrap());
    });
    calls.stakedfrax_call_sync_rewards_and_distributions.iter().for_each(|call| {
        tables
            .create_row("stakedfrax_call_sync_rewards_and_distribution", [("call_tx_hash", call.call_tx_hash.to_string()),("call_ordinal", call.call_ordinal.to_string())])
            .set("call_block_time", call.call_block_time.as_ref().unwrap())
            .set("call_block_number", call.call_block_number)
            .set("call_success", call.call_success);
    });
    calls.stakedfrax_call_transfers.iter().for_each(|call| {
        tables
            .create_row("stakedfrax_call_transfer", [("call_tx_hash", call.call_tx_hash.to_string()),("call_ordinal", call.call_ordinal.to_string())])
            .set("call_block_time", call.call_block_time.as_ref().unwrap())
            .set("call_block_number", call.call_block_number)
            .set("call_success", call.call_success)
            .set("amount", BigDecimal::from_str(&call.amount).unwrap())
            .set("output_param0", call.output_param0)
            .set("to", Hex(&call.to).to_string());
    });
    calls.stakedfrax_call_transfer_froms.iter().for_each(|call| {
        tables
            .create_row("stakedfrax_call_transfer_from", [("call_tx_hash", call.call_tx_hash.to_string()),("call_ordinal", call.call_ordinal.to_string())])
            .set("call_block_time", call.call_block_time.as_ref().unwrap())
            .set("call_block_number", call.call_block_number)
            .set("call_success", call.call_success)
            .set("amount", BigDecimal::from_str(&call.amount).unwrap())
            .set("from", Hex(&call.from).to_string())
            .set("output_param0", call.output_param0)
            .set("to", Hex(&call.to).to_string());
    });
    calls.stakedfrax_call_transfer_timelocks.iter().for_each(|call| {
        tables
            .create_row("stakedfrax_call_transfer_timelock", [("call_tx_hash", call.call_tx_hash.to_string()),("call_ordinal", call.call_ordinal.to_string())])
            .set("call_block_time", call.call_block_time.as_ref().unwrap())
            .set("call_block_number", call.call_block_number)
            .set("call_success", call.call_success)
            .set("u_new_timelock", Hex(&call.u_new_timelock).to_string());
    });
    calls.stakedfrax_call_withdraws.iter().for_each(|call| {
        tables
            .create_row("stakedfrax_call_withdraw", [("call_tx_hash", call.call_tx_hash.to_string()),("call_ordinal", call.call_ordinal.to_string())])
            .set("call_block_time", call.call_block_time.as_ref().unwrap())
            .set("call_block_number", call.call_block_number)
            .set("call_success", call.call_success)
            .set("output__shares", BigDecimal::from_str(&call.output__shares).unwrap())
            .set("u_assets", BigDecimal::from_str(&call.u_assets).unwrap())
            .set("u_owner", Hex(&call.u_owner).to_string())
            .set("u_receiver", Hex(&call.u_receiver).to_string());
    });
}


fn graph_stakedfrax_out(events: &contract::Events, tables: &mut EntityChangesTables) {
    // Loop over all the abis events to create table changes
    events.stakedfrax_approvals.iter().for_each(|evt| {
        tables
            .create_row("stakedfrax_approval", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("amount", BigDecimal::from_str(&evt.amount).unwrap())
            .set("owner", Hex(&evt.owner).to_string())
            .set("spender", Hex(&evt.spender).to_string());
    });
    events.stakedfrax_deposits.iter().for_each(|evt| {
        tables
            .create_row("stakedfrax_deposit", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("assets", BigDecimal::from_str(&evt.assets).unwrap())
            .set("caller", Hex(&evt.caller).to_string())
            .set("owner", Hex(&evt.owner).to_string())
            .set("shares", BigDecimal::from_str(&evt.shares).unwrap());
    });
    events.stakedfrax_distribute_rewards.iter().for_each(|evt| {
        tables
            .create_row("stakedfrax_distribute_rewards", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("rewards_to_distribute", BigDecimal::from_str(&evt.rewards_to_distribute).unwrap());
    });
    events.stakedfrax_set_max_distribution_per_second_per_assets.iter().for_each(|evt| {
        tables
            .create_row("stakedfrax_set_max_distribution_per_second_per_asset", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("new_max", BigDecimal::from_str(&evt.new_max).unwrap())
            .set("old_max", BigDecimal::from_str(&evt.old_max).unwrap());
    });
    events.stakedfrax_sync_rewards.iter().for_each(|evt| {
        tables
            .create_row("stakedfrax_sync_rewards", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("cycle_end", evt.cycle_end)
            .set("last_sync", evt.last_sync)
            .set("reward_cycle_amount", BigDecimal::from_str(&evt.reward_cycle_amount).unwrap());
    });
    events.stakedfrax_timelock_transfer_starteds.iter().for_each(|evt| {
        tables
            .create_row("stakedfrax_timelock_transfer_started", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("new_timelock", Hex(&evt.new_timelock).to_string())
            .set("previous_timelock", Hex(&evt.previous_timelock).to_string());
    });
    events.stakedfrax_timelock_transferreds.iter().for_each(|evt| {
        tables
            .create_row("stakedfrax_timelock_transferred", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("new_timelock", Hex(&evt.new_timelock).to_string())
            .set("previous_timelock", Hex(&evt.previous_timelock).to_string());
    });
    events.stakedfrax_transfers.iter().for_each(|evt| {
        tables
            .create_row("stakedfrax_transfer", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("amount", BigDecimal::from_str(&evt.amount).unwrap())
            .set("from", Hex(&evt.from).to_string())
            .set("to", Hex(&evt.to).to_string());
    });
    events.stakedfrax_withdraws.iter().for_each(|evt| {
        tables
            .create_row("stakedfrax_withdraw", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("assets", BigDecimal::from_str(&evt.assets).unwrap())
            .set("caller", Hex(&evt.caller).to_string())
            .set("owner", Hex(&evt.owner).to_string())
            .set("receiver", Hex(&evt.receiver).to_string())
            .set("shares", BigDecimal::from_str(&evt.shares).unwrap());
    });
}
fn graph_stakedfrax_calls_out(calls: &contract::Calls, tables: &mut EntityChangesTables) {
    // Loop over all the abis calls to create table changes
    calls.stakedfrax_call_accept_transfer_timelocks.iter().for_each(|call| {
        tables
            .create_row("stakedfrax_call_accept_transfer_timelock", format!("{}-{}", call.call_tx_hash, call.call_ordinal))
            .set("call_tx_hash", &call.call_tx_hash)
            .set("call_ordinal", call.call_ordinal)
            .set("call_block_time", call.call_block_time.as_ref().unwrap())
            .set("call_block_number", call.call_block_number)
            .set("call_success", call.call_success);
    });
    calls.stakedfrax_call_approves.iter().for_each(|call| {
        tables
            .create_row("stakedfrax_call_approve", format!("{}-{}", call.call_tx_hash, call.call_ordinal))
            .set("call_tx_hash", &call.call_tx_hash)
            .set("call_ordinal", call.call_ordinal)
            .set("call_block_time", call.call_block_time.as_ref().unwrap())
            .set("call_block_number", call.call_block_number)
            .set("call_success", call.call_success)
            .set("amount", BigDecimal::from_str(&call.amount).unwrap())
            .set("output_param0", call.output_param0)
            .set("spender", Hex(&call.spender).to_string());
    });
    calls.stakedfrax_call_deposits.iter().for_each(|call| {
        tables
            .create_row("stakedfrax_call_deposit", format!("{}-{}", call.call_tx_hash, call.call_ordinal))
            .set("call_tx_hash", &call.call_tx_hash)
            .set("call_ordinal", call.call_ordinal)
            .set("call_block_time", call.call_block_time.as_ref().unwrap())
            .set("call_block_number", call.call_block_number)
            .set("call_success", call.call_success)
            .set("output__shares", BigDecimal::from_str(&call.output__shares).unwrap())
            .set("u_assets", BigDecimal::from_str(&call.u_assets).unwrap())
            .set("u_receiver", Hex(&call.u_receiver).to_string());
    });
    calls.stakedfrax_call_deposit_with_signatures.iter().for_each(|call| {
        tables
            .create_row("stakedfrax_call_deposit_with_signature", format!("{}-{}", call.call_tx_hash, call.call_ordinal))
            .set("call_tx_hash", &call.call_tx_hash)
            .set("call_ordinal", call.call_ordinal)
            .set("call_block_time", call.call_block_time.as_ref().unwrap())
            .set("call_block_number", call.call_block_number)
            .set("call_success", call.call_success)
            .set("output__shares", BigDecimal::from_str(&call.output__shares).unwrap())
            .set("u_approve_max", call.u_approve_max)
            .set("u_assets", BigDecimal::from_str(&call.u_assets).unwrap())
            .set("u_deadline", BigDecimal::from_str(&call.u_deadline).unwrap())
            .set("u_r", Hex(&call.u_r).to_string())
            .set("u_receiver", Hex(&call.u_receiver).to_string())
            .set("u_s", Hex(&call.u_s).to_string())
            .set("u_v", call.u_v);
    });
    calls.stakedfrax_call_mints.iter().for_each(|call| {
        tables
            .create_row("stakedfrax_call_mint", format!("{}-{}", call.call_tx_hash, call.call_ordinal))
            .set("call_tx_hash", &call.call_tx_hash)
            .set("call_ordinal", call.call_ordinal)
            .set("call_block_time", call.call_block_time.as_ref().unwrap())
            .set("call_block_number", call.call_block_number)
            .set("call_success", call.call_success)
            .set("output__assets", BigDecimal::from_str(&call.output__assets).unwrap())
            .set("u_receiver", Hex(&call.u_receiver).to_string())
            .set("u_shares", BigDecimal::from_str(&call.u_shares).unwrap());
    });
    calls.stakedfrax_call_permits.iter().for_each(|call| {
        tables
            .create_row("stakedfrax_call_permit", format!("{}-{}", call.call_tx_hash, call.call_ordinal))
            .set("call_tx_hash", &call.call_tx_hash)
            .set("call_ordinal", call.call_ordinal)
            .set("call_block_time", call.call_block_time.as_ref().unwrap())
            .set("call_block_number", call.call_block_number)
            .set("call_success", call.call_success)
            .set("deadline", BigDecimal::from_str(&call.deadline).unwrap())
            .set("owner", Hex(&call.owner).to_string())
            .set("r", Hex(&call.r).to_string())
            .set("s", Hex(&call.s).to_string())
            .set("spender", Hex(&call.spender).to_string())
            .set("v", call.v)
            .set("value", BigDecimal::from_str(&call.value).unwrap());
    });
    calls.stakedfrax_call_redeems.iter().for_each(|call| {
        tables
            .create_row("stakedfrax_call_redeem", format!("{}-{}", call.call_tx_hash, call.call_ordinal))
            .set("call_tx_hash", &call.call_tx_hash)
            .set("call_ordinal", call.call_ordinal)
            .set("call_block_time", call.call_block_time.as_ref().unwrap())
            .set("call_block_number", call.call_block_number)
            .set("call_success", call.call_success)
            .set("output__assets", BigDecimal::from_str(&call.output__assets).unwrap())
            .set("u_owner", Hex(&call.u_owner).to_string())
            .set("u_receiver", Hex(&call.u_receiver).to_string())
            .set("u_shares", BigDecimal::from_str(&call.u_shares).unwrap());
    });
    calls.stakedfrax_call_renounce_timelocks.iter().for_each(|call| {
        tables
            .create_row("stakedfrax_call_renounce_timelock", format!("{}-{}", call.call_tx_hash, call.call_ordinal))
            .set("call_tx_hash", &call.call_tx_hash)
            .set("call_ordinal", call.call_ordinal)
            .set("call_block_time", call.call_block_time.as_ref().unwrap())
            .set("call_block_number", call.call_block_number)
            .set("call_success", call.call_success);
    });
    calls.stakedfrax_call_set_max_distribution_per_second_per_assets.iter().for_each(|call| {
        tables
            .create_row("stakedfrax_call_set_max_distribution_per_second_per_asset", format!("{}-{}", call.call_tx_hash, call.call_ordinal))
            .set("call_tx_hash", &call.call_tx_hash)
            .set("call_ordinal", call.call_ordinal)
            .set("call_block_time", call.call_block_time.as_ref().unwrap())
            .set("call_block_number", call.call_block_number)
            .set("call_success", call.call_success)
            .set("u_max_distribution_per_second_per_asset", BigDecimal::from_str(&call.u_max_distribution_per_second_per_asset).unwrap());
    });
    calls.stakedfrax_call_sync_rewards_and_distributions.iter().for_each(|call| {
        tables
            .create_row("stakedfrax_call_sync_rewards_and_distribution", format!("{}-{}", call.call_tx_hash, call.call_ordinal))
            .set("call_tx_hash", &call.call_tx_hash)
            .set("call_ordinal", call.call_ordinal)
            .set("call_block_time", call.call_block_time.as_ref().unwrap())
            .set("call_block_number", call.call_block_number)
            .set("call_success", call.call_success);
    });
    calls.stakedfrax_call_transfers.iter().for_each(|call| {
        tables
            .create_row("stakedfrax_call_transfer", format!("{}-{}", call.call_tx_hash, call.call_ordinal))
            .set("call_tx_hash", &call.call_tx_hash)
            .set("call_ordinal", call.call_ordinal)
            .set("call_block_time", call.call_block_time.as_ref().unwrap())
            .set("call_block_number", call.call_block_number)
            .set("call_success", call.call_success)
            .set("amount", BigDecimal::from_str(&call.amount).unwrap())
            .set("output_param0", call.output_param0)
            .set("to", Hex(&call.to).to_string());
    });
    calls.stakedfrax_call_transfer_froms.iter().for_each(|call| {
        tables
            .create_row("stakedfrax_call_transfer_from", format!("{}-{}", call.call_tx_hash, call.call_ordinal))
            .set("call_tx_hash", &call.call_tx_hash)
            .set("call_ordinal", call.call_ordinal)
            .set("call_block_time", call.call_block_time.as_ref().unwrap())
            .set("call_block_number", call.call_block_number)
            .set("call_success", call.call_success)
            .set("amount", BigDecimal::from_str(&call.amount).unwrap())
            .set("from", Hex(&call.from).to_string())
            .set("output_param0", call.output_param0)
            .set("to", Hex(&call.to).to_string());
    });
    calls.stakedfrax_call_transfer_timelocks.iter().for_each(|call| {
        tables
            .create_row("stakedfrax_call_transfer_timelock", format!("{}-{}", call.call_tx_hash, call.call_ordinal))
            .set("call_tx_hash", &call.call_tx_hash)
            .set("call_ordinal", call.call_ordinal)
            .set("call_block_time", call.call_block_time.as_ref().unwrap())
            .set("call_block_number", call.call_block_number)
            .set("call_success", call.call_success)
            .set("u_new_timelock", Hex(&call.u_new_timelock).to_string());
    });
    calls.stakedfrax_call_withdraws.iter().for_each(|call| {
        tables
            .create_row("stakedfrax_call_withdraw", format!("{}-{}", call.call_tx_hash, call.call_ordinal))
            .set("call_tx_hash", &call.call_tx_hash)
            .set("call_ordinal", call.call_ordinal)
            .set("call_block_time", call.call_block_time.as_ref().unwrap())
            .set("call_block_number", call.call_block_number)
            .set("call_success", call.call_success)
            .set("output__shares", BigDecimal::from_str(&call.output__shares).unwrap())
            .set("u_assets", BigDecimal::from_str(&call.u_assets).unwrap())
            .set("u_owner", Hex(&call.u_owner).to_string())
            .set("u_receiver", Hex(&call.u_receiver).to_string());
    });
  }

#[substreams::handlers::map]
fn map_events(blk: eth::Block) -> Result<contract::Events, substreams::errors::Error> {
    let mut events = contract::Events::default();
    map_stakedfrax_events(&blk, &mut events);
    Ok(events)
}
#[substreams::handlers::map]
fn map_calls(blk: eth::Block) -> Result<contract::Calls, substreams::errors::Error> {
    let mut calls = contract::Calls::default();
    map_stakedfrax_calls(&blk, &mut calls);
    Ok(calls)
}

#[substreams::handlers::map]
fn db_out(events: contract::Events, calls: contract::Calls) -> Result<DatabaseChanges, substreams::errors::Error> {
    // Initialize Database Changes container
    let mut tables = DatabaseChangeTables::new();
    db_stakedfrax_out(&events, &mut tables);
    db_stakedfrax_calls_out(&calls, &mut tables);
    Ok(tables.to_database_changes())
}

#[substreams::handlers::map]
fn graph_out(events: contract::Events, calls: contract::Calls) -> Result<EntityChanges, substreams::errors::Error> {
    // Initialize Database Changes container
    let mut tables = EntityChangesTables::new();
    graph_stakedfrax_out(&events, &mut tables);
    graph_stakedfrax_calls_out(&calls, &mut tables);
    Ok(tables.to_entity_changes())
}
