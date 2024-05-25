mod abi;
mod modules;
// mod pool_factories;
// mod abi;
// mod pb;
// use hex_literal::hex;
// use pb::contract::v1 as contract;
// use substreams::Hex;
// use substreams_database_change::pb::database::DatabaseChanges;
// use substreams_database_change::tables::Tables as DatabaseChangeTables;
// use substreams_entity_change::pb::entity::EntityChanges;
// use substreams_entity_change::tables::Tables as EntityChangesTables;
// use substreams_ethereum::pb::eth::v2 as eth;
// use substreams_ethereum::Event;

// #[allow(unused_imports)]
// use num_traits::cast::ToPrimitive;
// use std::str::FromStr;
// use substreams::scalar::BigDecimal;

// substreams_ethereum::init!();

// const WSTETH_TRACKED_CONTRACT: [u8; 20] = hex!("7f39c581f595b53c5cb19bd0b3f8da6c935e2ca0");

// fn map_wsteth_events(blk: &eth::Block, events: &mut contract::Events) {
//     events.wsteth_approvals.append(&mut blk
//         .receipts()
//         .flat_map(|view| {
//             view.receipt.logs.iter()
//                 .filter(|log| log.address == WSTETH_TRACKED_CONTRACT)
//                 .filter_map(|log| {
//                     if let Some(event) = abi::wsteth_contract::events::Approval::match_and_decode(log) {
//                         return Some(contract::WstethApproval {
//                             evt_tx_hash: Hex(&view.transaction.hash).to_string(),
//                             evt_index: log.block_index,
//                             evt_block_time: Some(blk.timestamp().to_owned()),
//                             evt_block_number: blk.number,
//                             owner: event.owner,
//                             spender: event.spender,
//                             value: event.value.to_string(),
//                         });
//                     }

//                     None
//                 })
//         })
//         .collect());
//     events.wsteth_transfers.append(&mut blk
//         .receipts()
//         .flat_map(|view| {
//             view.receipt.logs.iter()
//                 .filter(|log| log.address == WSTETH_TRACKED_CONTRACT)
//                 .filter_map(|log| {
//                     if let Some(event) = abi::wsteth_contract::events::Transfer::match_and_decode(log) {
//                         return Some(contract::WstethTransfer {
//                             evt_tx_hash: Hex(&view.transaction.hash).to_string(),
//                             evt_index: log.block_index,
//                             evt_block_time: Some(blk.timestamp().to_owned()),
//                             evt_block_number: blk.number,
//                             from: event.from,
//                             to: event.to,
//                             value: event.value.to_string(),
//                         });
//                     }

//                     None
//                 })
//         })
//         .collect());
// }

// fn map_wsteth_calls(blk: &eth::Block, calls: &mut contract::Calls) {
//     calls.wsteth_call_approves.append(&mut blk
//         .transactions()
//         .flat_map(|tx| {
//             tx.calls.iter()
//                 .filter(|call| call.address == WSTETH_TRACKED_CONTRACT && abi::wsteth_contract::functions::Approve::match_call(call))
//                 .filter_map(|call| {
//                     match abi::wsteth_contract::functions::Approve::decode(call) {
//                         Ok(decoded_call) => {
//                             let output_param0 = match abi::wsteth_contract::functions::Approve::output(&call.return_data) {
//                                 Ok(output_param0) => {output_param0}
//                                 Err(_) => Default::default(),
//                             };
                            
