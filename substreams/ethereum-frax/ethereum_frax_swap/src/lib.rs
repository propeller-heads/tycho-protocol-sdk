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

const FACTORY_TRACKED_CONTRACT: [u8; 20] = hex!("43ec799eadd63848443e2347c49f5f52e8fe0f6f");

fn map_factory_events(blk: &eth::Block, events: &mut contract::Events) {
    events.factory_pair_createds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == FACTORY_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::factory_contract::events::PairCreated::match_and_decode(log) {
                        return Some(contract::FactoryPairCreated {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            pair: event.pair,
                            param3: event.param3.to_string(),
                            token0: event.token0,
                            token1: event.token1,
                        });
                    }

                    None
                })
        })
        .collect());
}

fn map_factory_calls(blk: &eth::Block, calls: &mut contract::Calls) {
    calls.factory_call_create_pair_1s.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == FACTORY_TRACKED_CONTRACT && abi::factory_contract::functions::CreatePair1::match_call(call))
                .filter_map(|call| {
                    match abi::factory_contract::functions::CreatePair1::decode(call) {
                        Ok(decoded_call) => {
                            let output_pair = match abi::factory_contract::functions::CreatePair1::output(&call.return_data) {
                                Ok(output_pair) => {output_pair}
                                Err(_) => Default::default(),
                            };
                            
                            Some(contract::FactoryCreatePair1Call {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                fee: decoded_call.fee.to_string(),
                                output_pair: output_pair,
                                token_a: decoded_call.token_a,
                                token_b: decoded_call.token_b,
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.factory_call_create_pair_2s.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == FACTORY_TRACKED_CONTRACT && abi::factory_contract::functions::CreatePair2::match_call(call))
                .filter_map(|call| {
                    match abi::factory_contract::functions::CreatePair2::decode(call) {
                        Ok(decoded_call) => {
                            let output_pair = match abi::factory_contract::functions::CreatePair2::output(&call.return_data) {
                                Ok(output_pair) => {output_pair}
                                Err(_) => Default::default(),
                            };
                            
                            Some(contract::FactoryCreatePair2Call {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                output_pair: output_pair,
                                token_a: decoded_call.token_a,
                                token_b: decoded_call.token_b,
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.factory_call_set_fee_tos.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == FACTORY_TRACKED_CONTRACT && abi::factory_contract::functions::SetFeeTo::match_call(call))
                .filter_map(|call| {
                    match abi::factory_contract::functions::SetFeeTo::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::FactorySetFeeToCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                u_fee_to: decoded_call.u_fee_to,
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.factory_call_set_fee_to_setters.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == FACTORY_TRACKED_CONTRACT && abi::factory_contract::functions::SetFeeToSetter::match_call(call))
                .filter_map(|call| {
                    match abi::factory_contract::functions::SetFeeToSetter::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::FactorySetFeeToSetterCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                u_fee_to_setter: decoded_call.u_fee_to_setter,
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.factory_call_toggle_global_pauses.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == FACTORY_TRACKED_CONTRACT && abi::factory_contract::functions::ToggleGlobalPause::match_call(call))
                .filter_map(|call| {
                    match abi::factory_contract::functions::ToggleGlobalPause::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::FactoryToggleGlobalPauseCall {
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
}

fn is_declared_dds_address(addr: &Vec<u8>, ordinal: u64, dds_store: &store::StoreGetInt64) -> bool {
    //    substreams::log::info!("Checking if address {} is declared dds address", Hex(addr).to_string());
    if dds_store.get_at(ordinal, Hex(addr).to_string()).is_some() {
        return true;
    }
    return false;
}

fn map_pair_events(
    blk: &eth::Block,
    dds_store: &store::StoreGetInt64,
    events: &mut contract::Events,
) {

    events.pair_approvals.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| is_declared_dds_address(&log.address, log.ordinal, dds_store))
                .filter_map(|log| {
                    if let Some(event) = abi::pair_contract::events::Approval::match_and_decode(log) {
                        return Some(contract::PairApproval {
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

    events.pair_burns.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| is_declared_dds_address(&log.address, log.ordinal, dds_store))
                .filter_map(|log| {
                    if let Some(event) = abi::pair_contract::events::Burn::match_and_decode(log) {
                        return Some(contract::PairBurn {
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

    events.pair_cancel_long_term_orders.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| is_declared_dds_address(&log.address, log.ordinal, dds_store))
                .filter_map(|log| {
                    if let Some(event) = abi::pair_contract::events::CancelLongTermOrder::match_and_decode(log) {
                        return Some(contract::PairCancelLongTermOrder {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            evt_address: Hex(&log.address).to_string(),
                            addr: event.addr,
                            buy_token: event.buy_token,
                            order_id: event.order_id.to_string(),
                            purchased_amount: event.purchased_amount.to_string(),
                            sell_token: event.sell_token,
                            unsold_amount: event.unsold_amount.to_string(),
                        });
                    }

                    None
                })
        })
        .collect());

    events.pair_long_term_swap0_to_1s.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| is_declared_dds_address(&log.address, log.ordinal, dds_store))
                .filter_map(|log| {
                    if let Some(event) = abi::pair_contract::events::LongTermSwap0To1::match_and_decode(log) {
                        return Some(contract::PairLongTermSwap0To1 {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            evt_address: Hex(&log.address).to_string(),
                            addr: event.addr,
                            amount0_in: event.amount0_in.to_string(),
                            number_of_time_intervals: event.number_of_time_intervals.to_string(),
                            order_id: event.order_id.to_string(),
                        });
                    }

                    None
                })
        })
        .collect());

    events.pair_long_term_swap1_to_0s.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| is_declared_dds_address(&log.address, log.ordinal, dds_store))
                .filter_map(|log| {
                    if let Some(event) = abi::pair_contract::events::LongTermSwap1To0::match_and_decode(log) {
                        return Some(contract::PairLongTermSwap1To0 {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            evt_address: Hex(&log.address).to_string(),
                            addr: event.addr,
                            amount1_in: event.amount1_in.to_string(),
                            number_of_time_intervals: event.number_of_time_intervals.to_string(),
                            order_id: event.order_id.to_string(),
                        });
                    }

                    None
                })
        })
        .collect());

    events.pair_lp_fee_updateds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| is_declared_dds_address(&log.address, log.ordinal, dds_store))
                .filter_map(|log| {
                    if let Some(event) = abi::pair_contract::events::LpFeeUpdated::match_and_decode(log) {
                        return Some(contract::PairLpFeeUpdated {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            evt_address: Hex(&log.address).to_string(),
                            fee: event.fee.to_string(),
                        });
                    }

                    None
                })
        })
        .collect());

    events.pair_mints.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| is_declared_dds_address(&log.address, log.ordinal, dds_store))
                .filter_map(|log| {
                    if let Some(event) = abi::pair_contract::events::Mint::match_and_decode(log) {
                        return Some(contract::PairMint {
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

    events.pair_swaps.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| is_declared_dds_address(&log.address, log.ordinal, dds_store))
                .filter_map(|log| {
                    if let Some(event) = abi::pair_contract::events::Swap::match_and_decode(log) {
                        return Some(contract::PairSwap {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            evt_address: Hex(&log.address).to_string(),
                            amount0_in: event.amount0_in.to_string(),
                            amount0_out: event.amount0_out.to_string(),
                            amount1_in: event.amount1_in.to_string(),
                            amount1_out: event.amount1_out.to_string(),
                            sender: event.sender,
                            to: event.to,
                        });
                    }

                    None
                })
        })
        .collect());

    events.pair_syncs.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| is_declared_dds_address(&log.address, log.ordinal, dds_store))
                .filter_map(|log| {
                    if let Some(event) = abi::pair_contract::events::Sync::match_and_decode(log) {
                        return Some(contract::PairSync {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            evt_address: Hex(&log.address).to_string(),
                            reserve0: event.reserve0.to_string(),
                            reserve1: event.reserve1.to_string(),
                        });
                    }

                    None
                })
        })
        .collect());

    events.pair_transfers.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| is_declared_dds_address(&log.address, log.ordinal, dds_store))
                .filter_map(|log| {
                    if let Some(event) = abi::pair_contract::events::Transfer::match_and_decode(log) {
                        return Some(contract::PairTransfer {
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

    events.pair_withdraw_proceeds_from_long_term_orders.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| is_declared_dds_address(&log.address, log.ordinal, dds_store))
                .filter_map(|log| {
                    if let Some(event) = abi::pair_contract::events::WithdrawProceedsFromLongTermOrder::match_and_decode(log) {
                        return Some(contract::PairWithdrawProceedsFromLongTermOrder {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            evt_address: Hex(&log.address).to_string(),
                            addr: event.addr,
                            order_expired: event.order_expired,
                            order_id: event.order_id.to_string(),
                            proceed_token: event.proceed_token,
                            proceeds: event.proceeds.to_string(),
                        });
                    }

                    None
                })
        })
        .collect());
}
fn map_pair_calls(
    blk: &eth::Block,
    dds_store: &store::StoreGetInt64,
    calls: &mut contract::Calls,
) {
    calls.pair_call_approves.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| is_declared_dds_address(&call.address, call.begin_ordinal, dds_store) && abi::pair_contract::functions::Approve::match_call(call))
                .filter_map(|call| {
                    match abi::pair_contract::functions::Approve::decode(call) {
                            Ok(decoded_call) => {
                            let output_param0 = match abi::pair_contract::functions::Approve::output(&call.return_data) {
                                Ok(output_param0) => {output_param0}
                                Err(_) => Default::default(),
                            };
                            
                            Some(contract::PairApproveCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                call_address: Hex(&call.address).to_string(),
                                output_param0: output_param0,
                                spender: decoded_call.spender,
                                value: decoded_call.value.to_string(),
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.pair_call_burns.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| is_declared_dds_address(&call.address, call.begin_ordinal, dds_store) && abi::pair_contract::functions::Burn::match_call(call))
                .filter_map(|call| {
                    match abi::pair_contract::functions::Burn::decode(call) {
                            Ok(decoded_call) => {
                            let (output_amount0, output_amount1) = match abi::pair_contract::functions::Burn::output(&call.return_data) {
                                Ok((output_amount0, output_amount1)) => {(output_amount0, output_amount1)}
                                Err(_) => Default::default(),
                            };
                            
                            Some(contract::PairBurnCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                call_address: Hex(&call.address).to_string(),
                                output_amount0: output_amount0.to_string(),
                                output_amount1: output_amount1.to_string(),
                                to: decoded_call.to,
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.pair_call_cancel_long_term_swaps.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| is_declared_dds_address(&call.address, call.begin_ordinal, dds_store) && abi::pair_contract::functions::CancelLongTermSwap::match_call(call))
                .filter_map(|call| {
                    match abi::pair_contract::functions::CancelLongTermSwap::decode(call) {
                            Ok(decoded_call) => {
                            Some(contract::PairCancelLongTermSwapCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                call_address: Hex(&call.address).to_string(),
                                order_id: decoded_call.order_id.to_string(),
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.pair_call_execute_virtual_orders.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| is_declared_dds_address(&call.address, call.begin_ordinal, dds_store) && abi::pair_contract::functions::ExecuteVirtualOrders::match_call(call))
                .filter_map(|call| {
                    match abi::pair_contract::functions::ExecuteVirtualOrders::decode(call) {
                            Ok(decoded_call) => {
                            Some(contract::PairExecuteVirtualOrdersCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                call_address: Hex(&call.address).to_string(),
                                block_timestamp: decoded_call.block_timestamp.to_string(),
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.pair_call_get_twamm_order_proceeds.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| is_declared_dds_address(&call.address, call.begin_ordinal, dds_store) && abi::pair_contract::functions::GetTwammOrderProceeds::match_call(call))
                .filter_map(|call| {
                    match abi::pair_contract::functions::GetTwammOrderProceeds::decode(call) {
                            Ok(decoded_call) => {
                            let (output_order_expired, output_total_reward) = match abi::pair_contract::functions::GetTwammOrderProceeds::output(&call.return_data) {
                                Ok((output_order_expired, output_total_reward)) => {(output_order_expired, output_total_reward)}
                                Err(_) => Default::default(),
                            };
                            
                            Some(contract::PairGetTwammOrderProceedsCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                call_address: Hex(&call.address).to_string(),
                                order_id: decoded_call.order_id.to_string(),
                                output_order_expired: output_order_expired,
                                output_total_reward: output_total_reward.to_string(),
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.pair_call_initializes.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| is_declared_dds_address(&call.address, call.begin_ordinal, dds_store) && abi::pair_contract::functions::Initialize::match_call(call))
                .filter_map(|call| {
                    match abi::pair_contract::functions::Initialize::decode(call) {
                            Ok(decoded_call) => {
                            Some(contract::PairInitializeCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                call_address: Hex(&call.address).to_string(),
                                u_fee: decoded_call.u_fee.to_string(),
                                u_token0: decoded_call.u_token0,
                                u_token1: decoded_call.u_token1,
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.pair_call_long_term_swap_from0_to_1s.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| is_declared_dds_address(&call.address, call.begin_ordinal, dds_store) && abi::pair_contract::functions::LongTermSwapFrom0To1::match_call(call))
                .filter_map(|call| {
                    match abi::pair_contract::functions::LongTermSwapFrom0To1::decode(call) {
                            Ok(decoded_call) => {
                            let output_order_id = match abi::pair_contract::functions::LongTermSwapFrom0To1::output(&call.return_data) {
                                Ok(output_order_id) => {output_order_id}
                                Err(_) => Default::default(),
                            };
                            
                            Some(contract::PairLongTermSwapFrom0To1Call {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                call_address: Hex(&call.address).to_string(),
                                amount0_in: decoded_call.amount0_in.to_string(),
                                number_of_time_intervals: decoded_call.number_of_time_intervals.to_string(),
                                output_order_id: output_order_id.to_string(),
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.pair_call_long_term_swap_from1_to_0s.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| is_declared_dds_address(&call.address, call.begin_ordinal, dds_store) && abi::pair_contract::functions::LongTermSwapFrom1To0::match_call(call))
                .filter_map(|call| {
                    match abi::pair_contract::functions::LongTermSwapFrom1To0::decode(call) {
                            Ok(decoded_call) => {
                            let output_order_id = match abi::pair_contract::functions::LongTermSwapFrom1To0::output(&call.return_data) {
                                Ok(output_order_id) => {output_order_id}
                                Err(_) => Default::default(),
                            };
                            
                            Some(contract::PairLongTermSwapFrom1To0Call {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                call_address: Hex(&call.address).to_string(),
                                amount1_in: decoded_call.amount1_in.to_string(),
                                number_of_time_intervals: decoded_call.number_of_time_intervals.to_string(),
                                output_order_id: output_order_id.to_string(),
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.pair_call_mints.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| is_declared_dds_address(&call.address, call.begin_ordinal, dds_store) && abi::pair_contract::functions::Mint::match_call(call))
                .filter_map(|call| {
                    match abi::pair_contract::functions::Mint::decode(call) {
                            Ok(decoded_call) => {
                            let output_liquidity = match abi::pair_contract::functions::Mint::output(&call.return_data) {
                                Ok(output_liquidity) => {output_liquidity}
                                Err(_) => Default::default(),
                            };
                            
                            Some(contract::PairMintCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                call_address: Hex(&call.address).to_string(),
                                output_liquidity: output_liquidity.to_string(),
                                to: decoded_call.to,
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.pair_call_permits.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| is_declared_dds_address(&call.address, call.begin_ordinal, dds_store) && abi::pair_contract::functions::Permit::match_call(call))
                .filter_map(|call| {
                    match abi::pair_contract::functions::Permit::decode(call) {
                            Ok(decoded_call) => {
                            Some(contract::PairPermitCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                call_address: Hex(&call.address).to_string(),
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
    calls.pair_call_set_fees.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| is_declared_dds_address(&call.address, call.begin_ordinal, dds_store) && abi::pair_contract::functions::SetFee::match_call(call))
                .filter_map(|call| {
                    match abi::pair_contract::functions::SetFee::decode(call) {
                            Ok(decoded_call) => {
                            Some(contract::PairSetFeeCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                call_address: Hex(&call.address).to_string(),
                                new_fee: decoded_call.new_fee.to_string(),
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.pair_call_skims.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| is_declared_dds_address(&call.address, call.begin_ordinal, dds_store) && abi::pair_contract::functions::Skim::match_call(call))
                .filter_map(|call| {
                    match abi::pair_contract::functions::Skim::decode(call) {
                            Ok(decoded_call) => {
                            Some(contract::PairSkimCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                call_address: Hex(&call.address).to_string(),
                                to: decoded_call.to,
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.pair_call_swaps.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| is_declared_dds_address(&call.address, call.begin_ordinal, dds_store) && abi::pair_contract::functions::Swap::match_call(call))
                .filter_map(|call| {
                    match abi::pair_contract::functions::Swap::decode(call) {
                            Ok(decoded_call) => {
                            Some(contract::PairSwapCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                call_address: Hex(&call.address).to_string(),
                                amount0_out: decoded_call.amount0_out.to_string(),
                                amount1_out: decoded_call.amount1_out.to_string(),
                                data: decoded_call.data,
                                to: decoded_call.to,
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.pair_call_syncs.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| is_declared_dds_address(&call.address, call.begin_ordinal, dds_store) && abi::pair_contract::functions::Sync::match_call(call))
                .filter_map(|call| {
                    match abi::pair_contract::functions::Sync::decode(call) {
                            Ok(decoded_call) => {
                            Some(contract::PairSyncCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                call_address: Hex(&call.address).to_string(),
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.pair_call_toggle_pause_new_swaps.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| is_declared_dds_address(&call.address, call.begin_ordinal, dds_store) && abi::pair_contract::functions::TogglePauseNewSwaps::match_call(call))
                .filter_map(|call| {
                    match abi::pair_contract::functions::TogglePauseNewSwaps::decode(call) {
                            Ok(decoded_call) => {
                            Some(contract::PairTogglePauseNewSwapsCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                call_address: Hex(&call.address).to_string(),
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.pair_call_transfers.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| is_declared_dds_address(&call.address, call.begin_ordinal, dds_store) && abi::pair_contract::functions::Transfer::match_call(call))
                .filter_map(|call| {
                    match abi::pair_contract::functions::Transfer::decode(call) {
                            Ok(decoded_call) => {
                            let output_param0 = match abi::pair_contract::functions::Transfer::output(&call.return_data) {
                                Ok(output_param0) => {output_param0}
                                Err(_) => Default::default(),
                            };
                            
                            Some(contract::PairTransferCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                call_address: Hex(&call.address).to_string(),
                                output_param0: output_param0,
                                to: decoded_call.to,
                                value: decoded_call.value.to_string(),
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.pair_call_transfer_froms.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| is_declared_dds_address(&call.address, call.begin_ordinal, dds_store) && abi::pair_contract::functions::TransferFrom::match_call(call))
                .filter_map(|call| {
                    match abi::pair_contract::functions::TransferFrom::decode(call) {
                            Ok(decoded_call) => {
                            let output_param0 = match abi::pair_contract::functions::TransferFrom::output(&call.return_data) {
                                Ok(output_param0) => {output_param0}
                                Err(_) => Default::default(),
                            };
                            
                            Some(contract::PairTransferFromCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                call_address: Hex(&call.address).to_string(),
                                from: decoded_call.from,
                                output_param0: output_param0,
                                to: decoded_call.to,
                                value: decoded_call.value.to_string(),
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.pair_call_withdraw_proceeds_from_long_term_swaps.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| is_declared_dds_address(&call.address, call.begin_ordinal, dds_store) && abi::pair_contract::functions::WithdrawProceedsFromLongTermSwap::match_call(call))
                .filter_map(|call| {
                    match abi::pair_contract::functions::WithdrawProceedsFromLongTermSwap::decode(call) {
                            Ok(decoded_call) => {
                            let (output_is_expired, output_reward_tkn, output_total_reward) = match abi::pair_contract::functions::WithdrawProceedsFromLongTermSwap::output(&call.return_data) {
                                Ok((output_is_expired, output_reward_tkn, output_total_reward)) => {(output_is_expired, output_reward_tkn, output_total_reward)}
                                Err(_) => Default::default(),
                            };
                            
                            Some(contract::PairWithdrawProceedsFromLongTermSwapCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                call_address: Hex(&call.address).to_string(),
                                order_id: decoded_call.order_id.to_string(),
                                output_is_expired: output_is_expired,
                                output_reward_tkn: output_reward_tkn,
                                output_total_reward: output_total_reward.to_string(),
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
}



fn db_factory_out(events: &contract::Events, tables: &mut DatabaseChangeTables) {
    // Loop over all the abis events to create table changes
    events.factory_pair_createds.iter().for_each(|evt| {
        tables
            .create_row("factory_pair_created", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("pair", Hex(&evt.pair).to_string())
            .set("param3", BigDecimal::from_str(&evt.param3).unwrap())
            .set("token0", Hex(&evt.token0).to_string())
            .set("token1", Hex(&evt.token1).to_string());
    });
}
fn db_factory_calls_out(calls: &contract::Calls, tables: &mut DatabaseChangeTables) {
    // Loop over all the abis calls to create table changes
    calls.factory_call_create_pair_1s.iter().for_each(|call| {
        tables
            .create_row("factory_call_create_pair1", [("call_tx_hash", call.call_tx_hash.to_string()),("call_ordinal", call.call_ordinal.to_string())])
            .set("call_block_time", call.call_block_time.as_ref().unwrap())
            .set("call_block_number", call.call_block_number)
            .set("call_success", call.call_success)
            .set("fee", BigDecimal::from_str(&call.fee).unwrap())
            .set("output_pair", Hex(&call.output_pair).to_string())
            .set("token_a", Hex(&call.token_a).to_string())
            .set("token_b", Hex(&call.token_b).to_string());
    });
    calls.factory_call_create_pair_2s.iter().for_each(|call| {
        tables
            .create_row("factory_call_create_pair2", [("call_tx_hash", call.call_tx_hash.to_string()),("call_ordinal", call.call_ordinal.to_string())])
            .set("call_block_time", call.call_block_time.as_ref().unwrap())
            .set("call_block_number", call.call_block_number)
            .set("call_success", call.call_success)
            .set("output_pair", Hex(&call.output_pair).to_string())
            .set("token_a", Hex(&call.token_a).to_string())
            .set("token_b", Hex(&call.token_b).to_string());
    });
    calls.factory_call_set_fee_tos.iter().for_each(|call| {
        tables
            .create_row("factory_call_set_fee_to", [("call_tx_hash", call.call_tx_hash.to_string()),("call_ordinal", call.call_ordinal.to_string())])
            .set("call_block_time", call.call_block_time.as_ref().unwrap())
            .set("call_block_number", call.call_block_number)
            .set("call_success", call.call_success)
            .set("u_fee_to", Hex(&call.u_fee_to).to_string());
    });
    calls.factory_call_set_fee_to_setters.iter().for_each(|call| {
        tables
            .create_row("factory_call_set_fee_to_setter", [("call_tx_hash", call.call_tx_hash.to_string()),("call_ordinal", call.call_ordinal.to_string())])
            .set("call_block_time", call.call_block_time.as_ref().unwrap())
            .set("call_block_number", call.call_block_number)
            .set("call_success", call.call_success)
            .set("u_fee_to_setter", Hex(&call.u_fee_to_setter).to_string());
    });
    calls.factory_call_toggle_global_pauses.iter().for_each(|call| {
        tables
            .create_row("factory_call_toggle_global_pause", [("call_tx_hash", call.call_tx_hash.to_string()),("call_ordinal", call.call_ordinal.to_string())])
            .set("call_block_time", call.call_block_time.as_ref().unwrap())
            .set("call_block_number", call.call_block_number)
            .set("call_success", call.call_success);
    });
}
fn db_pair_out(events: &contract::Events, tables: &mut DatabaseChangeTables) {
    // Loop over all the abis events to create table changes
    events.pair_approvals.iter().for_each(|evt| {
        tables
            .create_row("pair_approval", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("evt_address", &evt.evt_address)
            .set("owner", Hex(&evt.owner).to_string())
            .set("spender", Hex(&evt.spender).to_string())
            .set("value", BigDecimal::from_str(&evt.value).unwrap());
    });
    events.pair_burns.iter().for_each(|evt| {
        tables
            .create_row("pair_burn", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("evt_address", &evt.evt_address)
            .set("amount0", BigDecimal::from_str(&evt.amount0).unwrap())
            .set("amount1", BigDecimal::from_str(&evt.amount1).unwrap())
            .set("sender", Hex(&evt.sender).to_string())
            .set("to", Hex(&evt.to).to_string());
    });
    events.pair_cancel_long_term_orders.iter().for_each(|evt| {
        tables
            .create_row("pair_cancel_long_term_order", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("evt_address", &evt.evt_address)
            .set("addr", Hex(&evt.addr).to_string())
            .set("buy_token", Hex(&evt.buy_token).to_string())
            .set("order_id", BigDecimal::from_str(&evt.order_id).unwrap())
            .set("purchased_amount", BigDecimal::from_str(&evt.purchased_amount).unwrap())
            .set("sell_token", Hex(&evt.sell_token).to_string())
            .set("unsold_amount", BigDecimal::from_str(&evt.unsold_amount).unwrap());
    });
    events.pair_long_term_swap0_to_1s.iter().for_each(|evt| {
        tables
            .create_row("pair_long_term_swap0_to1", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("evt_address", &evt.evt_address)
            .set("addr", Hex(&evt.addr).to_string())
            .set("amount0_in", BigDecimal::from_str(&evt.amount0_in).unwrap())
            .set("number_of_time_intervals", BigDecimal::from_str(&evt.number_of_time_intervals).unwrap())
            .set("order_id", BigDecimal::from_str(&evt.order_id).unwrap());
    });
    events.pair_long_term_swap1_to_0s.iter().for_each(|evt| {
        tables
            .create_row("pair_long_term_swap1_to0", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("evt_address", &evt.evt_address)
            .set("addr", Hex(&evt.addr).to_string())
            .set("amount1_in", BigDecimal::from_str(&evt.amount1_in).unwrap())
            .set("number_of_time_intervals", BigDecimal::from_str(&evt.number_of_time_intervals).unwrap())
            .set("order_id", BigDecimal::from_str(&evt.order_id).unwrap());
    });
    events.pair_lp_fee_updateds.iter().for_each(|evt| {
        tables
            .create_row("pair_lp_fee_updated", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("evt_address", &evt.evt_address)
            .set("fee", BigDecimal::from_str(&evt.fee).unwrap());
    });
    events.pair_mints.iter().for_each(|evt| {
        tables
            .create_row("pair_mint", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("evt_address", &evt.evt_address)
            .set("amount0", BigDecimal::from_str(&evt.amount0).unwrap())
            .set("amount1", BigDecimal::from_str(&evt.amount1).unwrap())
            .set("sender", Hex(&evt.sender).to_string());
    });
    events.pair_swaps.iter().for_each(|evt| {
        tables
            .create_row("pair_swap", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("evt_address", &evt.evt_address)
            .set("amount0_in", BigDecimal::from_str(&evt.amount0_in).unwrap())
            .set("amount0_out", BigDecimal::from_str(&evt.amount0_out).unwrap())
            .set("amount1_in", BigDecimal::from_str(&evt.amount1_in).unwrap())
            .set("amount1_out", BigDecimal::from_str(&evt.amount1_out).unwrap())
            .set("sender", Hex(&evt.sender).to_string())
            .set("to", Hex(&evt.to).to_string());
    });
    events.pair_syncs.iter().for_each(|evt| {
        tables
            .create_row("pair_sync", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("evt_address", &evt.evt_address)
            .set("reserve0", BigDecimal::from_str(&evt.reserve0).unwrap())
            .set("reserve1", BigDecimal::from_str(&evt.reserve1).unwrap());
    });
    events.pair_transfers.iter().for_each(|evt| {
        tables
            .create_row("pair_transfer", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("evt_address", &evt.evt_address)
            .set("from", Hex(&evt.from).to_string())
            .set("to", Hex(&evt.to).to_string())
            .set("value", BigDecimal::from_str(&evt.value).unwrap());
    });
    events.pair_withdraw_proceeds_from_long_term_orders.iter().for_each(|evt| {
        tables
            .create_row("pair_withdraw_proceeds_from_long_term_order", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("evt_address", &evt.evt_address)
            .set("addr", Hex(&evt.addr).to_string())
            .set("order_expired", evt.order_expired)
            .set("order_id", BigDecimal::from_str(&evt.order_id).unwrap())
            .set("proceed_token", Hex(&evt.proceed_token).to_string())
            .set("proceeds", BigDecimal::from_str(&evt.proceeds).unwrap());
    });
}
fn db_pair_calls_out(calls: &contract::Calls, tables: &mut DatabaseChangeTables) {
    // Loop over all the abis calls to create table changes
    calls.pair_call_approves.iter().for_each(|call| {
        tables
            .create_row("pair_call_approve", [("call_tx_hash", call.call_tx_hash.to_string()),("call_ordinal", call.call_ordinal.to_string())])
            .set("call_block_time", call.call_block_time.as_ref().unwrap())
            .set("call_block_number", call.call_block_number)
            .set("call_success", call.call_success)
            .set("call_address", &call.call_address)
            .set("output_param0", call.output_param0)
            .set("spender", Hex(&call.spender).to_string())
            .set("value", BigDecimal::from_str(&call.value).unwrap());
    });
    calls.pair_call_burns.iter().for_each(|call| {
        tables
            .create_row("pair_call_burn", [("call_tx_hash", call.call_tx_hash.to_string()),("call_ordinal", call.call_ordinal.to_string())])
            .set("call_block_time", call.call_block_time.as_ref().unwrap())
            .set("call_block_number", call.call_block_number)
            .set("call_success", call.call_success)
            .set("call_address", &call.call_address)
            .set("output_amount0", BigDecimal::from_str(&call.output_amount0).unwrap())
            .set("output_amount1", BigDecimal::from_str(&call.output_amount1).unwrap())
            .set("to", Hex(&call.to).to_string());
    });
    calls.pair_call_cancel_long_term_swaps.iter().for_each(|call| {
        tables
            .create_row("pair_call_cancel_long_term_swap", [("call_tx_hash", call.call_tx_hash.to_string()),("call_ordinal", call.call_ordinal.to_string())])
            .set("call_block_time", call.call_block_time.as_ref().unwrap())
            .set("call_block_number", call.call_block_number)
            .set("call_success", call.call_success)
            .set("call_address", &call.call_address)
            .set("order_id", BigDecimal::from_str(&call.order_id).unwrap());
    });
    calls.pair_call_execute_virtual_orders.iter().for_each(|call| {
        tables
            .create_row("pair_call_execute_virtual_orders", [("call_tx_hash", call.call_tx_hash.to_string()),("call_ordinal", call.call_ordinal.to_string())])
            .set("call_block_time", call.call_block_time.as_ref().unwrap())
            .set("call_block_number", call.call_block_number)
            .set("call_success", call.call_success)
            .set("call_address", &call.call_address)
            .set("block_timestamp", BigDecimal::from_str(&call.block_timestamp).unwrap());
    });
    calls.pair_call_get_twamm_order_proceeds.iter().for_each(|call| {
        tables
            .create_row("pair_call_get_twamm_order_proceeds", [("call_tx_hash", call.call_tx_hash.to_string()),("call_ordinal", call.call_ordinal.to_string())])
            .set("call_block_time", call.call_block_time.as_ref().unwrap())
            .set("call_block_number", call.call_block_number)
            .set("call_success", call.call_success)
            .set("call_address", &call.call_address)
            .set("order_id", BigDecimal::from_str(&call.order_id).unwrap())
            .set("output_order_expired", call.output_order_expired)
            .set("output_total_reward", BigDecimal::from_str(&call.output_total_reward).unwrap());
    });
    calls.pair_call_initializes.iter().for_each(|call| {
        tables
            .create_row("pair_call_initialize", [("call_tx_hash", call.call_tx_hash.to_string()),("call_ordinal", call.call_ordinal.to_string())])
            .set("call_block_time", call.call_block_time.as_ref().unwrap())
            .set("call_block_number", call.call_block_number)
            .set("call_success", call.call_success)
            .set("call_address", &call.call_address)
            .set("u_fee", BigDecimal::from_str(&call.u_fee).unwrap())
            .set("u_token0", Hex(&call.u_token0).to_string())
            .set("u_token1", Hex(&call.u_token1).to_string());
    });
    calls.pair_call_long_term_swap_from0_to_1s.iter().for_each(|call| {
        tables
            .create_row("pair_call_long_term_swap_from0_to1", [("call_tx_hash", call.call_tx_hash.to_string()),("call_ordinal", call.call_ordinal.to_string())])
            .set("call_block_time", call.call_block_time.as_ref().unwrap())
            .set("call_block_number", call.call_block_number)
            .set("call_success", call.call_success)
            .set("call_address", &call.call_address)
            .set("amount0_in", BigDecimal::from_str(&call.amount0_in).unwrap())
            .set("number_of_time_intervals", BigDecimal::from_str(&call.number_of_time_intervals).unwrap())
            .set("output_order_id", BigDecimal::from_str(&call.output_order_id).unwrap());
    });
    calls.pair_call_long_term_swap_from1_to_0s.iter().for_each(|call| {
        tables
            .create_row("pair_call_long_term_swap_from1_to0", [("call_tx_hash", call.call_tx_hash.to_string()),("call_ordinal", call.call_ordinal.to_string())])
            .set("call_block_time", call.call_block_time.as_ref().unwrap())
            .set("call_block_number", call.call_block_number)
            .set("call_success", call.call_success)
            .set("call_address", &call.call_address)
            .set("amount1_in", BigDecimal::from_str(&call.amount1_in).unwrap())
            .set("number_of_time_intervals", BigDecimal::from_str(&call.number_of_time_intervals).unwrap())
            .set("output_order_id", BigDecimal::from_str(&call.output_order_id).unwrap());
    });
    calls.pair_call_mints.iter().for_each(|call| {
        tables
            .create_row("pair_call_mint", [("call_tx_hash", call.call_tx_hash.to_string()),("call_ordinal", call.call_ordinal.to_string())])
            .set("call_block_time", call.call_block_time.as_ref().unwrap())
            .set("call_block_number", call.call_block_number)
            .set("call_success", call.call_success)
            .set("call_address", &call.call_address)
            .set("output_liquidity", BigDecimal::from_str(&call.output_liquidity).unwrap())
            .set("to", Hex(&call.to).to_string());
    });
    calls.pair_call_permits.iter().for_each(|call| {
        tables
            .create_row("pair_call_permit", [("call_tx_hash", call.call_tx_hash.to_string()),("call_ordinal", call.call_ordinal.to_string())])
            .set("call_block_time", call.call_block_time.as_ref().unwrap())
            .set("call_block_number", call.call_block_number)
            .set("call_success", call.call_success)
            .set("call_address", &call.call_address)
            .set("deadline", BigDecimal::from_str(&call.deadline).unwrap())
            .set("owner", Hex(&call.owner).to_string())
            .set("r", Hex(&call.r).to_string())
            .set("s", Hex(&call.s).to_string())
            .set("spender", Hex(&call.spender).to_string())
            .set("v", call.v)
            .set("value", BigDecimal::from_str(&call.value).unwrap());
    });
    calls.pair_call_set_fees.iter().for_each(|call| {
        tables
            .create_row("pair_call_set_fee", [("call_tx_hash", call.call_tx_hash.to_string()),("call_ordinal", call.call_ordinal.to_string())])
            .set("call_block_time", call.call_block_time.as_ref().unwrap())
            .set("call_block_number", call.call_block_number)
            .set("call_success", call.call_success)
            .set("call_address", &call.call_address)
            .set("new_fee", BigDecimal::from_str(&call.new_fee).unwrap());
    });
    calls.pair_call_skims.iter().for_each(|call| {
        tables
            .create_row("pair_call_skim", [("call_tx_hash", call.call_tx_hash.to_string()),("call_ordinal", call.call_ordinal.to_string())])
            .set("call_block_time", call.call_block_time.as_ref().unwrap())
            .set("call_block_number", call.call_block_number)
            .set("call_success", call.call_success)
            .set("call_address", &call.call_address)
            .set("to", Hex(&call.to).to_string());
    });
    calls.pair_call_swaps.iter().for_each(|call| {
        tables
            .create_row("pair_call_swap", [("call_tx_hash", call.call_tx_hash.to_string()),("call_ordinal", call.call_ordinal.to_string())])
            .set("call_block_time", call.call_block_time.as_ref().unwrap())
            .set("call_block_number", call.call_block_number)
            .set("call_success", call.call_success)
            .set("call_address", &call.call_address)
            .set("amount0_out", BigDecimal::from_str(&call.amount0_out).unwrap())
            .set("amount1_out", BigDecimal::from_str(&call.amount1_out).unwrap())
            .set("data", Hex(&call.data).to_string())
            .set("to", Hex(&call.to).to_string());
    });
    calls.pair_call_syncs.iter().for_each(|call| {
        tables
            .create_row("pair_call_sync", [("call_tx_hash", call.call_tx_hash.to_string()),("call_ordinal", call.call_ordinal.to_string())])
            .set("call_block_time", call.call_block_time.as_ref().unwrap())
            .set("call_block_number", call.call_block_number)
            .set("call_success", call.call_success)
            .set("call_address", &call.call_address);
    });
    calls.pair_call_toggle_pause_new_swaps.iter().for_each(|call| {
        tables
            .create_row("pair_call_toggle_pause_new_swaps", [("call_tx_hash", call.call_tx_hash.to_string()),("call_ordinal", call.call_ordinal.to_string())])
            .set("call_block_time", call.call_block_time.as_ref().unwrap())
            .set("call_block_number", call.call_block_number)
            .set("call_success", call.call_success)
            .set("call_address", &call.call_address);
    });
    calls.pair_call_transfers.iter().for_each(|call| {
        tables
            .create_row("pair_call_transfer", [("call_tx_hash", call.call_tx_hash.to_string()),("call_ordinal", call.call_ordinal.to_string())])
            .set("call_block_time", call.call_block_time.as_ref().unwrap())
            .set("call_block_number", call.call_block_number)
            .set("call_success", call.call_success)
            .set("call_address", &call.call_address)
            .set("output_param0", call.output_param0)
            .set("to", Hex(&call.to).to_string())
            .set("value", BigDecimal::from_str(&call.value).unwrap());
    });
    calls.pair_call_transfer_froms.iter().for_each(|call| {
        tables
            .create_row("pair_call_transfer_from", [("call_tx_hash", call.call_tx_hash.to_string()),("call_ordinal", call.call_ordinal.to_string())])
            .set("call_block_time", call.call_block_time.as_ref().unwrap())
            .set("call_block_number", call.call_block_number)
            .set("call_success", call.call_success)
            .set("call_address", &call.call_address)
            .set("from", Hex(&call.from).to_string())
            .set("output_param0", call.output_param0)
            .set("to", Hex(&call.to).to_string())
            .set("value", BigDecimal::from_str(&call.value).unwrap());
    });
    calls.pair_call_withdraw_proceeds_from_long_term_swaps.iter().for_each(|call| {
        tables
            .create_row("pair_call_withdraw_proceeds_from_long_term_swap", [("call_tx_hash", call.call_tx_hash.to_string()),("call_ordinal", call.call_ordinal.to_string())])
            .set("call_block_time", call.call_block_time.as_ref().unwrap())
            .set("call_block_number", call.call_block_number)
            .set("call_success", call.call_success)
            .set("call_address", &call.call_address)
            .set("order_id", BigDecimal::from_str(&call.order_id).unwrap())
            .set("output_is_expired", call.output_is_expired)
            .set("output_reward_tkn", Hex(&call.output_reward_tkn).to_string())
            .set("output_total_reward", BigDecimal::from_str(&call.output_total_reward).unwrap());
    });
}


fn graph_factory_out(events: &contract::Events, tables: &mut EntityChangesTables) {
    // Loop over all the abis events to create table changes
    events.factory_pair_createds.iter().for_each(|evt| {
        tables
            .create_row("factory_pair_created", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("pair", Hex(&evt.pair).to_string())
            .set("param3", BigDecimal::from_str(&evt.param3).unwrap())
            .set("token0", Hex(&evt.token0).to_string())
            .set("token1", Hex(&evt.token1).to_string());
    });
}
fn graph_factory_calls_out(calls: &contract::Calls, tables: &mut EntityChangesTables) {
    // Loop over all the abis calls to create table changes
    calls.factory_call_create_pair_1s.iter().for_each(|call| {
        tables
            .create_row("factory_call_create_pair1", format!("{}-{}", call.call_tx_hash, call.call_ordinal))
            .set("call_tx_hash", &call.call_tx_hash)
            .set("call_ordinal", call.call_ordinal)
            .set("call_block_time", call.call_block_time.as_ref().unwrap())
            .set("call_block_number", call.call_block_number)
            .set("call_success", call.call_success)
            .set("fee", BigDecimal::from_str(&call.fee).unwrap())
            .set("output_pair", Hex(&call.output_pair).to_string())
            .set("token_a", Hex(&call.token_a).to_string())
            .set("token_b", Hex(&call.token_b).to_string());
    });
    calls.factory_call_create_pair_2s.iter().for_each(|call| {
        tables
            .create_row("factory_call_create_pair2", format!("{}-{}", call.call_tx_hash, call.call_ordinal))
            .set("call_tx_hash", &call.call_tx_hash)
            .set("call_ordinal", call.call_ordinal)
            .set("call_block_time", call.call_block_time.as_ref().unwrap())
            .set("call_block_number", call.call_block_number)
            .set("call_success", call.call_success)
            .set("output_pair", Hex(&call.output_pair).to_string())
            .set("token_a", Hex(&call.token_a).to_string())
            .set("token_b", Hex(&call.token_b).to_string());
    });
    calls.factory_call_set_fee_tos.iter().for_each(|call| {
        tables
            .create_row("factory_call_set_fee_to", format!("{}-{}", call.call_tx_hash, call.call_ordinal))
            .set("call_tx_hash", &call.call_tx_hash)
            .set("call_ordinal", call.call_ordinal)
            .set("call_block_time", call.call_block_time.as_ref().unwrap())
            .set("call_block_number", call.call_block_number)
            .set("call_success", call.call_success)
            .set("u_fee_to", Hex(&call.u_fee_to).to_string());
    });
    calls.factory_call_set_fee_to_setters.iter().for_each(|call| {
        tables
            .create_row("factory_call_set_fee_to_setter", format!("{}-{}", call.call_tx_hash, call.call_ordinal))
            .set("call_tx_hash", &call.call_tx_hash)
            .set("call_ordinal", call.call_ordinal)
            .set("call_block_time", call.call_block_time.as_ref().unwrap())
            .set("call_block_number", call.call_block_number)
            .set("call_success", call.call_success)
            .set("u_fee_to_setter", Hex(&call.u_fee_to_setter).to_string());
    });
    calls.factory_call_toggle_global_pauses.iter().for_each(|call| {
        tables
            .create_row("factory_call_toggle_global_pause", format!("{}-{}", call.call_tx_hash, call.call_ordinal))
            .set("call_tx_hash", &call.call_tx_hash)
            .set("call_ordinal", call.call_ordinal)
            .set("call_block_time", call.call_block_time.as_ref().unwrap())
            .set("call_block_number", call.call_block_number)
            .set("call_success", call.call_success);
    });
  }
fn graph_pair_out(events: &contract::Events, tables: &mut EntityChangesTables) {
    // Loop over all the abis events to create table changes
    events.pair_approvals.iter().for_each(|evt| {
        tables
            .create_row("pair_approval", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("evt_address", &evt.evt_address)
            .set("owner", Hex(&evt.owner).to_string())
            .set("spender", Hex(&evt.spender).to_string())
            .set("value", BigDecimal::from_str(&evt.value).unwrap());
    });
    events.pair_burns.iter().for_each(|evt| {
        tables
            .create_row("pair_burn", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
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
    events.pair_cancel_long_term_orders.iter().for_each(|evt| {
        tables
            .create_row("pair_cancel_long_term_order", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("evt_address", &evt.evt_address)
            .set("addr", Hex(&evt.addr).to_string())
            .set("buy_token", Hex(&evt.buy_token).to_string())
            .set("order_id", BigDecimal::from_str(&evt.order_id).unwrap())
            .set("purchased_amount", BigDecimal::from_str(&evt.purchased_amount).unwrap())
            .set("sell_token", Hex(&evt.sell_token).to_string())
            .set("unsold_amount", BigDecimal::from_str(&evt.unsold_amount).unwrap());
    });
    events.pair_long_term_swap0_to_1s.iter().for_each(|evt| {
        tables
            .create_row("pair_long_term_swap0_to1", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("evt_address", &evt.evt_address)
            .set("addr", Hex(&evt.addr).to_string())
            .set("amount0_in", BigDecimal::from_str(&evt.amount0_in).unwrap())
            .set("number_of_time_intervals", BigDecimal::from_str(&evt.number_of_time_intervals).unwrap())
            .set("order_id", BigDecimal::from_str(&evt.order_id).unwrap());
    });
    events.pair_long_term_swap1_to_0s.iter().for_each(|evt| {
        tables
            .create_row("pair_long_term_swap1_to0", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("evt_address", &evt.evt_address)
            .set("addr", Hex(&evt.addr).to_string())
            .set("amount1_in", BigDecimal::from_str(&evt.amount1_in).unwrap())
            .set("number_of_time_intervals", BigDecimal::from_str(&evt.number_of_time_intervals).unwrap())
            .set("order_id", BigDecimal::from_str(&evt.order_id).unwrap());
    });
    events.pair_lp_fee_updateds.iter().for_each(|evt| {
        tables
            .create_row("pair_lp_fee_updated", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("evt_address", &evt.evt_address)
            .set("fee", BigDecimal::from_str(&evt.fee).unwrap());
    });
    events.pair_mints.iter().for_each(|evt| {
        tables
            .create_row("pair_mint", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("evt_address", &evt.evt_address)
            .set("amount0", BigDecimal::from_str(&evt.amount0).unwrap())
            .set("amount1", BigDecimal::from_str(&evt.amount1).unwrap())
            .set("sender", Hex(&evt.sender).to_string());
    });
    events.pair_swaps.iter().for_each(|evt| {
        tables
            .create_row("pair_swap", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("evt_address", &evt.evt_address)
            .set("amount0_in", BigDecimal::from_str(&evt.amount0_in).unwrap())
            .set("amount0_out", BigDecimal::from_str(&evt.amount0_out).unwrap())
            .set("amount1_in", BigDecimal::from_str(&evt.amount1_in).unwrap())
            .set("amount1_out", BigDecimal::from_str(&evt.amount1_out).unwrap())
            .set("sender", Hex(&evt.sender).to_string())
            .set("to", Hex(&evt.to).to_string());
    });
    events.pair_syncs.iter().for_each(|evt| {
        tables
            .create_row("pair_sync", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("evt_address", &evt.evt_address)
            .set("reserve0", BigDecimal::from_str(&evt.reserve0).unwrap())
            .set("reserve1", BigDecimal::from_str(&evt.reserve1).unwrap());
    });
    events.pair_transfers.iter().for_each(|evt| {
        tables
            .create_row("pair_transfer", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("evt_address", &evt.evt_address)
            .set("from", Hex(&evt.from).to_string())
            .set("to", Hex(&evt.to).to_string())
            .set("value", BigDecimal::from_str(&evt.value).unwrap());
    });
    events.pair_withdraw_proceeds_from_long_term_orders.iter().for_each(|evt| {
        tables
            .create_row("pair_withdraw_proceeds_from_long_term_order", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", evt.evt_block_number)
            .set("evt_address", &evt.evt_address)
            .set("addr", Hex(&evt.addr).to_string())
            .set("order_expired", evt.order_expired)
            .set("order_id", BigDecimal::from_str(&evt.order_id).unwrap())
            .set("proceed_token", Hex(&evt.proceed_token).to_string())
            .set("proceeds", BigDecimal::from_str(&evt.proceeds).unwrap());
    });
}
fn graph_pair_calls_out(calls: &contract::Calls, tables: &mut EntityChangesTables) {
    // Loop over all the abis calls to create table changes
    calls.pair_call_approves.iter().for_each(|call| {
        tables
            .create_row("pair_call_approve", format!("{}-{}", call.call_tx_hash, call.call_ordinal))
            .set("call_tx_hash", &call.call_tx_hash)
            .set("call_ordinal", call.call_ordinal)
            .set("call_block_time", call.call_block_time.as_ref().unwrap())
            .set("call_block_number", call.call_block_number)
            .set("call_success", call.call_success)
            .set("call_address", &call.call_address)
            .set("output_param0", call.output_param0)
            .set("spender", Hex(&call.spender).to_string())
            .set("value", BigDecimal::from_str(&call.value).unwrap());
    });
    calls.pair_call_burns.iter().for_each(|call| {
        tables
            .create_row("pair_call_burn", format!("{}-{}", call.call_tx_hash, call.call_ordinal))
            .set("call_tx_hash", &call.call_tx_hash)
            .set("call_ordinal", call.call_ordinal)
            .set("call_block_time", call.call_block_time.as_ref().unwrap())
            .set("call_block_number", call.call_block_number)
            .set("call_success", call.call_success)
            .set("call_address", &call.call_address)
            .set("output_amount0", BigDecimal::from_str(&call.output_amount0).unwrap())
            .set("output_amount1", BigDecimal::from_str(&call.output_amount1).unwrap())
            .set("to", Hex(&call.to).to_string());
    });
    calls.pair_call_cancel_long_term_swaps.iter().for_each(|call| {
        tables
            .create_row("pair_call_cancel_long_term_swap", format!("{}-{}", call.call_tx_hash, call.call_ordinal))
            .set("call_tx_hash", &call.call_tx_hash)
            .set("call_ordinal", call.call_ordinal)
            .set("call_block_time", call.call_block_time.as_ref().unwrap())
            .set("call_block_number", call.call_block_number)
            .set("call_success", call.call_success)
            .set("call_address", &call.call_address)
            .set("order_id", BigDecimal::from_str(&call.order_id).unwrap());
    });
    calls.pair_call_execute_virtual_orders.iter().for_each(|call| {
        tables
            .create_row("pair_call_execute_virtual_orders", format!("{}-{}", call.call_tx_hash, call.call_ordinal))
            .set("call_tx_hash", &call.call_tx_hash)
            .set("call_ordinal", call.call_ordinal)
            .set("call_block_time", call.call_block_time.as_ref().unwrap())
            .set("call_block_number", call.call_block_number)
            .set("call_success", call.call_success)
            .set("call_address", &call.call_address)
            .set("block_timestamp", BigDecimal::from_str(&call.block_timestamp).unwrap());
    });
    calls.pair_call_get_twamm_order_proceeds.iter().for_each(|call| {
        tables
            .create_row("pair_call_get_twamm_order_proceeds", format!("{}-{}", call.call_tx_hash, call.call_ordinal))
            .set("call_tx_hash", &call.call_tx_hash)
            .set("call_ordinal", call.call_ordinal)
            .set("call_block_time", call.call_block_time.as_ref().unwrap())
            .set("call_block_number", call.call_block_number)
            .set("call_success", call.call_success)
            .set("call_address", &call.call_address)
            .set("order_id", BigDecimal::from_str(&call.order_id).unwrap())
            .set("output_order_expired", call.output_order_expired)
            .set("output_total_reward", BigDecimal::from_str(&call.output_total_reward).unwrap());
    });
    calls.pair_call_initializes.iter().for_each(|call| {
        tables
            .create_row("pair_call_initialize", format!("{}-{}", call.call_tx_hash, call.call_ordinal))
            .set("call_tx_hash", &call.call_tx_hash)
            .set("call_ordinal", call.call_ordinal)
            .set("call_block_time", call.call_block_time.as_ref().unwrap())
            .set("call_block_number", call.call_block_number)
            .set("call_success", call.call_success)
            .set("call_address", &call.call_address)
            .set("u_fee", BigDecimal::from_str(&call.u_fee).unwrap())
            .set("u_token0", Hex(&call.u_token0).to_string())
            .set("u_token1", Hex(&call.u_token1).to_string());
    });
    calls.pair_call_long_term_swap_from0_to_1s.iter().for_each(|call| {
        tables
            .create_row("pair_call_long_term_swap_from0_to1", format!("{}-{}", call.call_tx_hash, call.call_ordinal))
            .set("call_tx_hash", &call.call_tx_hash)
            .set("call_ordinal", call.call_ordinal)
            .set("call_block_time", call.call_block_time.as_ref().unwrap())
            .set("call_block_number", call.call_block_number)
            .set("call_success", call.call_success)
            .set("call_address", &call.call_address)
            .set("amount0_in", BigDecimal::from_str(&call.amount0_in).unwrap())
            .set("number_of_time_intervals", BigDecimal::from_str(&call.number_of_time_intervals).unwrap())
            .set("output_order_id", BigDecimal::from_str(&call.output_order_id).unwrap());
    });
    calls.pair_call_long_term_swap_from1_to_0s.iter().for_each(|call| {
        tables
            .create_row("pair_call_long_term_swap_from1_to0", format!("{}-{}", call.call_tx_hash, call.call_ordinal))
            .set("call_tx_hash", &call.call_tx_hash)
            .set("call_ordinal", call.call_ordinal)
            .set("call_block_time", call.call_block_time.as_ref().unwrap())
            .set("call_block_number", call.call_block_number)
            .set("call_success", call.call_success)
            .set("call_address", &call.call_address)
            .set("amount1_in", BigDecimal::from_str(&call.amount1_in).unwrap())
            .set("number_of_time_intervals", BigDecimal::from_str(&call.number_of_time_intervals).unwrap())
            .set("output_order_id", BigDecimal::from_str(&call.output_order_id).unwrap());
    });
    calls.pair_call_mints.iter().for_each(|call| {
        tables
            .create_row("pair_call_mint", format!("{}-{}", call.call_tx_hash, call.call_ordinal))
            .set("call_tx_hash", &call.call_tx_hash)
            .set("call_ordinal", call.call_ordinal)
            .set("call_block_time", call.call_block_time.as_ref().unwrap())
            .set("call_block_number", call.call_block_number)
            .set("call_success", call.call_success)
            .set("call_address", &call.call_address)
            .set("output_liquidity", BigDecimal::from_str(&call.output_liquidity).unwrap())
            .set("to", Hex(&call.to).to_string());
    });
    calls.pair_call_permits.iter().for_each(|call| {
        tables
            .create_row("pair_call_permit", format!("{}-{}", call.call_tx_hash, call.call_ordinal))
            .set("call_tx_hash", &call.call_tx_hash)
            .set("call_ordinal", call.call_ordinal)
            .set("call_block_time", call.call_block_time.as_ref().unwrap())
            .set("call_block_number", call.call_block_number)
            .set("call_success", call.call_success)
            .set("call_address", &call.call_address)
            .set("deadline", BigDecimal::from_str(&call.deadline).unwrap())
            .set("owner", Hex(&call.owner).to_string())
            .set("r", Hex(&call.r).to_string())
            .set("s", Hex(&call.s).to_string())
            .set("spender", Hex(&call.spender).to_string())
            .set("v", call.v)
            .set("value", BigDecimal::from_str(&call.value).unwrap());
    });
    calls.pair_call_set_fees.iter().for_each(|call| {
        tables
            .create_row("pair_call_set_fee", format!("{}-{}", call.call_tx_hash, call.call_ordinal))
            .set("call_tx_hash", &call.call_tx_hash)
            .set("call_ordinal", call.call_ordinal)
            .set("call_block_time", call.call_block_time.as_ref().unwrap())
            .set("call_block_number", call.call_block_number)
            .set("call_success", call.call_success)
            .set("call_address", &call.call_address)
            .set("new_fee", BigDecimal::from_str(&call.new_fee).unwrap());
    });
    calls.pair_call_skims.iter().for_each(|call| {
        tables
            .create_row("pair_call_skim", format!("{}-{}", call.call_tx_hash, call.call_ordinal))
            .set("call_tx_hash", &call.call_tx_hash)
            .set("call_ordinal", call.call_ordinal)
            .set("call_block_time", call.call_block_time.as_ref().unwrap())
            .set("call_block_number", call.call_block_number)
            .set("call_success", call.call_success)
            .set("call_address", &call.call_address)
            .set("to", Hex(&call.to).to_string());
    });
    calls.pair_call_swaps.iter().for_each(|call| {
        tables
            .create_row("pair_call_swap", format!("{}-{}", call.call_tx_hash, call.call_ordinal))
            .set("call_tx_hash", &call.call_tx_hash)
            .set("call_ordinal", call.call_ordinal)
            .set("call_block_time", call.call_block_time.as_ref().unwrap())
            .set("call_block_number", call.call_block_number)
            .set("call_success", call.call_success)
            .set("call_address", &call.call_address)
            .set("amount0_out", BigDecimal::from_str(&call.amount0_out).unwrap())
            .set("amount1_out", BigDecimal::from_str(&call.amount1_out).unwrap())
            .set("data", Hex(&call.data).to_string())
            .set("to", Hex(&call.to).to_string());
    });
    calls.pair_call_syncs.iter().for_each(|call| {
        tables
            .create_row("pair_call_sync", format!("{}-{}", call.call_tx_hash, call.call_ordinal))
            .set("call_tx_hash", &call.call_tx_hash)
            .set("call_ordinal", call.call_ordinal)
            .set("call_block_time", call.call_block_time.as_ref().unwrap())
            .set("call_block_number", call.call_block_number)
            .set("call_success", call.call_success)
            .set("call_address", &call.call_address);
    });
    calls.pair_call_toggle_pause_new_swaps.iter().for_each(|call| {
        tables
            .create_row("pair_call_toggle_pause_new_swaps", format!("{}-{}", call.call_tx_hash, call.call_ordinal))
            .set("call_tx_hash", &call.call_tx_hash)
            .set("call_ordinal", call.call_ordinal)
            .set("call_block_time", call.call_block_time.as_ref().unwrap())
            .set("call_block_number", call.call_block_number)
            .set("call_success", call.call_success)
            .set("call_address", &call.call_address);
    });
    calls.pair_call_transfers.iter().for_each(|call| {
        tables
            .create_row("pair_call_transfer", format!("{}-{}", call.call_tx_hash, call.call_ordinal))
            .set("call_tx_hash", &call.call_tx_hash)
            .set("call_ordinal", call.call_ordinal)
            .set("call_block_time", call.call_block_time.as_ref().unwrap())
            .set("call_block_number", call.call_block_number)
            .set("call_success", call.call_success)
            .set("call_address", &call.call_address)
            .set("output_param0", call.output_param0)
            .set("to", Hex(&call.to).to_string())
            .set("value", BigDecimal::from_str(&call.value).unwrap());
    });
    calls.pair_call_transfer_froms.iter().for_each(|call| {
        tables
            .create_row("pair_call_transfer_from", format!("{}-{}", call.call_tx_hash, call.call_ordinal))
            .set("call_tx_hash", &call.call_tx_hash)
            .set("call_ordinal", call.call_ordinal)
            .set("call_block_time", call.call_block_time.as_ref().unwrap())
            .set("call_block_number", call.call_block_number)
            .set("call_success", call.call_success)
            .set("call_address", &call.call_address)
            .set("from", Hex(&call.from).to_string())
            .set("output_param0", call.output_param0)
            .set("to", Hex(&call.to).to_string())
            .set("value", BigDecimal::from_str(&call.value).unwrap());
    });
    calls.pair_call_withdraw_proceeds_from_long_term_swaps.iter().for_each(|call| {
        tables
            .create_row("pair_call_withdraw_proceeds_from_long_term_swap", format!("{}-{}", call.call_tx_hash, call.call_ordinal))
            .set("call_tx_hash", &call.call_tx_hash)
            .set("call_ordinal", call.call_ordinal)
            .set("call_block_time", call.call_block_time.as_ref().unwrap())
            .set("call_block_number", call.call_block_number)
            .set("call_success", call.call_success)
            .set("call_address", &call.call_address)
            .set("order_id", BigDecimal::from_str(&call.order_id).unwrap())
            .set("output_is_expired", call.output_is_expired)
            .set("output_reward_tkn", Hex(&call.output_reward_tkn).to_string())
            .set("output_total_reward", BigDecimal::from_str(&call.output_total_reward).unwrap());
    });
  }
#[substreams::handlers::store]
fn store_factory_pair_created(blk: eth::Block, store: StoreSetInt64) {
    for rcpt in blk.receipts() {
        for log in rcpt
            .receipt
            .logs
            .iter()
            .filter(|log| log.address == FACTORY_TRACKED_CONTRACT)
        {
            if let Some(event) = abi::factory_contract::events::PairCreated::match_and_decode(log) {
                store.set(log.ordinal, Hex(event.pair).to_string(), &1);
            }
        }
    }
}

#[substreams::handlers::map]
fn map_events(
    blk: eth::Block,
    store_pair: StoreGetInt64,
) -> Result<contract::Events, substreams::errors::Error> {
    let mut events = contract::Events::default();
    map_factory_events(&blk, &mut events);
    map_pair_events(&blk, &store_pair, &mut events);
    Ok(events)
}
#[substreams::handlers::map]
fn map_calls(
    blk: eth::Block,
    store_pair: StoreGetInt64,
) -> Result<contract::Calls, substreams::errors::Error> {
    let mut calls = contract::Calls::default();
    map_factory_calls(&blk, &mut calls);
    map_pair_calls(&blk, &store_pair, &mut calls);
    Ok(calls)
}

#[substreams::handlers::map]
fn db_out(events: contract::Events, calls: contract::Calls) -> Result<DatabaseChanges, substreams::errors::Error> {
    // Initialize Database Changes container
    let mut tables = DatabaseChangeTables::new();
    db_factory_out(&events, &mut tables);
    db_factory_calls_out(&calls, &mut tables);
    db_pair_out(&events, &mut tables);
    db_pair_calls_out(&calls, &mut tables);
    Ok(tables.to_database_changes())
}

#[substreams::handlers::map]
fn graph_out(events: contract::Events, calls: contract::Calls) -> Result<EntityChanges, substreams::errors::Error> {
    // Initialize Database Changes container
    let mut tables = EntityChangesTables::new();
    graph_factory_out(&events, &mut tables);
    graph_factory_calls_out(&calls, &mut tables);
    graph_pair_out(&events, &mut tables);
    graph_pair_calls_out(&calls, &mut tables);
    Ok(tables.to_entity_changes())
}
