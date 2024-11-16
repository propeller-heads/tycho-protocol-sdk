mod abi;
mod pb;
use hex_literal::hex;
use pb::contract::v1 as contract;
use substreams::Hex;
use substreams_ethereum::pb::eth::v2 as eth;
use substreams_ethereum::Event;

#[allow(unused_imports)]
use num_traits::cast::ToPrimitive;
use std::str::FromStr;
use substreams::scalar::BigDecimal;

substreams_ethereum::init!();

const RESTAKE_MANAGER_TRACKED_CONTRACT: [u8; 20] = hex!("74a09653a083691711cf8215a6ab074bb4e99ef5");
const WITHDRAWAL_CONTRACT_TRACKED_CONTRACT: [u8; 20] = hex!("5efc9d10e42fb517456f4ac41eb5e2ebe42c8918");

fn map_restake_manager_events(blk: &eth::Block, events: &mut contract::Events) {
    events.restake_manager_admin_changeds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == RESTAKE_MANAGER_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::restake_manager_contract::events::AdminChanged::match_and_decode(log) {
                        return Some(contract::RestakeManagerAdminChanged {
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
    events.restake_manager_beacon_upgradeds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == RESTAKE_MANAGER_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::restake_manager_contract::events::BeaconUpgraded::match_and_decode(log) {
                        return Some(contract::RestakeManagerBeaconUpgraded {
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
    events.restake_manager_collateral_token_addeds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == RESTAKE_MANAGER_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::restake_manager_contract::events::CollateralTokenAdded::match_and_decode(log) {
                        return Some(contract::RestakeManagerCollateralTokenAdded {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            token: event.token,
                        });
                    }

                    None
                })
        })
        .collect());
    events.restake_manager_collateral_token_removeds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == RESTAKE_MANAGER_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::restake_manager_contract::events::CollateralTokenRemoved::match_and_decode(log) {
                        return Some(contract::RestakeManagerCollateralTokenRemoved {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            token: event.token,
                        });
                    }

                    None
                })
        })
        .collect());
    events.restake_manager_collateral_token_tvl_updateds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == RESTAKE_MANAGER_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::restake_manager_contract::events::CollateralTokenTvlUpdated::match_and_decode(log) {
                        return Some(contract::RestakeManagerCollateralTokenTvlUpdated {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            token: event.token,
                            tvl: event.tvl.to_string(),
                        });
                    }

                    None
                })
        })
        .collect());
    events.restake_manager_deposits.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == RESTAKE_MANAGER_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::restake_manager_contract::events::Deposit::match_and_decode(log) {
                        return Some(contract::RestakeManagerDeposit {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            amount: event.amount.to_string(),
                            depositor: event.depositor,
                            ez_eth_minted: event.ez_eth_minted.to_string(),
                            referral_id: event.referral_id.to_string(),
                            token: event.token,
                        });
                    }

                    None
                })
        })
        .collect());
    events.restake_manager_initializeds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == RESTAKE_MANAGER_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::restake_manager_contract::events::Initialized::match_and_decode(log) {
                        return Some(contract::RestakeManagerInitialized {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            version: event.version.to_u64(),
                        });
                    }

                    None
                })
        })
        .collect());
    events.restake_manager_operator_delegator_addeds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == RESTAKE_MANAGER_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::restake_manager_contract::events::OperatorDelegatorAdded::match_and_decode(log) {
                        return Some(contract::RestakeManagerOperatorDelegatorAdded {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            od: event.od,
                        });
                    }

                    None
                })
        })
        .collect());
    events.restake_manager_operator_delegator_allocation_updateds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == RESTAKE_MANAGER_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::restake_manager_contract::events::OperatorDelegatorAllocationUpdated::match_and_decode(log) {
                        return Some(contract::RestakeManagerOperatorDelegatorAllocationUpdated {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            allocation: event.allocation.to_string(),
                            od: event.od,
                        });
                    }

                    None
                })
        })
        .collect());
    events.restake_manager_operator_delegator_removeds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == RESTAKE_MANAGER_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::restake_manager_contract::events::OperatorDelegatorRemoved::match_and_decode(log) {
                        return Some(contract::RestakeManagerOperatorDelegatorRemoved {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            od: event.od,
                        });
                    }

                    None
                })
        })
        .collect());
    events.restake_manager_upgradeds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == RESTAKE_MANAGER_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::restake_manager_contract::events::Upgraded::match_and_decode(log) {
                        return Some(contract::RestakeManagerUpgraded {
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
    events.restake_manager_user_withdraw_completeds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == RESTAKE_MANAGER_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::restake_manager_contract::events::UserWithdrawCompleted::match_and_decode(log) {
                        return Some(contract::RestakeManagerUserWithdrawCompleted {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            amount: event.amount.to_string(),
                            ez_eth_burned: event.ez_eth_burned.to_string(),
                            token: event.token,
                            withdrawal_root: Vec::from(event.withdrawal_root),
                            withdrawer: event.withdrawer,
                        });
                    }

                    None
                })
        })
        .collect());
    events.restake_manager_user_withdraw_starteds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == RESTAKE_MANAGER_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::restake_manager_contract::events::UserWithdrawStarted::match_and_decode(log) {
                        return Some(contract::RestakeManagerUserWithdrawStarted {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            amount: event.amount.to_string(),
                            ez_eth_to_burn: event.ez_eth_to_burn.to_string(),
                            token: event.token,
                            withdrawal_root: Vec::from(event.withdrawal_root),
                            withdrawer: event.withdrawer,
                        });
                    }

                    None
                })
        })
        .collect());
}
fn map_restake_manager_calls(blk: &eth::Block, calls: &mut contract::Calls) {
    calls.restake_manager_call_add_collateral_tokens.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == RESTAKE_MANAGER_TRACKED_CONTRACT && abi::restake_manager_contract::functions::AddCollateralToken::match_call(call))
                .filter_map(|call| {
                    match abi::restake_manager_contract::functions::AddCollateralToken::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::RestakeManagerAddCollateralTokenCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                u_new_collateral_token: decoded_call.u_new_collateral_token,
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.restake_manager_call_add_operator_delegators.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == RESTAKE_MANAGER_TRACKED_CONTRACT && abi::restake_manager_contract::functions::AddOperatorDelegator::match_call(call))
                .filter_map(|call| {
                    match abi::restake_manager_contract::functions::AddOperatorDelegator::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::RestakeManagerAddOperatorDelegatorCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                u_allocation_basis_points: decoded_call.u_allocation_basis_points.to_string(),
                                u_new_operator_delegator: decoded_call.u_new_operator_delegator,
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.restake_manager_call_admins.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == RESTAKE_MANAGER_TRACKED_CONTRACT && abi::restake_manager_contract::functions::Admin::match_call(call))
                .filter_map(|call| {
                    match abi::restake_manager_contract::functions::Admin::decode(call) {
                        Ok(decoded_call) => {
                            let output_admin = match abi::restake_manager_contract::functions::Admin::output(&call.return_data) {
                                Ok(output_admin) => {output_admin}
                                Err(_) => Default::default(),
                            };
                            
                            Some(contract::RestakeManagerAdminCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                output_admin: output_admin,
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.restake_manager_call_change_admins.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == RESTAKE_MANAGER_TRACKED_CONTRACT && abi::restake_manager_contract::functions::ChangeAdmin::match_call(call))
                .filter_map(|call| {
                    match abi::restake_manager_contract::functions::ChangeAdmin::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::RestakeManagerChangeAdminCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                new_admin: decoded_call.new_admin,
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.restake_manager_call_deposit_1s.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == RESTAKE_MANAGER_TRACKED_CONTRACT && abi::restake_manager_contract::functions::Deposit1::match_call(call))
                .filter_map(|call| {
                    match abi::restake_manager_contract::functions::Deposit1::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::RestakeManagerDeposit1call {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                u_amount: decoded_call.u_amount.to_string(),
                                u_collateral_token: decoded_call.u_collateral_token,
                                u_referral_id: decoded_call.u_referral_id.to_string(),
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.restake_manager_call_deposit_2s.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == RESTAKE_MANAGER_TRACKED_CONTRACT && abi::restake_manager_contract::functions::Deposit2::match_call(call))
                .filter_map(|call| {
                    match abi::restake_manager_contract::functions::Deposit2::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::RestakeManagerDeposit2call {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                u_amount: decoded_call.u_amount.to_string(),
                                u_collateral_token: decoded_call.u_collateral_token,
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.restake_manager_call_deposit_eth_1s.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == RESTAKE_MANAGER_TRACKED_CONTRACT && abi::restake_manager_contract::functions::DepositEth1::match_call(call))
                .filter_map(|call| {
                    match abi::restake_manager_contract::functions::DepositEth1::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::RestakeManagerDepositEth1call {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                u_referral_id: decoded_call.u_referral_id.to_string(),
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.restake_manager_call_deposit_eth_2s.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == RESTAKE_MANAGER_TRACKED_CONTRACT && abi::restake_manager_contract::functions::DepositEth2::match_call(call))
                .filter_map(|call| {
                    match abi::restake_manager_contract::functions::DepositEth2::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::RestakeManagerDepositEth2call {
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
    calls.restake_manager_call_deposit_token_rewards_from_protocols.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == RESTAKE_MANAGER_TRACKED_CONTRACT && abi::restake_manager_contract::functions::DepositTokenRewardsFromProtocol::match_call(call))
                .filter_map(|call| {
                    match abi::restake_manager_contract::functions::DepositTokenRewardsFromProtocol::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::RestakeManagerDepositTokenRewardsFromProtocolCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                u_amount: decoded_call.u_amount.to_string(),
                                u_token: decoded_call.u_token,
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.restake_manager_call_implementations.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == RESTAKE_MANAGER_TRACKED_CONTRACT && abi::restake_manager_contract::functions::Implementation::match_call(call))
                .filter_map(|call| {
                    match abi::restake_manager_contract::functions::Implementation::decode(call) {
                        Ok(decoded_call) => {
                            let output_implementation = match abi::restake_manager_contract::functions::Implementation::output(&call.return_data) {
                                Ok(output_implementation) => {output_implementation}
                                Err(_) => Default::default(),
                            };
                            
                            Some(contract::RestakeManagerImplementationCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                output_implementation: output_implementation,
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.restake_manager_call_initializes.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == RESTAKE_MANAGER_TRACKED_CONTRACT && abi::restake_manager_contract::functions::Initialize::match_call(call))
                .filter_map(|call| {
                    match abi::restake_manager_contract::functions::Initialize::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::RestakeManagerInitializeCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                u_delegation_manager: decoded_call.u_delegation_manager,
                                u_deposit_queue: decoded_call.u_deposit_queue,
                                u_ez_eth: decoded_call.u_ez_eth,
                                u_renzo_oracle: decoded_call.u_renzo_oracle,
                                u_role_manager: decoded_call.u_role_manager,
                                u_strategy_manager: decoded_call.u_strategy_manager,
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.restake_manager_call_remove_collateral_tokens.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == RESTAKE_MANAGER_TRACKED_CONTRACT && abi::restake_manager_contract::functions::RemoveCollateralToken::match_call(call))
                .filter_map(|call| {
                    match abi::restake_manager_contract::functions::RemoveCollateralToken::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::RestakeManagerRemoveCollateralTokenCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                u_collateral_token_to_remove: decoded_call.u_collateral_token_to_remove,
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.restake_manager_call_remove_operator_delegators.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == RESTAKE_MANAGER_TRACKED_CONTRACT && abi::restake_manager_contract::functions::RemoveOperatorDelegator::match_call(call))
                .filter_map(|call| {
                    match abi::restake_manager_contract::functions::RemoveOperatorDelegator::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::RestakeManagerRemoveOperatorDelegatorCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                u_operator_delegator_to_remove: decoded_call.u_operator_delegator_to_remove,
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.restake_manager_call_set_operator_delegator_allocations.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == RESTAKE_MANAGER_TRACKED_CONTRACT && abi::restake_manager_contract::functions::SetOperatorDelegatorAllocation::match_call(call))
                .filter_map(|call| {
                    match abi::restake_manager_contract::functions::SetOperatorDelegatorAllocation::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::RestakeManagerSetOperatorDelegatorAllocationCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                u_allocation_basis_points: decoded_call.u_allocation_basis_points.to_string(),
                                u_operator_delegator: decoded_call.u_operator_delegator,
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.restake_manager_call_set_pauseds.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == RESTAKE_MANAGER_TRACKED_CONTRACT && abi::restake_manager_contract::functions::SetPaused::match_call(call))
                .filter_map(|call| {
                    match abi::restake_manager_contract::functions::SetPaused::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::RestakeManagerSetPausedCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                u_paused: decoded_call.u_paused,
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.restake_manager_call_set_token_tvl_limits.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == RESTAKE_MANAGER_TRACKED_CONTRACT && abi::restake_manager_contract::functions::SetTokenTvlLimit::match_call(call))
                .filter_map(|call| {
                    match abi::restake_manager_contract::functions::SetTokenTvlLimit::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::RestakeManagerSetTokenTvlLimitCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                u_limit: decoded_call.u_limit.to_string(),
                                u_token: decoded_call.u_token,
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.restake_manager_call_stake_eth_in_operator_delegators.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == RESTAKE_MANAGER_TRACKED_CONTRACT && abi::restake_manager_contract::functions::StakeEthInOperatorDelegator::match_call(call))
                .filter_map(|call| {
                    match abi::restake_manager_contract::functions::StakeEthInOperatorDelegator::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::RestakeManagerStakeEthInOperatorDelegatorCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                deposit_data_root: Vec::from(decoded_call.deposit_data_root),
                                operator_delegator: decoded_call.operator_delegator,
                                pubkey: decoded_call.pubkey,
                                signature: decoded_call.signature,
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.restake_manager_call_upgrade_tos.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == RESTAKE_MANAGER_TRACKED_CONTRACT && abi::restake_manager_contract::functions::UpgradeTo::match_call(call))
                .filter_map(|call| {
                    match abi::restake_manager_contract::functions::UpgradeTo::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::RestakeManagerUpgradeToCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                new_implementation: decoded_call.new_implementation,
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.restake_manager_call_upgrade_to_and_calls.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == RESTAKE_MANAGER_TRACKED_CONTRACT && abi::restake_manager_contract::functions::UpgradeToAndCall::match_call(call))
                .filter_map(|call| {
                    match abi::restake_manager_contract::functions::UpgradeToAndCall::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::RestakeManagerUpgradeToAndCallCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                data: decoded_call.data,
                                new_implementation: decoded_call.new_implementation,
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
}

fn map_withdrawal_contract_events(blk: &eth::Block, events: &mut contract::Events) {
    events.withdrawal_contract_admin_changeds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == WITHDRAWAL_CONTRACT_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::withdrawal_contract_contract::events::AdminChanged::match_and_decode(log) {
                        return Some(contract::WithdrawalContractAdminChanged {
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
    events.withdrawal_contract_beacon_upgradeds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == WITHDRAWAL_CONTRACT_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::withdrawal_contract_contract::events::BeaconUpgraded::match_and_decode(log) {
                        return Some(contract::WithdrawalContractBeaconUpgraded {
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
    events.withdrawal_contract_cool_down_period_updateds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == WITHDRAWAL_CONTRACT_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::withdrawal_contract_contract::events::CoolDownPeriodUpdated::match_and_decode(log) {
                        return Some(contract::WithdrawalContractCoolDownPeriodUpdated {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            new_cool_down_period: event.new_cool_down_period.to_string(),
                            old_cool_down_period: event.old_cool_down_period.to_string(),
                        });
                    }

                    None
                })
        })
        .collect());
    events.withdrawal_contract_erc20_buffer_filleds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == WITHDRAWAL_CONTRACT_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::withdrawal_contract_contract::events::Erc20BufferFilled::match_and_decode(log) {
                        return Some(contract::WithdrawalContractErc20BufferFilled {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            amount: event.amount.to_string(),
                            asset: event.asset,
                        });
                    }

                    None
                })
        })
        .collect());
    events.withdrawal_contract_eth_buffer_filleds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == WITHDRAWAL_CONTRACT_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::withdrawal_contract_contract::events::EthBufferFilled::match_and_decode(log) {
                        return Some(contract::WithdrawalContractEthBufferFilled {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            amount: event.amount.to_string(),
                        });
                    }

                    None
                })
        })
        .collect());
    events.withdrawal_contract_initializeds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == WITHDRAWAL_CONTRACT_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::withdrawal_contract_contract::events::Initialized::match_and_decode(log) {
                        return Some(contract::WithdrawalContractInitialized {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            version: event.version.to_u64(),
                        });
                    }

                    None
                })
        })
        .collect());
    events.withdrawal_contract_pauseds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == WITHDRAWAL_CONTRACT_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::withdrawal_contract_contract::events::Paused::match_and_decode(log) {
                        return Some(contract::WithdrawalContractPaused {
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
    events.withdrawal_contract_queue_filleds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == WITHDRAWAL_CONTRACT_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::withdrawal_contract_contract::events::QueueFilled::match_and_decode(log) {
                        return Some(contract::WithdrawalContractQueueFilled {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            amount: event.amount.to_string(),
                            asset: event.asset,
                        });
                    }

                    None
                })
        })
        .collect());
    events.withdrawal_contract_unpauseds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == WITHDRAWAL_CONTRACT_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::withdrawal_contract_contract::events::Unpaused::match_and_decode(log) {
                        return Some(contract::WithdrawalContractUnpaused {
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
    events.withdrawal_contract_upgradeds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == WITHDRAWAL_CONTRACT_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::withdrawal_contract_contract::events::Upgraded::match_and_decode(log) {
                        return Some(contract::WithdrawalContractUpgraded {
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
    events.withdrawal_contract_withdraw_buffer_target_updateds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == WITHDRAWAL_CONTRACT_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::withdrawal_contract_contract::events::WithdrawBufferTargetUpdated::match_and_decode(log) {
                        return Some(contract::WithdrawalContractWithdrawBufferTargetUpdated {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            new_buffer_target: event.new_buffer_target.to_string(),
                            old_buffer_target: event.old_buffer_target.to_string(),
                        });
                    }

                    None
                })
        })
        .collect());
    events.withdrawal_contract_withdraw_request_claimeds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == WITHDRAWAL_CONTRACT_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::withdrawal_contract_contract::events::WithdrawRequestClaimed::match_and_decode(log) {
                        return Some(contract::WithdrawalContractWithdrawRequestClaimed {
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
    events.withdrawal_contract_withdraw_request_createds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == WITHDRAWAL_CONTRACT_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::withdrawal_contract_contract::events::WithdrawRequestCreated::match_and_decode(log) {
                        return Some(contract::WithdrawalContractWithdrawRequestCreated {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            amount_to_redeem: event.amount_to_redeem.to_string(),
                            claim_token: event.claim_token,
                            ez_eth_amount_locked: event.ez_eth_amount_locked.to_string(),
                            queue_filled: event.queue_filled.to_string(),
                            queued: event.queued,
                            user: event.user,
                            withdraw_request_id: event.withdraw_request_id.to_string(),
                            withdraw_request_index: event.withdraw_request_index.to_string(),
                        });
                    }

                    None
                })
        })
        .collect());
}
fn map_withdrawal_contract_calls(blk: &eth::Block, calls: &mut contract::Calls) {
    calls.withdrawal_contract_call_admins.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == WITHDRAWAL_CONTRACT_TRACKED_CONTRACT && abi::withdrawal_contract_contract::functions::Admin::match_call(call))
                .filter_map(|call| {
                    match abi::withdrawal_contract_contract::functions::Admin::decode(call) {
                        Ok(decoded_call) => {
                            let output_admin = match abi::withdrawal_contract_contract::functions::Admin::output(&call.return_data) {
                                Ok(output_admin) => {output_admin}
                                Err(_) => Default::default(),
                            };
                            
                            Some(contract::WithdrawalContractAdminCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                output_admin: output_admin,
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.withdrawal_contract_call_change_admins.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == WITHDRAWAL_CONTRACT_TRACKED_CONTRACT && abi::withdrawal_contract_contract::functions::ChangeAdmin::match_call(call))
                .filter_map(|call| {
                    match abi::withdrawal_contract_contract::functions::ChangeAdmin::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::WithdrawalContractChangeAdminCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                new_admin: decoded_call.new_admin,
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.withdrawal_contract_call_claims.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == WITHDRAWAL_CONTRACT_TRACKED_CONTRACT && abi::withdrawal_contract_contract::functions::Claim::match_call(call))
                .filter_map(|call| {
                    match abi::withdrawal_contract_contract::functions::Claim::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::WithdrawalContractClaimCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                user: decoded_call.user,
                                withdraw_request_index: decoded_call.withdraw_request_index.to_string(),
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.withdrawal_contract_call_fill_erc20_withdraw_buffers.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == WITHDRAWAL_CONTRACT_TRACKED_CONTRACT && abi::withdrawal_contract_contract::functions::FillErc20WithdrawBuffer::match_call(call))
                .filter_map(|call| {
                    match abi::withdrawal_contract_contract::functions::FillErc20WithdrawBuffer::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::WithdrawalContractFillErc20WithdrawBufferCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                u_amount: decoded_call.u_amount.to_string(),
                                u_asset: decoded_call.u_asset,
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.withdrawal_contract_call_fill_eth_withdraw_buffers.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == WITHDRAWAL_CONTRACT_TRACKED_CONTRACT && abi::withdrawal_contract_contract::functions::FillEthWithdrawBuffer::match_call(call))
                .filter_map(|call| {
                    match abi::withdrawal_contract_contract::functions::FillEthWithdrawBuffer::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::WithdrawalContractFillEthWithdrawBufferCall {
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
    calls.withdrawal_contract_call_implementations.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == WITHDRAWAL_CONTRACT_TRACKED_CONTRACT && abi::withdrawal_contract_contract::functions::Implementation::match_call(call))
                .filter_map(|call| {
                    match abi::withdrawal_contract_contract::functions::Implementation::decode(call) {
                        Ok(decoded_call) => {
                            let output_implementation = match abi::withdrawal_contract_contract::functions::Implementation::output(&call.return_data) {
                                Ok(output_implementation) => {output_implementation}
                                Err(_) => Default::default(),
                            };
                            
                            Some(contract::WithdrawalContractImplementationCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                output_implementation: output_implementation,
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.withdrawal_contract_call_initializes.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == WITHDRAWAL_CONTRACT_TRACKED_CONTRACT && abi::withdrawal_contract_contract::functions::Initialize::match_call(call))
                .filter_map(|call| {
                    match abi::withdrawal_contract_contract::functions::Initialize::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::WithdrawalContractInitializeCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                u_cool_down_period: decoded_call.u_cool_down_period.to_string(),
                                u_ez_eth: decoded_call.u_ez_eth,
                                u_renzo_oracle: decoded_call.u_renzo_oracle,
                                u_restake_manager: decoded_call.u_restake_manager,
                                u_role_manager: decoded_call.u_role_manager,
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.withdrawal_contract_call_pauses.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == WITHDRAWAL_CONTRACT_TRACKED_CONTRACT && abi::withdrawal_contract_contract::functions::Pause::match_call(call))
                .filter_map(|call| {
                    match abi::withdrawal_contract_contract::functions::Pause::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::WithdrawalContractPauseCall {
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
    calls.withdrawal_contract_call_unpauses.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == WITHDRAWAL_CONTRACT_TRACKED_CONTRACT && abi::withdrawal_contract_contract::functions::Unpause::match_call(call))
                .filter_map(|call| {
                    match abi::withdrawal_contract_contract::functions::Unpause::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::WithdrawalContractUnpauseCall {
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
    calls.withdrawal_contract_call_update_cool_down_periods.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == WITHDRAWAL_CONTRACT_TRACKED_CONTRACT && abi::withdrawal_contract_contract::functions::UpdateCoolDownPeriod::match_call(call))
                .filter_map(|call| {
                    match abi::withdrawal_contract_contract::functions::UpdateCoolDownPeriod::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::WithdrawalContractUpdateCoolDownPeriodCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                u_new_cool_down_period: decoded_call.u_new_cool_down_period.to_string(),
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.withdrawal_contract_call_update_withdraw_buffer_targets.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == WITHDRAWAL_CONTRACT_TRACKED_CONTRACT && abi::withdrawal_contract_contract::functions::UpdateWithdrawBufferTarget::match_call(call))
                .filter_map(|call| {
                    match abi::withdrawal_contract_contract::functions::UpdateWithdrawBufferTarget::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::WithdrawalContractUpdateWithdrawBufferTargetCall {
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
    calls.withdrawal_contract_call_upgrade_tos.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == WITHDRAWAL_CONTRACT_TRACKED_CONTRACT && abi::withdrawal_contract_contract::functions::UpgradeTo::match_call(call))
                .filter_map(|call| {
                    match abi::withdrawal_contract_contract::functions::UpgradeTo::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::WithdrawalContractUpgradeToCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                new_implementation: decoded_call.new_implementation,
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.withdrawal_contract_call_upgrade_to_and_calls.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == WITHDRAWAL_CONTRACT_TRACKED_CONTRACT && abi::withdrawal_contract_contract::functions::UpgradeToAndCall::match_call(call))
                .filter_map(|call| {
                    match abi::withdrawal_contract_contract::functions::UpgradeToAndCall::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::WithdrawalContractUpgradeToAndCallCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                data: decoded_call.data,
                                new_implementation: decoded_call.new_implementation,
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.withdrawal_contract_call_withdraws.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == WITHDRAWAL_CONTRACT_TRACKED_CONTRACT && abi::withdrawal_contract_contract::functions::Withdraw::match_call(call))
                .filter_map(|call| {
                    match abi::withdrawal_contract_contract::functions::Withdraw::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::WithdrawalContractWithdrawCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                u_amount: decoded_call.u_amount.to_string(),
                                u_asset_out: decoded_call.u_asset_out,
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
}

#[substreams::handlers::map]
fn map_events_calls(
    events: contract::Events,
    calls: contract::Calls,
) -> Result<contract::EventsCalls, substreams::errors::Error> {
    Ok(contract::EventsCalls {
        events: Some(events),
        calls: Some(calls),
    })
}
#[substreams::handlers::map]
fn map_events(blk: eth::Block) -> Result<contract::Events, substreams::errors::Error> {
    let mut events = contract::Events::default();
    map_restake_manager_events(&blk, &mut events);
    map_withdrawal_contract_events(&blk, &mut events);
    Ok(events)
}
#[substreams::handlers::map]
fn map_calls(blk: eth::Block) -> Result<contract::Calls, substreams::errors::Error> {
let mut calls = contract::Calls::default();
    map_restake_manager_calls(&blk, &mut calls);
    map_withdrawal_contract_calls(&blk, &mut calls);
    Ok(calls)
}