//                             Some(contract::WstethApproveCall {
//                                 call_tx_hash: Hex(&tx.hash).to_string(),
//                                 call_block_time: Some(blk.timestamp().to_owned()),
//                                 call_block_number: blk.number,
//                                 call_ordinal: call.begin_ordinal,
//                                 call_success: !call.state_reverted,
//                                 amount: decoded_call.amount.to_string(),
//                                 output_param0: output_param0,
//                                 spender: decoded_call.spender,
//                             })
//                         },
//                         Err(_) => None,
//                     }
//                 })
//         })
//         .collect());
//     calls.wsteth_call_decrease_allowances.append(&mut blk
//         .transactions()
//         .flat_map(|tx| {
//             tx.calls.iter()
//                 .filter(|call| call.address == WSTETH_TRACKED_CONTRACT && abi::wsteth_contract::functions::DecreaseAllowance::match_call(call))
//                 .filter_map(|call| {
//                     match abi::wsteth_contract::functions::DecreaseAllowance::decode(call) {
//                         Ok(decoded_call) => {
//                             let output_param0 = match abi::wsteth_contract::functions::DecreaseAllowance::output(&call.return_data) {
//                                 Ok(output_param0) => {output_param0}
//                                 Err(_) => Default::default(),
//                             };
                            
//                             Some(contract::WstethDecreaseAllowanceCall {
//                                 call_tx_hash: Hex(&tx.hash).to_string(),
//                                 call_block_time: Some(blk.timestamp().to_owned()),
//                                 call_block_number: blk.number,
//                                 call_ordinal: call.begin_ordinal,
//                                 call_success: !call.state_reverted,
//                                 output_param0: output_param0,
//                                 spender: decoded_call.spender,
//                                 subtracted_value: decoded_call.subtracted_value.to_string(),
//                             })
//                         },
//                         Err(_) => None,
//                     }
//                 })
//         })
//         .collect());
//     calls.wsteth_call_increase_allowances.append(&mut blk
//         .transactions()
//         .flat_map(|tx| {
//             tx.calls.iter()
//                 .filter(|call| call.address == WSTETH_TRACKED_CONTRACT && abi::wsteth_contract::functions::IncreaseAllowance::match_call(call))
//                 .filter_map(|call| {
//                     match abi::wsteth_contract::functions::IncreaseAllowance::decode(call) {
//                         Ok(decoded_call) => {
//                             let output_param0 = match abi::wsteth_contract::functions::IncreaseAllowance::output(&call.return_data) {
//                                 Ok(output_param0) => {output_param0}
//                                 Err(_) => Default::default(),
//                             };
                            
//                             Some(contract::WstethIncreaseAllowanceCall {
//                                 call_tx_hash: Hex(&tx.hash).to_string(),
//                                 call_block_time: Some(blk.timestamp().to_owned()),
//                                 call_block_number: blk.number,
//                                 call_ordinal: call.begin_ordinal,
//                                 call_success: !call.state_reverted,
//                                 added_value: decoded_call.added_value.to_string(),
//                                 output_param0: output_param0,
//                                 spender: decoded_call.spender,
//                             })
//                         },
//                         Err(_) => None,
//                     }
//                 })
//         })
//         .collect());
//     calls.wsteth_call_permits.append(&mut blk
//         .transactions()
//         .flat_map(|tx| {
//             tx.calls.iter()
//                 .filter(|call| call.address == WSTETH_TRACKED_CONTRACT && abi::wsteth_contract::functions::Permit::match_call(call))
//                 .filter_map(|call| {
//                     match abi::wsteth_contract::functions::Permit::decode(call) {
//                         Ok(decoded_call) => {
//                             Some(contract::WstethPermitCall {
//                                 call_tx_hash: Hex(&tx.hash).to_string(),
//                                 call_block_time: Some(blk.timestamp().to_owned()),
//                                 call_block_number: blk.number,
//                                 call_ordinal: call.begin_ordinal,
//                                 call_success: !call.state_reverted,
//                                 deadline: decoded_call.deadline.to_string(),
//                                 owner: decoded_call.owner,
//                                 r: Vec::from(decoded_call.r),
//                                 s: Vec::from(decoded_call.s),
//                                 spender: decoded_call.spender,
//                                 v: decoded_call.v.to_u64(),
//                                 value: decoded_call.value.to_string(),
//                             })
//                         },
//                         Err(_) => None,
//                     }
//                 })
//         })
//         .collect());
//     calls.wsteth_call_transfers.append(&mut blk
//         .transactions()
//         .flat_map(|tx| {
//             tx.calls.iter()
//                 .filter(|call| call.address == WSTETH_TRACKED_CONTRACT && abi::wsteth_contract::functions::Transfer::match_call(call))
//                 .filter_map(|call| {
//                     match abi::wsteth_contract::functions::Transfer::decode(call) {
//                         Ok(decoded_call) => {
//                             let output_param0 = match abi::wsteth_contract::functions::Transfer::output(&call.return_data) {
//                                 Ok(output_param0) => {output_param0}
//                                 Err(_) => Default::default(),
//                             };
                            
//                             Some(contract::WstethTransferCall {
//                                 call_tx_hash: Hex(&tx.hash).to_string(),
//                                 call_block_time: Some(blk.timestamp().to_owned()),
//                                 call_block_number: blk.number,
//                                 call_ordinal: call.begin_ordinal,
//                                 call_success: !call.state_reverted,
//                                 amount: decoded_call.amount.to_string(),
//                                 output_param0: output_param0,
//                                 recipient: decoded_call.recipient,
//                             })
//                         },
//                         Err(_) => None,
//                     }
//                 })
//         })
//         .collect());
//     calls.wsteth_call_transfer_froms.append(&mut blk
//         .transactions()
//         .flat_map(|tx| {
//             tx.calls.iter()
//                 .filter(|call| call.address == WSTETH_TRACKED_CONTRACT && abi::wsteth_contract::functions::TransferFrom::match_call(call))
//                 .filter_map(|call| {
//                     match abi::wsteth_contract::functions::TransferFrom::decode(call) {
//                         Ok(decoded_call) => {
//                             let output_param0 = match abi::wsteth_contract::functions::TransferFrom::output(&call.return_data) {
//                                 Ok(output_param0) => {output_param0}
//                                 Err(_) => Default::default(),
//                             };
                            
//                             Some(contract::WstethTransferFromCall {
//                                 call_tx_hash: Hex(&tx.hash).to_string(),
//                                 call_block_time: Some(blk.timestamp().to_owned()),
//                                 call_block_number: blk.number,
//                                 call_ordinal: call.begin_ordinal,
//                                 call_success: !call.state_reverted,
//                                 amount: decoded_call.amount.to_string(),
//                                 output_param0: output_param0,
//                                 recipient: decoded_call.recipient,
//                                 sender: decoded_call.sender,
//                             })
//                         },
//                         Err(_) => None,
//                     }
//                 })
//         })
//         .collect());
//     calls.wsteth_call_unwraps.append(&mut blk
//         .transactions()
//         .flat_map(|tx| {
//             tx.calls.iter()
//                 .filter(|call| call.address == WSTETH_TRACKED_CONTRACT && abi::wsteth_contract::functions::Unwrap::match_call(call))
//                 .filter_map(|call| {
//                     match abi::wsteth_contract::functions::Unwrap::decode(call) {
//                         Ok(decoded_call) => {
//                             let output_param0 = match abi::wsteth_contract::functions::Unwrap::output(&call.return_data) {
//                                 Ok(output_param0) => {output_param0}
//                                 Err(_) => Default::default(),
//                             };
                            
//                             Some(contract::WstethUnwrapCall {
//                                 call_tx_hash: Hex(&tx.hash).to_string(),
//                                 call_block_time: Some(blk.timestamp().to_owned()),
//                                 call_block_number: blk.number,
//                                 call_ordinal: call.begin_ordinal,
//                                 call_success: !call.state_reverted,
//                                 output_param0: output_param0.to_string(),
//                                 u_wst_eth_amount: decoded_call.u_wst_eth_amount.to_string(),
//                             })
//                         },
//                         Err(_) => None,
//                     }
//                 })
//         })
//         .collect());
//     calls.wsteth_call_wraps.append(&mut blk
//         .transactions()
//         .flat_map(|tx| {
//             tx.calls.iter()
//                 .filter(|call| call.address == WSTETH_TRACKED_CONTRACT && abi::wsteth_contract::functions::Wrap::match_call(call))
//                 .filter_map(|call| {
//                     match abi::wsteth_contract::functions::Wrap::decode(call) {
//                         Ok(decoded_call) => {
//                             let output_param0 = match abi::wsteth_contract::functions::Wrap::output(&call.return_data) {
//                                 Ok(output_param0) => {output_param0}
//                                 Err(_) => Default::default(),
//                             };
                            
//                             Some(contract::WstethWrapCall {
//                                 call_tx_hash: Hex(&tx.hash).to_string(),
//                                 call_block_time: Some(blk.timestamp().to_owned()),
//                                 call_block_number: blk.number,
//                                 call_ordinal: call.begin_ordinal,
//                                 call_success: !call.state_reverted,
//                                 output_param0: output_param0.to_string(),
//                                 u_st_eth_amount: decoded_call.u_st_eth_amount.to_string(),
//                             })
//                         },
//                         Err(_) => None,
//                     }
//                 })
//         })
//         .collect());
// }

// fn db_wsteth_out(events: &contract::Events, tables: &mut DatabaseChangeTables) {
//     // Loop over all the abis events to create table changes
//     events.wsteth_approvals.iter().for_each(|evt| {
//         tables
//             .create_row("wsteth_approval", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
//             .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
//             .set("evt_block_number", evt.evt_block_number)
//             .set("owner", Hex(&evt.owner).to_string())
//             .set("spender", Hex(&evt.spender).to_string())
//             .set("value", BigDecimal::from_str(&evt.value).unwrap());
//     });
//     events.wsteth_transfers.iter().for_each(|evt| {
//         tables
//             .create_row("wsteth_transfer", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
//             .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
//             .set("evt_block_number", evt.evt_block_number)
//             .set("from", Hex(&evt.from).to_string())
//             .set("to", Hex(&evt.to).to_string())
//             .set("value", BigDecimal::from_str(&evt.value).unwrap());
//     });
// }
// fn db_wsteth_calls_out(calls: &contract::Calls, tables: &mut DatabaseChangeTables) {
//     // Loop over all the abis calls to create table changes
//     calls.wsteth_call_approves.iter().for_each(|call| {
//         tables
//             .create_row("wsteth_call_approve", [("call_tx_hash", call.call_tx_hash.to_string()),("call_ordinal", call.call_ordinal.to_string())])
//             .set("call_block_time", call.call_block_time.as_ref().unwrap())
//             .set("call_block_number", call.call_block_number)
//             .set("call_success", call.call_success)
//             .set("amount", BigDecimal::from_str(&call.amount).unwrap())
//             .set("output_param0", call.output_param0)
//             .set("spender", Hex(&call.spender).to_string());
//     });
//     calls.wsteth_call_decrease_allowances.iter().for_each(|call| {
//         tables
//             .create_row("wsteth_call_decrease_allowance", [("call_tx_hash", call.call_tx_hash.to_string()),("call_ordinal", call.call_ordinal.to_string())])
//             .set("call_block_time", call.call_block_time.as_ref().unwrap())
//             .set("call_block_number", call.call_block_number)
//             .set("call_success", call.call_success)
//             .set("output_param0", call.output_param0)
//             .set("spender", Hex(&call.spender).to_string())
//             .set("subtracted_value", BigDecimal::from_str(&call.subtracted_value).unwrap());
//     });
//     calls.wsteth_call_increase_allowances.iter().for_each(|call| {
//         tables
//             .create_row("wsteth_call_increase_allowance", [("call_tx_hash", call.call_tx_hash.to_string()),("call_ordinal", call.call_ordinal.to_string())])
//             .set("call_block_time", call.call_block_time.as_ref().unwrap())
//             .set("call_block_number", call.call_block_number)
//             .set("call_success", call.call_success)
//             .set("added_value", BigDecimal::from_str(&call.added_value).unwrap())
//             .set("output_param0", call.output_param0)
//             .set("spender", Hex(&call.spender).to_string());
//     });
//     calls.wsteth_call_permits.iter().for_each(|call| {
//         tables
//             .create_row("wsteth_call_permit", [("call_tx_hash", call.call_tx_hash.to_string()),("call_ordinal", call.call_ordinal.to_string())])
//             .set("call_block_time", call.call_block_time.as_ref().unwrap())
//             .set("call_block_number", call.call_block_number)
//             .set("call_success", call.call_success)
//             .set("deadline", BigDecimal::from_str(&call.deadline).unwrap())
//             .set("owner", Hex(&call.owner).to_string())
//             .set("r", Hex(&call.r).to_string())
//             .set("s", Hex(&call.s).to_string())
//             .set("spender", Hex(&call.spender).to_string())
//             .set("v", call.v)
//             .set("value", BigDecimal::from_str(&call.value).unwrap());
//     });
//     calls.wsteth_call_transfers.iter().for_each(|call| {
//         tables
//             .create_row("wsteth_call_transfer", [("call_tx_hash", call.call_tx_hash.to_string()),("call_ordinal", call.call_ordinal.to_string())])
//             .set("call_block_time", call.call_block_time.as_ref().unwrap())
//             .set("call_block_number", call.call_block_number)
//             .set("call_success", call.call_success)
//             .set("amount", BigDecimal::from_str(&call.amount).unwrap())
//             .set("output_param0", call.output_param0)
//             .set("recipient", Hex(&call.recipient).to_string());
//     });
//     calls.wsteth_call_transfer_froms.iter().for_each(|call| {
//         tables
//             .create_row("wsteth_call_transfer_from", [("call_tx_hash", call.call_tx_hash.to_string()),("call_ordinal", call.call_ordinal.to_string())])
//             .set("call_block_time", call.call_block_time.as_ref().unwrap())
//             .set("call_block_number", call.call_block_number)
//             .set("call_success", call.call_success)
//             .set("amount", BigDecimal::from_str(&call.amount).unwrap())
//             .set("output_param0", call.output_param0)
//             .set("recipient", Hex(&call.recipient).to_string())
//             .set("sender", Hex(&call.sender).to_string());
//     });
//     calls.wsteth_call_unwraps.iter().for_each(|call| {
//         tables
//             .create_row("wsteth_call_unwrap", [("call_tx_hash", call.call_tx_hash.to_string()),("call_ordinal", call.call_ordinal.to_string())])
//             .set("call_block_time", call.call_block_time.as_ref().unwrap())
//             .set("call_block_number", call.call_block_number)
//             .set("call_success", call.call_success)
//             .set("output_param0", BigDecimal::from_str(&call.output_param0).unwrap())
//             .set("u_wst_eth_amount", BigDecimal::from_str(&call.u_wst_eth_amount).unwrap());
//     });
//     calls.wsteth_call_wraps.iter().for_each(|call| {
//         tables
//             .create_row("wsteth_call_wrap", [("call_tx_hash", call.call_tx_hash.to_string()),("call_ordinal", call.call_ordinal.to_string())])
//             .set("call_block_time", call.call_block_time.as_ref().unwrap())
//             .set("call_block_number", call.call_block_number)
//             .set("call_success", call.call_success)
//             .set("output_param0", BigDecimal::from_str(&call.output_param0).unwrap())
//             .set("u_st_eth_amount", BigDecimal::from_str(&call.u_st_eth_amount).unwrap());
//     });
// }


// fn graph_wsteth_out(events: &contract::Events, tables: &mut EntityChangesTables) {
//     // Loop over all the abis events to create table changes
//     events.wsteth_approvals.iter().for_each(|evt| {
//         tables
//             .create_row("wsteth_approval", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
//             .set("evt_tx_hash", &evt.evt_tx_hash)
//             .set("evt_index", evt.evt_index)
//             .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
//             .set("evt_block_number", evt.evt_block_number)
//             .set("owner", Hex(&evt.owner).to_string())
//             .set("spender", Hex(&evt.spender).to_string())
//             .set("value", BigDecimal::from_str(&evt.value).unwrap());
//     });
//     events.wsteth_transfers.iter().for_each(|evt| {
//         tables
//             .create_row("wsteth_transfer", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
//             .set("evt_tx_hash", &evt.evt_tx_hash)
//             .set("evt_index", evt.evt_index)
//             .set("evt_block_time", evt.evt_block_time.as_ref().unwrap())
//             .set("evt_block_number", evt.evt_block_number)
//             .set("from", Hex(&evt.from).to_string())
//             .set("to", Hex(&evt.to).to_string())
//             .set("value", BigDecimal::from_str(&evt.value).unwrap());
//     });
// }
// fn graph_wsteth_calls_out(calls: &contract::Calls, tables: &mut EntityChangesTables) {
//     // Loop over all the abis calls to create table changes
//     calls.wsteth_call_approves.iter().for_each(|call| {
//         tables
//             .create_row("wsteth_call_approve", format!("{}-{}", call.call_tx_hash, call.call_ordinal))
//             .set("call_tx_hash", &call.call_tx_hash)
//             .set("call_ordinal", call.call_ordinal)
//             .set("call_block_time", call.call_block_time.as_ref().unwrap())
//             .set("call_block_number", call.call_block_number)
//             .set("call_success", call.call_success)
//             .set("amount", BigDecimal::from_str(&call.amount).unwrap())
//             .set("output_param0", call.output_param0)
//             .set("spender", Hex(&call.spender).to_string());
//     });
//     calls.wsteth_call_decrease_allowances.iter().for_each(|call| {
//         tables
//             .create_row("wsteth_call_decrease_allowance", format!("{}-{}", call.call_tx_hash, call.call_ordinal))
//             .set("call_tx_hash", &call.call_tx_hash)
//             .set("call_ordinal", call.call_ordinal)
//             .set("call_block_time", call.call_block_time.as_ref().unwrap())
//             .set("call_block_number", call.call_block_number)
//             .set("call_success", call.call_success)
//             .set("output_param0", call.output_param0)
//             .set("spender", Hex(&call.spender).to_string())
//             .set("subtracted_value", BigDecimal::from_str(&call.subtracted_value).unwrap());
//     });
//     calls.wsteth_call_increase_allowances.iter().for_each(|call| {
//         tables
//             .create_row("wsteth_call_increase_allowance", format!("{}-{}", call.call_tx_hash, call.call_ordinal))
//             .set("call_tx_hash", &call.call_tx_hash)
//             .set("call_ordinal", call.call_ordinal)
//             .set("call_block_time", call.call_block_time.as_ref().unwrap())
//             .set("call_block_number", call.call_block_number)
//             .set("call_success", call.call_success)
//             .set("added_value", BigDecimal::from_str(&call.added_value).unwrap())
//             .set("output_param0", call.output_param0)
//             .set("spender", Hex(&call.spender).to_string());
//     });
//     calls.wsteth_call_permits.iter().for_each(|call| {
//         tables
//             .create_row("wsteth_call_permit", format!("{}-{}", call.call_tx_hash, call.call_ordinal))
//             .set("call_tx_hash", &call.call_tx_hash)
//             .set("call_ordinal", call.call_ordinal)
//             .set("call_block_time", call.call_block_time.as_ref().unwrap())
//             .set("call_block_number", call.call_block_number)
//             .set("call_success", call.call_success)
//             .set("deadline", BigDecimal::from_str(&call.deadline).unwrap())
//             .set("owner", Hex(&call.owner).to_string())
//             .set("r", Hex(&call.r).to_string())
//             .set("s", Hex(&call.s).to_string())
//             .set("spender", Hex(&call.spender).to_string())
//             .set("v", call.v)
//             .set("value", BigDecimal::from_str(&call.value).unwrap());
//     });
//     calls.wsteth_call_transfers.iter().for_each(|call| {
//         tables
//             .create_row("wsteth_call_transfer", format!("{}-{}", call.call_tx_hash, call.call_ordinal))
//             .set("call_tx_hash", &call.call_tx_hash)
//             .set("call_ordinal", call.call_ordinal)
//             .set("call_block_time", call.call_block_time.as_ref().unwrap())
//             .set("call_block_number", call.call_block_number)
//             .set("call_success", call.call_success)
//             .set("amount", BigDecimal::from_str(&call.amount).unwrap())
//             .set("output_param0", call.output_param0)
//             .set("recipient", Hex(&call.recipient).to_string());
//     });
//     calls.wsteth_call_transfer_froms.iter().for_each(|call| {
//         tables
//             .create_row("wsteth_call_transfer_from", format!("{}-{}", call.call_tx_hash, call.call_ordinal))
//             .set("call_tx_hash", &call.call_tx_hash)
//             .set("call_ordinal", call.call_ordinal)
//             .set("call_block_time", call.call_block_time.as_ref().unwrap())
//             .set("call_block_number", call.call_block_number)
//             .set("call_success", call.call_success)
//             .set("amount", BigDecimal::from_str(&call.amount).unwrap())
//             .set("output_param0", call.output_param0)
//             .set("recipient", Hex(&call.recipient).to_string())
//             .set("sender", Hex(&call.sender).to_string());
//     });
//     calls.wsteth_call_unwraps.iter().for_each(|call| {
//         tables
//             .create_row("wsteth_call_unwrap", format!("{}-{}", call.call_tx_hash, call.call_ordinal))
//             .set("call_tx_hash", &call.call_tx_hash)
//             .set("call_ordinal", call.call_ordinal)
//             .set("call_block_time", call.call_block_time.as_ref().unwrap())
//             .set("call_block_number", call.call_block_number)
//             .set("call_success", call.call_success)
//             .set("output_param0", BigDecimal::from_str(&call.output_param0).unwrap())
//             .set("u_wst_eth_amount", BigDecimal::from_str(&call.u_wst_eth_amount).unwrap());
//     });
//     calls.wsteth_call_wraps.iter().for_each(|call| {
//         tables
//             .create_row("wsteth_call_wrap", format!("{}-{}", call.call_tx_hash, call.call_ordinal))
//             .set("call_tx_hash", &call.call_tx_hash)
//             .set("call_ordinal", call.call_ordinal)
//             .set("call_block_time", call.call_block_time.as_ref().unwrap())
//             .set("call_block_number", call.call_block_number)
//             .set("call_success", call.call_success)
//             .set("output_param0", BigDecimal::from_str(&call.output_param0).unwrap())
//             .set("u_st_eth_amount", BigDecimal::from_str(&call.u_st_eth_amount).unwrap());
//     });
//   }

// #[substreams::handlers::map]
// fn map_events(blk: eth::Block) -> Result<contract::Events, substreams::errors::Error> {
//     let mut events = contract::Events::default();
//     map_wsteth_events(&blk, &mut events);
//     Ok(events)
// }
// #[substreams::handlers::map]
// fn map_calls(blk: eth::Block) -> Result<contract::Calls, substreams::errors::Error> {
//     let mut calls = contract::Calls::default();
//     map_wsteth_calls(&blk, &mut calls);
//     Ok(calls)
// }

// #[substreams::handlers::map]
// fn db_out(events: contract::Events, calls: contract::Calls) -> Result<DatabaseChanges, substreams::errors::Error> {
//     // Initialize Database Changes container
//     let mut tables = DatabaseChangeTables::new();
//     db_wsteth_out(&events, &mut tables);
//     db_wsteth_calls_out(&calls, &mut tables);
//     Ok(tables.to_database_changes())
// }

// #[substreams::handlers::map]
// fn graph_out(events: contract::Events, calls: contract::Calls) -> Result<EntityChanges, substreams::errors::Error> {
//     // Initialize Database Changes container
//     let mut tables = EntityChangesTables::new();
//     graph_wsteth_out(&events, &mut tables);
//     graph_wsteth_calls_out(&calls, &mut tables);
//     Ok(tables.to_entity_changes())
// }
