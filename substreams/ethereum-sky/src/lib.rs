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

const SDAI_TRACKED_CONTRACT: [u8; 20] = hex!("83f20f44975d03b1b09e64809b757c47f942beea");
const DAI_USDS_CONVERTER_TRACKED_CONTRACT: [u8; 20] = hex!("3225737a9bbb6473cb4a45b7244aca2befdb276a");
const DAI_LITE_PSM_TRACKED_CONTRACT: [u8; 20] = hex!("f6e72db5454dd049d0788e411b06cfaf16853042");
const USDS_PSM_WRAPPER_TRACKED_CONTRACT: [u8; 20] = hex!("a188eec8f81263234da3622a406892f3d630f98c");
const SUSDS_TRACKED_CONTRACT: [u8; 20] = hex!("a3931d71877c0e7a3148cb7eb4463524fec27fbd");
const MKR_SKY_CONVERTER_TRACKED_CONTRACT: [u8; 20] = hex!("bdcfca946b6cdd965f99a839e4435bcdc1bc470b");
const USDC_TRACKED_CONTRACT: [u8; 20] = hex!("a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48");

fn map_sdai_events(blk: &eth::Block, events: &mut contract::Events) {
    events.sdai_approvals.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == SDAI_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::sdai_contract::events::Approval::match_and_decode(log) {
                        return Some(contract::SdaiApproval {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            owner: event.owner,
                            spender: event.spender,
                            value: event.value.to_string(),
                        });
                    }

                    None
                })
        })
        .collect());
    events.sdai_deposits.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == SDAI_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::sdai_contract::events::Deposit::match_and_decode(log) {
                        return Some(contract::SdaiDeposit {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            assets: event.assets.to_string(),
                            owner: event.owner,
                            sender: event.sender,
                            shares: event.shares.to_string(),
                        });
                    }

                    None
                })
        })
        .collect());
    events.sdai_transfers.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == SDAI_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::sdai_contract::events::Transfer::match_and_decode(log) {
                        return Some(contract::SdaiTransfer {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            from: event.from,
                            to: event.to,
                            value: event.value.to_string(),
                        });
                    }

                    None
                })
        })
        .collect());
    events.sdai_withdraws.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == SDAI_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::sdai_contract::events::Withdraw::match_and_decode(log) {
                        return Some(contract::SdaiWithdraw {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            assets: event.assets.to_string(),
                            owner: event.owner,
                            receiver: event.receiver,
                            sender: event.sender,
                            shares: event.shares.to_string(),
                        });
                    }

                    None
                })
        })
        .collect());
}
fn map_sdai_calls(blk: &eth::Block, calls: &mut contract::Calls) {
    calls.sdai_call_approves.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == SDAI_TRACKED_CONTRACT && abi::sdai_contract::functions::Approve::match_call(call))
                .filter_map(|call| {
                    match abi::sdai_contract::functions::Approve::decode(call) {
                        Ok(decoded_call) => {
                            let output_param0 = match abi::sdai_contract::functions::Approve::output(&call.return_data) {
                                Ok(output_param0) => {output_param0}
                                Err(_) => Default::default(),
                            };
                            
                            Some(contract::SdaiApproveCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
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
    calls.sdai_call_decrease_allowances.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == SDAI_TRACKED_CONTRACT && abi::sdai_contract::functions::DecreaseAllowance::match_call(call))
                .filter_map(|call| {
                    match abi::sdai_contract::functions::DecreaseAllowance::decode(call) {
                        Ok(decoded_call) => {
                            let output_param0 = match abi::sdai_contract::functions::DecreaseAllowance::output(&call.return_data) {
                                Ok(output_param0) => {output_param0}
                                Err(_) => Default::default(),
                            };
                            
                            Some(contract::SdaiDecreaseAllowanceCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                output_param0: output_param0,
                                spender: decoded_call.spender,
                                subtracted_value: decoded_call.subtracted_value.to_string(),
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.sdai_call_deposits.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == SDAI_TRACKED_CONTRACT && abi::sdai_contract::functions::Deposit::match_call(call))
                .filter_map(|call| {
                    match abi::sdai_contract::functions::Deposit::decode(call) {
                        Ok(decoded_call) => {
                            let output_shares = match abi::sdai_contract::functions::Deposit::output(&call.return_data) {
                                Ok(output_shares) => {output_shares}
                                Err(_) => Default::default(),
                            };
                            
                            Some(contract::SdaiDepositCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                assets: decoded_call.assets.to_string(),
                                output_shares: output_shares.to_string(),
                                receiver: decoded_call.receiver,
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.sdai_call_increase_allowances.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == SDAI_TRACKED_CONTRACT && abi::sdai_contract::functions::IncreaseAllowance::match_call(call))
                .filter_map(|call| {
                    match abi::sdai_contract::functions::IncreaseAllowance::decode(call) {
                        Ok(decoded_call) => {
                            let output_param0 = match abi::sdai_contract::functions::IncreaseAllowance::output(&call.return_data) {
                                Ok(output_param0) => {output_param0}
                                Err(_) => Default::default(),
                            };
                            
                            Some(contract::SdaiIncreaseAllowanceCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                added_value: decoded_call.added_value.to_string(),
                                output_param0: output_param0,
                                spender: decoded_call.spender,
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.sdai_call_mints.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == SDAI_TRACKED_CONTRACT && abi::sdai_contract::functions::Mint::match_call(call))
                .filter_map(|call| {
                    match abi::sdai_contract::functions::Mint::decode(call) {
                        Ok(decoded_call) => {
                            let output_assets = match abi::sdai_contract::functions::Mint::output(&call.return_data) {
                                Ok(output_assets) => {output_assets}
                                Err(_) => Default::default(),
                            };
                            
                            Some(contract::SdaiMintCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                output_assets: output_assets.to_string(),
                                receiver: decoded_call.receiver,
                                shares: decoded_call.shares.to_string(),
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.sdai_call_permit_1s.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == SDAI_TRACKED_CONTRACT && abi::sdai_contract::functions::Permit1::match_call(call))
                .filter_map(|call| {
                    match abi::sdai_contract::functions::Permit1::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::SdaiPermit1call {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                deadline: decoded_call.deadline.to_string(),
                                owner: decoded_call.owner,
                                signature: decoded_call.signature,
                                spender: decoded_call.spender,
                                value: decoded_call.value.to_string(),
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.sdai_call_permit_2s.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == SDAI_TRACKED_CONTRACT && abi::sdai_contract::functions::Permit2::match_call(call))
                .filter_map(|call| {
                    match abi::sdai_contract::functions::Permit2::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::SdaiPermit2call {
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
    calls.sdai_call_redeems.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == SDAI_TRACKED_CONTRACT && abi::sdai_contract::functions::Redeem::match_call(call))
                .filter_map(|call| {
                    match abi::sdai_contract::functions::Redeem::decode(call) {
                        Ok(decoded_call) => {
                            let output_assets = match abi::sdai_contract::functions::Redeem::output(&call.return_data) {
                                Ok(output_assets) => {output_assets}
                                Err(_) => Default::default(),
                            };
                            
                            Some(contract::SdaiRedeemCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                output_assets: output_assets.to_string(),
                                owner: decoded_call.owner,
                                receiver: decoded_call.receiver,
                                shares: decoded_call.shares.to_string(),
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.sdai_call_transfers.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == SDAI_TRACKED_CONTRACT && abi::sdai_contract::functions::Transfer::match_call(call))
                .filter_map(|call| {
                    match abi::sdai_contract::functions::Transfer::decode(call) {
                        Ok(decoded_call) => {
                            let output_param0 = match abi::sdai_contract::functions::Transfer::output(&call.return_data) {
                                Ok(output_param0) => {output_param0}
                                Err(_) => Default::default(),
                            };
                            
                            Some(contract::SdaiTransferCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
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
    calls.sdai_call_transfer_froms.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == SDAI_TRACKED_CONTRACT && abi::sdai_contract::functions::TransferFrom::match_call(call))
                .filter_map(|call| {
                    match abi::sdai_contract::functions::TransferFrom::decode(call) {
                        Ok(decoded_call) => {
                            let output_param0 = match abi::sdai_contract::functions::TransferFrom::output(&call.return_data) {
                                Ok(output_param0) => {output_param0}
                                Err(_) => Default::default(),
                            };
                            
                            Some(contract::SdaiTransferFromCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
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
    calls.sdai_call_withdraws.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == SDAI_TRACKED_CONTRACT && abi::sdai_contract::functions::Withdraw::match_call(call))
                .filter_map(|call| {
                    match abi::sdai_contract::functions::Withdraw::decode(call) {
                        Ok(decoded_call) => {
                            let output_shares = match abi::sdai_contract::functions::Withdraw::output(&call.return_data) {
                                Ok(output_shares) => {output_shares}
                                Err(_) => Default::default(),
                            };
                            
                            Some(contract::SdaiWithdrawCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                assets: decoded_call.assets.to_string(),
                                output_shares: output_shares.to_string(),
                                owner: decoded_call.owner,
                                receiver: decoded_call.receiver,
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
}

fn map_dai_usds_converter_events(blk: &eth::Block, events: &mut contract::Events) {
    events.dai_usds_converter_dai_to_usds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == DAI_USDS_CONVERTER_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::dai_usds_converter_contract::events::DaiToUsds::match_and_decode(log) {
                        return Some(contract::DaiUsdsConverterDaiToUsds {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            caller: event.caller,
                            usr: event.usr,
                            wad: event.wad.to_string(),
                        });
                    }

                    None
                })
        })
        .collect());
    events.dai_usds_converter_usds_to_dais.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == DAI_USDS_CONVERTER_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::dai_usds_converter_contract::events::UsdsToDai::match_and_decode(log) {
                        return Some(contract::DaiUsdsConverterUsdsToDai {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            caller: event.caller,
                            usr: event.usr,
                            wad: event.wad.to_string(),
                        });
                    }

                    None
                })
        })
        .collect());
}
fn map_dai_usds_converter_calls(blk: &eth::Block, calls: &mut contract::Calls) {
    calls.dai_usds_converter_call_dai_to_usds.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == DAI_USDS_CONVERTER_TRACKED_CONTRACT && abi::dai_usds_converter_contract::functions::DaiToUsds::match_call(call))
                .filter_map(|call| {
                    match abi::dai_usds_converter_contract::functions::DaiToUsds::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::DaiUsdsConverterDaiToUsdsCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                usr: decoded_call.usr,
                                wad: decoded_call.wad.to_string(),
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.dai_usds_converter_call_usds_to_dais.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == DAI_USDS_CONVERTER_TRACKED_CONTRACT && abi::dai_usds_converter_contract::functions::UsdsToDai::match_call(call))
                .filter_map(|call| {
                    match abi::dai_usds_converter_contract::functions::UsdsToDai::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::DaiUsdsConverterUsdsToDaiCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                usr: decoded_call.usr,
                                wad: decoded_call.wad.to_string(),
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
}

fn map_dai_lite_psm_events(blk: &eth::Block, events: &mut contract::Events) {
    events.dai_lite_psm_buy_gems.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == DAI_LITE_PSM_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::dai_lite_psm_contract::events::BuyGem::match_and_decode(log) {
                        return Some(contract::DaiLitePsmBuyGem {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            fee: event.fee.to_string(),
                            owner: event.owner,
                            value: event.value.to_string(),
                        });
                    }

                    None
                })
        })
        .collect());
    events.dai_lite_psm_chugs.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == DAI_LITE_PSM_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::dai_lite_psm_contract::events::Chug::match_and_decode(log) {
                        return Some(contract::DaiLitePsmChug {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            wad: event.wad.to_string(),
                        });
                    }

                    None
                })
        })
        .collect());
    events.dai_lite_psm_denies.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == DAI_LITE_PSM_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::dai_lite_psm_contract::events::Deny::match_and_decode(log) {
                        return Some(contract::DaiLitePsmDeny {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            usr: event.usr,
                        });
                    }

                    None
                })
        })
        .collect());
    events.dai_lite_psm_disses.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == DAI_LITE_PSM_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::dai_lite_psm_contract::events::Diss::match_and_decode(log) {
                        return Some(contract::DaiLitePsmDiss {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            usr: event.usr,
                        });
                    }

                    None
                })
        })
        .collect());
    events.dai_lite_psm_file_1s.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == DAI_LITE_PSM_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::dai_lite_psm_contract::events::File1::match_and_decode(log) {
                        return Some(contract::DaiLitePsmFile1 {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            data: event.data,
                            what: Vec::from(event.what),
                        });
                    }

                    None
                })
        })
        .collect());
    events.dai_lite_psm_file_2s.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == DAI_LITE_PSM_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::dai_lite_psm_contract::events::File2::match_and_decode(log) {
                        return Some(contract::DaiLitePsmFile2 {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            data: event.data.to_string(),
                            what: Vec::from(event.what),
                        });
                    }

                    None
                })
        })
        .collect());
    events.dai_lite_psm_fills.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == DAI_LITE_PSM_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::dai_lite_psm_contract::events::Fill::match_and_decode(log) {
                        return Some(contract::DaiLitePsmFill {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            wad: event.wad.to_string(),
                        });
                    }

                    None
                })
        })
        .collect());
    events.dai_lite_psm_kisses.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == DAI_LITE_PSM_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::dai_lite_psm_contract::events::Kiss::match_and_decode(log) {
                        return Some(contract::DaiLitePsmKiss {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            usr: event.usr,
                        });
                    }

                    None
                })
        })
        .collect());
    events.dai_lite_psm_relies.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == DAI_LITE_PSM_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::dai_lite_psm_contract::events::Rely::match_and_decode(log) {
                        return Some(contract::DaiLitePsmRely {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            usr: event.usr,
                        });
                    }

                    None
                })
        })
        .collect());
    events.dai_lite_psm_sell_gems.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == DAI_LITE_PSM_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::dai_lite_psm_contract::events::SellGem::match_and_decode(log) {
                        return Some(contract::DaiLitePsmSellGem {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            fee: event.fee.to_string(),
                            owner: event.owner,
                            value: event.value.to_string(),
                        });
                    }

                    None
                })
        })
        .collect());
    events.dai_lite_psm_trims.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == DAI_LITE_PSM_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::dai_lite_psm_contract::events::Trim::match_and_decode(log) {
                        return Some(contract::DaiLitePsmTrim {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            wad: event.wad.to_string(),
                        });
                    }

                    None
                })
        })
        .collect());
}
fn map_dai_lite_psm_calls(blk: &eth::Block, calls: &mut contract::Calls) {
    calls.dai_lite_psm_call_buy_gems.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == DAI_LITE_PSM_TRACKED_CONTRACT && abi::dai_lite_psm_contract::functions::BuyGem::match_call(call))
                .filter_map(|call| {
                    match abi::dai_lite_psm_contract::functions::BuyGem::decode(call) {
                        Ok(decoded_call) => {
                            let output_dai_in_wad = match abi::dai_lite_psm_contract::functions::BuyGem::output(&call.return_data) {
                                Ok(output_dai_in_wad) => {output_dai_in_wad}
                                Err(_) => Default::default(),
                            };
                            
                            Some(contract::DaiLitePsmBuyGemCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                gem_amt: decoded_call.gem_amt.to_string(),
                                output_dai_in_wad: output_dai_in_wad.to_string(),
                                usr: decoded_call.usr,
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.dai_lite_psm_call_buy_gem_no_fees.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == DAI_LITE_PSM_TRACKED_CONTRACT && abi::dai_lite_psm_contract::functions::BuyGemNoFee::match_call(call))
                .filter_map(|call| {
                    match abi::dai_lite_psm_contract::functions::BuyGemNoFee::decode(call) {
                        Ok(decoded_call) => {
                            let output_dai_in_wad = match abi::dai_lite_psm_contract::functions::BuyGemNoFee::output(&call.return_data) {
                                Ok(output_dai_in_wad) => {output_dai_in_wad}
                                Err(_) => Default::default(),
                            };
                            
                            Some(contract::DaiLitePsmBuyGemNoFeeCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                gem_amt: decoded_call.gem_amt.to_string(),
                                output_dai_in_wad: output_dai_in_wad.to_string(),
                                usr: decoded_call.usr,
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.dai_lite_psm_call_chugs.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == DAI_LITE_PSM_TRACKED_CONTRACT && abi::dai_lite_psm_contract::functions::Chug::match_call(call))
                .filter_map(|call| {
                    match abi::dai_lite_psm_contract::functions::Chug::decode(call) {
                        Ok(decoded_call) => {
                            let output_wad = match abi::dai_lite_psm_contract::functions::Chug::output(&call.return_data) {
                                Ok(output_wad) => {output_wad}
                                Err(_) => Default::default(),
                            };
                            
                            Some(contract::DaiLitePsmChugCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                output_wad: output_wad.to_string(),
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.dai_lite_psm_call_denies.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == DAI_LITE_PSM_TRACKED_CONTRACT && abi::dai_lite_psm_contract::functions::Deny::match_call(call))
                .filter_map(|call| {
                    match abi::dai_lite_psm_contract::functions::Deny::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::DaiLitePsmDenyCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                usr: decoded_call.usr,
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.dai_lite_psm_call_disses.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == DAI_LITE_PSM_TRACKED_CONTRACT && abi::dai_lite_psm_contract::functions::Diss::match_call(call))
                .filter_map(|call| {
                    match abi::dai_lite_psm_contract::functions::Diss::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::DaiLitePsmDissCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                usr: decoded_call.usr,
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.dai_lite_psm_call_file_1s.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == DAI_LITE_PSM_TRACKED_CONTRACT && abi::dai_lite_psm_contract::functions::File1::match_call(call))
                .filter_map(|call| {
                    match abi::dai_lite_psm_contract::functions::File1::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::DaiLitePsmFile1call {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                data: decoded_call.data.to_string(),
                                what: Vec::from(decoded_call.what),
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.dai_lite_psm_call_file_2s.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == DAI_LITE_PSM_TRACKED_CONTRACT && abi::dai_lite_psm_contract::functions::File2::match_call(call))
                .filter_map(|call| {
                    match abi::dai_lite_psm_contract::functions::File2::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::DaiLitePsmFile2call {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                data: decoded_call.data,
                                what: Vec::from(decoded_call.what),
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.dai_lite_psm_call_fills.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == DAI_LITE_PSM_TRACKED_CONTRACT && abi::dai_lite_psm_contract::functions::Fill::match_call(call))
                .filter_map(|call| {
                    match abi::dai_lite_psm_contract::functions::Fill::decode(call) {
                        Ok(decoded_call) => {
                            let output_wad = match abi::dai_lite_psm_contract::functions::Fill::output(&call.return_data) {
                                Ok(output_wad) => {output_wad}
                                Err(_) => Default::default(),
                            };
                            
                            Some(contract::DaiLitePsmFillCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                output_wad: output_wad.to_string(),
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.dai_lite_psm_call_kisses.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == DAI_LITE_PSM_TRACKED_CONTRACT && abi::dai_lite_psm_contract::functions::Kiss::match_call(call))
                .filter_map(|call| {
                    match abi::dai_lite_psm_contract::functions::Kiss::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::DaiLitePsmKissCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                usr: decoded_call.usr,
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.dai_lite_psm_call_relies.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == DAI_LITE_PSM_TRACKED_CONTRACT && abi::dai_lite_psm_contract::functions::Rely::match_call(call))
                .filter_map(|call| {
                    match abi::dai_lite_psm_contract::functions::Rely::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::DaiLitePsmRelyCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                usr: decoded_call.usr,
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.dai_lite_psm_call_sell_gems.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == DAI_LITE_PSM_TRACKED_CONTRACT && abi::dai_lite_psm_contract::functions::SellGem::match_call(call))
                .filter_map(|call| {
                    match abi::dai_lite_psm_contract::functions::SellGem::decode(call) {
                        Ok(decoded_call) => {
                            let output_dai_out_wad = match abi::dai_lite_psm_contract::functions::SellGem::output(&call.return_data) {
                                Ok(output_dai_out_wad) => {output_dai_out_wad}
                                Err(_) => Default::default(),
                            };
                            
                            Some(contract::DaiLitePsmSellGemCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                gem_amt: decoded_call.gem_amt.to_string(),
                                output_dai_out_wad: output_dai_out_wad.to_string(),
                                usr: decoded_call.usr,
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.dai_lite_psm_call_sell_gem_no_fees.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == DAI_LITE_PSM_TRACKED_CONTRACT && abi::dai_lite_psm_contract::functions::SellGemNoFee::match_call(call))
                .filter_map(|call| {
                    match abi::dai_lite_psm_contract::functions::SellGemNoFee::decode(call) {
                        Ok(decoded_call) => {
                            let output_dai_out_wad = match abi::dai_lite_psm_contract::functions::SellGemNoFee::output(&call.return_data) {
                                Ok(output_dai_out_wad) => {output_dai_out_wad}
                                Err(_) => Default::default(),
                            };
                            
                            Some(contract::DaiLitePsmSellGemNoFeeCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                gem_amt: decoded_call.gem_amt.to_string(),
                                output_dai_out_wad: output_dai_out_wad.to_string(),
                                usr: decoded_call.usr,
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.dai_lite_psm_call_trims.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == DAI_LITE_PSM_TRACKED_CONTRACT && abi::dai_lite_psm_contract::functions::Trim::match_call(call))
                .filter_map(|call| {
                    match abi::dai_lite_psm_contract::functions::Trim::decode(call) {
                        Ok(decoded_call) => {
                            let output_wad = match abi::dai_lite_psm_contract::functions::Trim::output(&call.return_data) {
                                Ok(output_wad) => {output_wad}
                                Err(_) => Default::default(),
                            };
                            
                            Some(contract::DaiLitePsmTrimCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                output_wad: output_wad.to_string(),
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
}

fn map_usds_psm_wrapper_events(blk: &eth::Block, events: &mut contract::Events) {
}
fn map_usds_psm_wrapper_calls(blk: &eth::Block, calls: &mut contract::Calls) {
    calls.usds_psm_wrapper_call_buy_gems.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == USDS_PSM_WRAPPER_TRACKED_CONTRACT && abi::usds_psm_wrapper_contract::functions::BuyGem::match_call(call))
                .filter_map(|call| {
                    match abi::usds_psm_wrapper_contract::functions::BuyGem::decode(call) {
                        Ok(decoded_call) => {
                            let output_usds_in_wad = match abi::usds_psm_wrapper_contract::functions::BuyGem::output(&call.return_data) {
                                Ok(output_usds_in_wad) => {output_usds_in_wad}
                                Err(_) => Default::default(),
                            };
                            
                            Some(contract::UsdsPsmWrapperBuyGemCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                gem_amt: decoded_call.gem_amt.to_string(),
                                output_usds_in_wad: output_usds_in_wad.to_string(),
                                usr: decoded_call.usr,
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.usds_psm_wrapper_call_sell_gems.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == USDS_PSM_WRAPPER_TRACKED_CONTRACT && abi::usds_psm_wrapper_contract::functions::SellGem::match_call(call))
                .filter_map(|call| {
                    match abi::usds_psm_wrapper_contract::functions::SellGem::decode(call) {
                        Ok(decoded_call) => {
                            let output_usds_out_wad = match abi::usds_psm_wrapper_contract::functions::SellGem::output(&call.return_data) {
                                Ok(output_usds_out_wad) => {output_usds_out_wad}
                                Err(_) => Default::default(),
                            };
                            
                            Some(contract::UsdsPsmWrapperSellGemCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                gem_amt: decoded_call.gem_amt.to_string(),
                                output_usds_out_wad: output_usds_out_wad.to_string(),
                                usr: decoded_call.usr,
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
}

fn map_susds_events(blk: &eth::Block, events: &mut contract::Events) {
    events.susds_approvals.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == SUSDS_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::susds_contract::events::Approval::match_and_decode(log) {
                        return Some(contract::SusdsApproval {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            owner: event.owner,
                            spender: event.spender,
                            value: event.value.to_string(),
                        });
                    }

                    None
                })
        })
        .collect());
    events.susds_denies.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == SUSDS_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::susds_contract::events::Deny::match_and_decode(log) {
                        return Some(contract::SusdsDeny {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            usr: event.usr,
                        });
                    }

                    None
                })
        })
        .collect());
    events.susds_deposits.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == SUSDS_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::susds_contract::events::Deposit::match_and_decode(log) {
                        return Some(contract::SusdsDeposit {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            assets: event.assets.to_string(),
                            owner: event.owner,
                            sender: event.sender,
                            shares: event.shares.to_string(),
                        });
                    }

                    None
                })
        })
        .collect());
    events.susds_drips.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == SUSDS_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::susds_contract::events::Drip::match_and_decode(log) {
                        return Some(contract::SusdsDrip {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            chi: event.chi.to_string(),
                            diff: event.diff.to_string(),
                        });
                    }

                    None
                })
        })
        .collect());
    events.susds_files.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == SUSDS_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::susds_contract::events::File::match_and_decode(log) {
                        return Some(contract::SusdsFile {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            data: event.data.to_string(),
                            what: Vec::from(event.what),
                        });
                    }

                    None
                })
        })
        .collect());
    events.susds_initializeds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == SUSDS_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::susds_contract::events::Initialized::match_and_decode(log) {
                        return Some(contract::SusdsInitialized {
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
    events.susds_referrals.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == SUSDS_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::susds_contract::events::Referral::match_and_decode(log) {
                        return Some(contract::SusdsReferral {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            assets: event.assets.to_string(),
                            owner: event.owner,
                            referral: event.referral.to_u64(),
                            shares: event.shares.to_string(),
                        });
                    }

                    None
                })
        })
        .collect());
    events.susds_relies.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == SUSDS_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::susds_contract::events::Rely::match_and_decode(log) {
                        return Some(contract::SusdsRely {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            usr: event.usr,
                        });
                    }

                    None
                })
        })
        .collect());
    events.susds_transfers.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == SUSDS_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::susds_contract::events::Transfer::match_and_decode(log) {
                        return Some(contract::SusdsTransfer {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            from: event.from,
                            to: event.to,
                            value: event.value.to_string(),
                        });
                    }

                    None
                })
        })
        .collect());
    events.susds_upgraded_1s.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == SUSDS_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::susds_contract::events::Upgraded1::match_and_decode(log) {
                        return Some(contract::SusdsUpgraded1 {
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
    events.susds_upgraded_2s.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == SUSDS_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::susds_contract::events::Upgraded2::match_and_decode(log) {
                        return Some(contract::SusdsUpgraded2 {
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
    events.susds_withdraws.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == SUSDS_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::susds_contract::events::Withdraw::match_and_decode(log) {
                        return Some(contract::SusdsWithdraw {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            assets: event.assets.to_string(),
                            owner: event.owner,
                            receiver: event.receiver,
                            sender: event.sender,
                            shares: event.shares.to_string(),
                        });
                    }

                    None
                })
        })
        .collect());
}
fn map_susds_calls(blk: &eth::Block, calls: &mut contract::Calls) {
    calls.susds_call_approves.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == SUSDS_TRACKED_CONTRACT && abi::susds_contract::functions::Approve::match_call(call))
                .filter_map(|call| {
                    match abi::susds_contract::functions::Approve::decode(call) {
                        Ok(decoded_call) => {
                            let output_param0 = match abi::susds_contract::functions::Approve::output(&call.return_data) {
                                Ok(output_param0) => {output_param0}
                                Err(_) => Default::default(),
                            };
                            
                            Some(contract::SusdsApproveCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
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
    calls.susds_call_denies.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == SUSDS_TRACKED_CONTRACT && abi::susds_contract::functions::Deny::match_call(call))
                .filter_map(|call| {
                    match abi::susds_contract::functions::Deny::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::SusdsDenyCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                usr: decoded_call.usr,
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.susds_call_deposit_1s.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == SUSDS_TRACKED_CONTRACT && abi::susds_contract::functions::Deposit1::match_call(call))
                .filter_map(|call| {
                    match abi::susds_contract::functions::Deposit1::decode(call) {
                        Ok(decoded_call) => {
                            let output_shares = match abi::susds_contract::functions::Deposit1::output(&call.return_data) {
                                Ok(output_shares) => {output_shares}
                                Err(_) => Default::default(),
                            };
                            
                            Some(contract::SusdsDeposit1call {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                assets: decoded_call.assets.to_string(),
                                output_shares: output_shares.to_string(),
                                receiver: decoded_call.receiver,
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.susds_call_deposit_2s.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == SUSDS_TRACKED_CONTRACT && abi::susds_contract::functions::Deposit2::match_call(call))
                .filter_map(|call| {
                    match abi::susds_contract::functions::Deposit2::decode(call) {
                        Ok(decoded_call) => {
                            let output_shares = match abi::susds_contract::functions::Deposit2::output(&call.return_data) {
                                Ok(output_shares) => {output_shares}
                                Err(_) => Default::default(),
                            };
                            
                            Some(contract::SusdsDeposit2call {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                assets: decoded_call.assets.to_string(),
                                output_shares: output_shares.to_string(),
                                receiver: decoded_call.receiver,
                                referral: decoded_call.referral.to_u64(),
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.susds_call_drips.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == SUSDS_TRACKED_CONTRACT && abi::susds_contract::functions::Drip::match_call(call))
                .filter_map(|call| {
                    match abi::susds_contract::functions::Drip::decode(call) {
                        Ok(decoded_call) => {
                            let output_n_chi = match abi::susds_contract::functions::Drip::output(&call.return_data) {
                                Ok(output_n_chi) => {output_n_chi}
                                Err(_) => Default::default(),
                            };
                            
                            Some(contract::SusdsDripCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                output_n_chi: output_n_chi.to_string(),
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.susds_call_files.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == SUSDS_TRACKED_CONTRACT && abi::susds_contract::functions::File::match_call(call))
                .filter_map(|call| {
                    match abi::susds_contract::functions::File::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::SusdsFileCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                data: decoded_call.data.to_string(),
                                what: Vec::from(decoded_call.what),
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.susds_call_initializes.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == SUSDS_TRACKED_CONTRACT && abi::susds_contract::functions::Initialize::match_call(call))
                .filter_map(|call| {
                    match abi::susds_contract::functions::Initialize::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::SusdsInitializeCall {
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
    calls.susds_call_mint_1s.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == SUSDS_TRACKED_CONTRACT && abi::susds_contract::functions::Mint1::match_call(call))
                .filter_map(|call| {
                    match abi::susds_contract::functions::Mint1::decode(call) {
                        Ok(decoded_call) => {
                            let output_assets = match abi::susds_contract::functions::Mint1::output(&call.return_data) {
                                Ok(output_assets) => {output_assets}
                                Err(_) => Default::default(),
                            };
                            
                            Some(contract::SusdsMint1call {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                output_assets: output_assets.to_string(),
                                receiver: decoded_call.receiver,
                                referral: decoded_call.referral.to_u64(),
                                shares: decoded_call.shares.to_string(),
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.susds_call_mint_2s.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == SUSDS_TRACKED_CONTRACT && abi::susds_contract::functions::Mint2::match_call(call))
                .filter_map(|call| {
                    match abi::susds_contract::functions::Mint2::decode(call) {
                        Ok(decoded_call) => {
                            let output_assets = match abi::susds_contract::functions::Mint2::output(&call.return_data) {
                                Ok(output_assets) => {output_assets}
                                Err(_) => Default::default(),
                            };
                            
                            Some(contract::SusdsMint2call {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                output_assets: output_assets.to_string(),
                                receiver: decoded_call.receiver,
                                shares: decoded_call.shares.to_string(),
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.susds_call_permit_1s.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == SUSDS_TRACKED_CONTRACT && abi::susds_contract::functions::Permit1::match_call(call))
                .filter_map(|call| {
                    match abi::susds_contract::functions::Permit1::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::SusdsPermit1call {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                deadline: decoded_call.deadline.to_string(),
                                owner: decoded_call.owner,
                                signature: decoded_call.signature,
                                spender: decoded_call.spender,
                                value: decoded_call.value.to_string(),
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.susds_call_permit_2s.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == SUSDS_TRACKED_CONTRACT && abi::susds_contract::functions::Permit2::match_call(call))
                .filter_map(|call| {
                    match abi::susds_contract::functions::Permit2::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::SusdsPermit2call {
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
    calls.susds_call_redeems.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == SUSDS_TRACKED_CONTRACT && abi::susds_contract::functions::Redeem::match_call(call))
                .filter_map(|call| {
                    match abi::susds_contract::functions::Redeem::decode(call) {
                        Ok(decoded_call) => {
                            let output_assets = match abi::susds_contract::functions::Redeem::output(&call.return_data) {
                                Ok(output_assets) => {output_assets}
                                Err(_) => Default::default(),
                            };
                            
                            Some(contract::SusdsRedeemCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                output_assets: output_assets.to_string(),
                                owner: decoded_call.owner,
                                receiver: decoded_call.receiver,
                                shares: decoded_call.shares.to_string(),
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.susds_call_relies.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == SUSDS_TRACKED_CONTRACT && abi::susds_contract::functions::Rely::match_call(call))
                .filter_map(|call| {
                    match abi::susds_contract::functions::Rely::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::SusdsRelyCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                usr: decoded_call.usr,
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.susds_call_transfers.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == SUSDS_TRACKED_CONTRACT && abi::susds_contract::functions::Transfer::match_call(call))
                .filter_map(|call| {
                    match abi::susds_contract::functions::Transfer::decode(call) {
                        Ok(decoded_call) => {
                            let output_param0 = match abi::susds_contract::functions::Transfer::output(&call.return_data) {
                                Ok(output_param0) => {output_param0}
                                Err(_) => Default::default(),
                            };
                            
                            Some(contract::SusdsTransferCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
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
    calls.susds_call_transfer_froms.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == SUSDS_TRACKED_CONTRACT && abi::susds_contract::functions::TransferFrom::match_call(call))
                .filter_map(|call| {
                    match abi::susds_contract::functions::TransferFrom::decode(call) {
                        Ok(decoded_call) => {
                            let output_param0 = match abi::susds_contract::functions::TransferFrom::output(&call.return_data) {
                                Ok(output_param0) => {output_param0}
                                Err(_) => Default::default(),
                            };
                            
                            Some(contract::SusdsTransferFromCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
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
    calls.susds_call_upgrade_to_and_calls.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == SUSDS_TRACKED_CONTRACT && abi::susds_contract::functions::UpgradeToAndCall::match_call(call))
                .filter_map(|call| {
                    match abi::susds_contract::functions::UpgradeToAndCall::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::SusdsUpgradeToAndCallCall {
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
    calls.susds_call_withdraws.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == SUSDS_TRACKED_CONTRACT && abi::susds_contract::functions::Withdraw::match_call(call))
                .filter_map(|call| {
                    match abi::susds_contract::functions::Withdraw::decode(call) {
                        Ok(decoded_call) => {
                            let output_shares = match abi::susds_contract::functions::Withdraw::output(&call.return_data) {
                                Ok(output_shares) => {output_shares}
                                Err(_) => Default::default(),
                            };
                            
                            Some(contract::SusdsWithdrawCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                assets: decoded_call.assets.to_string(),
                                output_shares: output_shares.to_string(),
                                owner: decoded_call.owner,
                                receiver: decoded_call.receiver,
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
}

fn map_mkr_sky_converter_events(blk: &eth::Block, events: &mut contract::Events) {
    events.mkr_sky_converter_mkr_to_skies.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == MKR_SKY_CONVERTER_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::mkr_sky_converter_contract::events::MkrToSky::match_and_decode(log) {
                        return Some(contract::MkrSkyConverterMkrToSky {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            caller: event.caller,
                            mkr_amt: event.mkr_amt.to_string(),
                            sky_amt: event.sky_amt.to_string(),
                            usr: event.usr,
                        });
                    }

                    None
                })
        })
        .collect());
    events.mkr_sky_converter_sky_to_mkrs.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == MKR_SKY_CONVERTER_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::mkr_sky_converter_contract::events::SkyToMkr::match_and_decode(log) {
                        return Some(contract::MkrSkyConverterSkyToMkr {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            caller: event.caller,
                            mkr_amt: event.mkr_amt.to_string(),
                            sky_amt: event.sky_amt.to_string(),
                            usr: event.usr,
                        });
                    }

                    None
                })
        })
        .collect());
}
fn map_mkr_sky_converter_calls(blk: &eth::Block, calls: &mut contract::Calls) {
    calls.mkr_sky_converter_call_mkr_to_skies.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == MKR_SKY_CONVERTER_TRACKED_CONTRACT && abi::mkr_sky_converter_contract::functions::MkrToSky::match_call(call))
                .filter_map(|call| {
                    match abi::mkr_sky_converter_contract::functions::MkrToSky::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::MkrSkyConverterMkrToSkyCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                mkr_amt: decoded_call.mkr_amt.to_string(),
                                usr: decoded_call.usr,
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.mkr_sky_converter_call_sky_to_mkrs.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == MKR_SKY_CONVERTER_TRACKED_CONTRACT && abi::mkr_sky_converter_contract::functions::SkyToMkr::match_call(call))
                .filter_map(|call| {
                    match abi::mkr_sky_converter_contract::functions::SkyToMkr::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::MkrSkyConverterSkyToMkrCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                sky_amt: decoded_call.sky_amt.to_string(),
                                usr: decoded_call.usr,
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
}

fn map_usdc_events(blk: &eth::Block, events: &mut contract::Events) {
    events.usdc_admin_changeds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == USDC_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::usdc_contract::events::AdminChanged::match_and_decode(log) {
                        return Some(contract::UsdcAdminChanged {
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
    events.usdc_approvals.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == USDC_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::usdc_contract::events::Approval::match_and_decode(log) {
                        return Some(contract::UsdcApproval {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            owner: event.owner,
                            spender: event.spender,
                            value: event.value.to_string(),
                        });
                    }

                    None
                })
        })
        .collect());
    events.usdc_authorization_canceleds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == USDC_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::usdc_contract::events::AuthorizationCanceled::match_and_decode(log) {
                        return Some(contract::UsdcAuthorizationCanceled {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            authorizer: event.authorizer,
                            nonce: Vec::from(event.nonce),
                        });
                    }

                    None
                })
        })
        .collect());
    events.usdc_authorization_useds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == USDC_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::usdc_contract::events::AuthorizationUsed::match_and_decode(log) {
                        return Some(contract::UsdcAuthorizationUsed {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            authorizer: event.authorizer,
                            nonce: Vec::from(event.nonce),
                        });
                    }

                    None
                })
        })
        .collect());
    events.usdc_blacklisteds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == USDC_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::usdc_contract::events::Blacklisted::match_and_decode(log) {
                        return Some(contract::UsdcBlacklisted {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            u_account: event.u_account,
                        });
                    }

                    None
                })
        })
        .collect());
    events.usdc_blacklister_changeds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == USDC_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::usdc_contract::events::BlacklisterChanged::match_and_decode(log) {
                        return Some(contract::UsdcBlacklisterChanged {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            new_blacklister: event.new_blacklister,
                        });
                    }

                    None
                })
        })
        .collect());
    events.usdc_burns.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == USDC_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::usdc_contract::events::Burn::match_and_decode(log) {
                        return Some(contract::UsdcBurn {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            amount: event.amount.to_string(),
                            burner: event.burner,
                        });
                    }

                    None
                })
        })
        .collect());
    events.usdc_master_minter_changeds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == USDC_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::usdc_contract::events::MasterMinterChanged::match_and_decode(log) {
                        return Some(contract::UsdcMasterMinterChanged {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            new_master_minter: event.new_master_minter,
                        });
                    }

                    None
                })
        })
        .collect());
    events.usdc_mints.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == USDC_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::usdc_contract::events::Mint::match_and_decode(log) {
                        return Some(contract::UsdcMint {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            amount: event.amount.to_string(),
                            minter: event.minter,
                            to: event.to,
                        });
                    }

                    None
                })
        })
        .collect());
    events.usdc_minter_configureds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == USDC_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::usdc_contract::events::MinterConfigured::match_and_decode(log) {
                        return Some(contract::UsdcMinterConfigured {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            minter: event.minter,
                            minter_allowed_amount: event.minter_allowed_amount.to_string(),
                        });
                    }

                    None
                })
        })
        .collect());
    events.usdc_minter_removeds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == USDC_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::usdc_contract::events::MinterRemoved::match_and_decode(log) {
                        return Some(contract::UsdcMinterRemoved {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            old_minter: event.old_minter,
                        });
                    }

                    None
                })
        })
        .collect());
    events.usdc_ownership_transferreds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == USDC_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::usdc_contract::events::OwnershipTransferred::match_and_decode(log) {
                        return Some(contract::UsdcOwnershipTransferred {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            new_owner: event.new_owner,
                            previous_owner: event.previous_owner,
                        });
                    }

                    None
                })
        })
        .collect());
    events.usdc_pauses.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == USDC_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::usdc_contract::events::Pause::match_and_decode(log) {
                        return Some(contract::UsdcPause {
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
    events.usdc_pauser_changeds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == USDC_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::usdc_contract::events::PauserChanged::match_and_decode(log) {
                        return Some(contract::UsdcPauserChanged {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            new_address: event.new_address,
                        });
                    }

                    None
                })
        })
        .collect());
    events.usdc_rescuer_changeds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == USDC_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::usdc_contract::events::RescuerChanged::match_and_decode(log) {
                        return Some(contract::UsdcRescuerChanged {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            new_rescuer: event.new_rescuer,
                        });
                    }

                    None
                })
        })
        .collect());
    events.usdc_transfers.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == USDC_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::usdc_contract::events::Transfer::match_and_decode(log) {
                        return Some(contract::UsdcTransfer {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            from: event.from,
                            to: event.to,
                            value: event.value.to_string(),
                        });
                    }

                    None
                })
        })
        .collect());
    events.usdc_un_blacklisteds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == USDC_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::usdc_contract::events::UnBlacklisted::match_and_decode(log) {
                        return Some(contract::UsdcUnBlacklisted {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            u_account: event.u_account,
                        });
                    }

                    None
                })
        })
        .collect());
    events.usdc_unpauses.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == USDC_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::usdc_contract::events::Unpause::match_and_decode(log) {
                        return Some(contract::UsdcUnpause {
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
    events.usdc_upgradeds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == USDC_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::usdc_contract::events::Upgraded::match_and_decode(log) {
                        return Some(contract::UsdcUpgraded {
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
fn map_usdc_calls(blk: &eth::Block, calls: &mut contract::Calls) {
    calls.usdc_call_approves.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == USDC_TRACKED_CONTRACT && abi::usdc_contract::functions::Approve::match_call(call))
                .filter_map(|call| {
                    match abi::usdc_contract::functions::Approve::decode(call) {
                        Ok(decoded_call) => {
                            let output_param0 = match abi::usdc_contract::functions::Approve::output(&call.return_data) {
                                Ok(output_param0) => {output_param0}
                                Err(_) => Default::default(),
                            };
                            
                            Some(contract::UsdcApproveCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
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
    calls.usdc_call_blacklists.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == USDC_TRACKED_CONTRACT && abi::usdc_contract::functions::Blacklist::match_call(call))
                .filter_map(|call| {
                    match abi::usdc_contract::functions::Blacklist::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::UsdcBlacklistCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                u_account: decoded_call.u_account,
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.usdc_call_burns.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == USDC_TRACKED_CONTRACT && abi::usdc_contract::functions::Burn::match_call(call))
                .filter_map(|call| {
                    match abi::usdc_contract::functions::Burn::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::UsdcBurnCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                u_amount: decoded_call.u_amount.to_string(),
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.usdc_call_cancel_authorization_1s.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == USDC_TRACKED_CONTRACT && abi::usdc_contract::functions::CancelAuthorization1::match_call(call))
                .filter_map(|call| {
                    match abi::usdc_contract::functions::CancelAuthorization1::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::UsdcCancelAuthorization1call {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                authorizer: decoded_call.authorizer,
                                nonce: Vec::from(decoded_call.nonce),
                                r: Vec::from(decoded_call.r),
                                s: Vec::from(decoded_call.s),
                                v: decoded_call.v.to_u64(),
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.usdc_call_cancel_authorization_2s.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == USDC_TRACKED_CONTRACT && abi::usdc_contract::functions::CancelAuthorization2::match_call(call))
                .filter_map(|call| {
                    match abi::usdc_contract::functions::CancelAuthorization2::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::UsdcCancelAuthorization2call {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                authorizer: decoded_call.authorizer,
                                nonce: Vec::from(decoded_call.nonce),
                                signature: decoded_call.signature,
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.usdc_call_change_admins.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == USDC_TRACKED_CONTRACT && abi::usdc_contract::functions::ChangeAdmin::match_call(call))
                .filter_map(|call| {
                    match abi::usdc_contract::functions::ChangeAdmin::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::UsdcChangeAdminCall {
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
    calls.usdc_call_configure_minters.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == USDC_TRACKED_CONTRACT && abi::usdc_contract::functions::ConfigureMinter::match_call(call))
                .filter_map(|call| {
                    match abi::usdc_contract::functions::ConfigureMinter::decode(call) {
                        Ok(decoded_call) => {
                            let output_param0 = match abi::usdc_contract::functions::ConfigureMinter::output(&call.return_data) {
                                Ok(output_param0) => {output_param0}
                                Err(_) => Default::default(),
                            };
                            
                            Some(contract::UsdcConfigureMinterCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                minter: decoded_call.minter,
                                minter_allowed_amount: decoded_call.minter_allowed_amount.to_string(),
                                output_param0: output_param0,
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.usdc_call_decrease_allowances.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == USDC_TRACKED_CONTRACT && abi::usdc_contract::functions::DecreaseAllowance::match_call(call))
                .filter_map(|call| {
                    match abi::usdc_contract::functions::DecreaseAllowance::decode(call) {
                        Ok(decoded_call) => {
                            let output_param0 = match abi::usdc_contract::functions::DecreaseAllowance::output(&call.return_data) {
                                Ok(output_param0) => {output_param0}
                                Err(_) => Default::default(),
                            };
                            
                            Some(contract::UsdcDecreaseAllowanceCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                decrement: decoded_call.decrement.to_string(),
                                output_param0: output_param0,
                                spender: decoded_call.spender,
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.usdc_call_increase_allowances.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == USDC_TRACKED_CONTRACT && abi::usdc_contract::functions::IncreaseAllowance::match_call(call))
                .filter_map(|call| {
                    match abi::usdc_contract::functions::IncreaseAllowance::decode(call) {
                        Ok(decoded_call) => {
                            let output_param0 = match abi::usdc_contract::functions::IncreaseAllowance::output(&call.return_data) {
                                Ok(output_param0) => {output_param0}
                                Err(_) => Default::default(),
                            };
                            
                            Some(contract::UsdcIncreaseAllowanceCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                increment: decoded_call.increment.to_string(),
                                output_param0: output_param0,
                                spender: decoded_call.spender,
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.usdc_call_initializes.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == USDC_TRACKED_CONTRACT && abi::usdc_contract::functions::Initialize::match_call(call))
                .filter_map(|call| {
                    match abi::usdc_contract::functions::Initialize::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::UsdcInitializeCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                new_blacklister: decoded_call.new_blacklister,
                                new_master_minter: decoded_call.new_master_minter,
                                new_owner: decoded_call.new_owner,
                                new_pauser: decoded_call.new_pauser,
                                token_currency: decoded_call.token_currency,
                                token_decimals: decoded_call.token_decimals.to_u64(),
                                token_name: decoded_call.token_name,
                                token_symbol: decoded_call.token_symbol,
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.usdc_call_initialize_v_2s.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == USDC_TRACKED_CONTRACT && abi::usdc_contract::functions::InitializeV2::match_call(call))
                .filter_map(|call| {
                    match abi::usdc_contract::functions::InitializeV2::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::UsdcInitializeV2call {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                new_name: decoded_call.new_name,
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.usdc_call_initialize_v2_1s.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == USDC_TRACKED_CONTRACT && abi::usdc_contract::functions::InitializeV21::match_call(call))
                .filter_map(|call| {
                    match abi::usdc_contract::functions::InitializeV21::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::UsdcInitializeV21call {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                lost_and_found: decoded_call.lost_and_found,
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.usdc_call_initialize_v2_2s.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == USDC_TRACKED_CONTRACT && abi::usdc_contract::functions::InitializeV22::match_call(call))
                .filter_map(|call| {
                    match abi::usdc_contract::functions::InitializeV22::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::UsdcInitializeV22call {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                accounts_to_blacklist: decoded_call.accounts_to_blacklist.into_iter().map(|x| x).collect::<Vec<_>>(),
                                new_symbol: decoded_call.new_symbol,
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.usdc_call_mints.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == USDC_TRACKED_CONTRACT && abi::usdc_contract::functions::Mint::match_call(call))
                .filter_map(|call| {
                    match abi::usdc_contract::functions::Mint::decode(call) {
                        Ok(decoded_call) => {
                            let output_param0 = match abi::usdc_contract::functions::Mint::output(&call.return_data) {
                                Ok(output_param0) => {output_param0}
                                Err(_) => Default::default(),
                            };
                            
                            Some(contract::UsdcMintCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                output_param0: output_param0,
                                u_amount: decoded_call.u_amount.to_string(),
                                u_to: decoded_call.u_to,
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.usdc_call_pauses.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == USDC_TRACKED_CONTRACT && abi::usdc_contract::functions::Pause::match_call(call))
                .filter_map(|call| {
                    match abi::usdc_contract::functions::Pause::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::UsdcPauseCall {
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
    calls.usdc_call_permit_1s.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == USDC_TRACKED_CONTRACT && abi::usdc_contract::functions::Permit1::match_call(call))
                .filter_map(|call| {
                    match abi::usdc_contract::functions::Permit1::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::UsdcPermit1call {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                deadline: decoded_call.deadline.to_string(),
                                owner: decoded_call.owner,
                                signature: decoded_call.signature,
                                spender: decoded_call.spender,
                                value: decoded_call.value.to_string(),
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.usdc_call_permit_2s.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == USDC_TRACKED_CONTRACT && abi::usdc_contract::functions::Permit2::match_call(call))
                .filter_map(|call| {
                    match abi::usdc_contract::functions::Permit2::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::UsdcPermit2call {
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
    calls.usdc_call_receive_with_authorization_1s.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == USDC_TRACKED_CONTRACT && abi::usdc_contract::functions::ReceiveWithAuthorization1::match_call(call))
                .filter_map(|call| {
                    match abi::usdc_contract::functions::ReceiveWithAuthorization1::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::UsdcReceiveWithAuthorization1call {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                from: decoded_call.from,
                                nonce: Vec::from(decoded_call.nonce),
                                signature: decoded_call.signature,
                                to: decoded_call.to,
                                valid_after: decoded_call.valid_after.to_string(),
                                valid_before: decoded_call.valid_before.to_string(),
                                value: decoded_call.value.to_string(),
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.usdc_call_receive_with_authorization_2s.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == USDC_TRACKED_CONTRACT && abi::usdc_contract::functions::ReceiveWithAuthorization2::match_call(call))
                .filter_map(|call| {
                    match abi::usdc_contract::functions::ReceiveWithAuthorization2::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::UsdcReceiveWithAuthorization2call {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                from: decoded_call.from,
                                nonce: Vec::from(decoded_call.nonce),
                                r: Vec::from(decoded_call.r),
                                s: Vec::from(decoded_call.s),
                                to: decoded_call.to,
                                v: decoded_call.v.to_u64(),
                                valid_after: decoded_call.valid_after.to_string(),
                                valid_before: decoded_call.valid_before.to_string(),
                                value: decoded_call.value.to_string(),
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.usdc_call_remove_minters.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == USDC_TRACKED_CONTRACT && abi::usdc_contract::functions::RemoveMinter::match_call(call))
                .filter_map(|call| {
                    match abi::usdc_contract::functions::RemoveMinter::decode(call) {
                        Ok(decoded_call) => {
                            let output_param0 = match abi::usdc_contract::functions::RemoveMinter::output(&call.return_data) {
                                Ok(output_param0) => {output_param0}
                                Err(_) => Default::default(),
                            };
                            
                            Some(contract::UsdcRemoveMinterCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                minter: decoded_call.minter,
                                output_param0: output_param0,
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.usdc_call_rescue_erc_20s.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == USDC_TRACKED_CONTRACT && abi::usdc_contract::functions::RescueErc20::match_call(call))
                .filter_map(|call| {
                    match abi::usdc_contract::functions::RescueErc20::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::UsdcRescueErc20call {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                amount: decoded_call.amount.to_string(),
                                to: decoded_call.to,
                                token_contract: decoded_call.token_contract,
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.usdc_call_transfers.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == USDC_TRACKED_CONTRACT && abi::usdc_contract::functions::Transfer::match_call(call))
                .filter_map(|call| {
                    match abi::usdc_contract::functions::Transfer::decode(call) {
                        Ok(decoded_call) => {
                            let output_param0 = match abi::usdc_contract::functions::Transfer::output(&call.return_data) {
                                Ok(output_param0) => {output_param0}
                                Err(_) => Default::default(),
                            };
                            
                            Some(contract::UsdcTransferCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
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
    calls.usdc_call_transfer_froms.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == USDC_TRACKED_CONTRACT && abi::usdc_contract::functions::TransferFrom::match_call(call))
                .filter_map(|call| {
                    match abi::usdc_contract::functions::TransferFrom::decode(call) {
                        Ok(decoded_call) => {
                            let output_param0 = match abi::usdc_contract::functions::TransferFrom::output(&call.return_data) {
                                Ok(output_param0) => {output_param0}
                                Err(_) => Default::default(),
                            };
                            
                            Some(contract::UsdcTransferFromCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
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
    calls.usdc_call_transfer_ownerships.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == USDC_TRACKED_CONTRACT && abi::usdc_contract::functions::TransferOwnership::match_call(call))
                .filter_map(|call| {
                    match abi::usdc_contract::functions::TransferOwnership::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::UsdcTransferOwnershipCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                new_owner: decoded_call.new_owner,
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.usdc_call_transfer_with_authorization_1s.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == USDC_TRACKED_CONTRACT && abi::usdc_contract::functions::TransferWithAuthorization1::match_call(call))
                .filter_map(|call| {
                    match abi::usdc_contract::functions::TransferWithAuthorization1::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::UsdcTransferWithAuthorization1call {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                from: decoded_call.from,
                                nonce: Vec::from(decoded_call.nonce),
                                signature: decoded_call.signature,
                                to: decoded_call.to,
                                valid_after: decoded_call.valid_after.to_string(),
                                valid_before: decoded_call.valid_before.to_string(),
                                value: decoded_call.value.to_string(),
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.usdc_call_transfer_with_authorization_2s.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == USDC_TRACKED_CONTRACT && abi::usdc_contract::functions::TransferWithAuthorization2::match_call(call))
                .filter_map(|call| {
                    match abi::usdc_contract::functions::TransferWithAuthorization2::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::UsdcTransferWithAuthorization2call {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                from: decoded_call.from,
                                nonce: Vec::from(decoded_call.nonce),
                                r: Vec::from(decoded_call.r),
                                s: Vec::from(decoded_call.s),
                                to: decoded_call.to,
                                v: decoded_call.v.to_u64(),
                                valid_after: decoded_call.valid_after.to_string(),
                                valid_before: decoded_call.valid_before.to_string(),
                                value: decoded_call.value.to_string(),
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.usdc_call_un_blacklists.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == USDC_TRACKED_CONTRACT && abi::usdc_contract::functions::UnBlacklist::match_call(call))
                .filter_map(|call| {
                    match abi::usdc_contract::functions::UnBlacklist::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::UsdcUnBlacklistCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                u_account: decoded_call.u_account,
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.usdc_call_unpauses.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == USDC_TRACKED_CONTRACT && abi::usdc_contract::functions::Unpause::match_call(call))
                .filter_map(|call| {
                    match abi::usdc_contract::functions::Unpause::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::UsdcUnpauseCall {
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
    calls.usdc_call_update_blacklisters.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == USDC_TRACKED_CONTRACT && abi::usdc_contract::functions::UpdateBlacklister::match_call(call))
                .filter_map(|call| {
                    match abi::usdc_contract::functions::UpdateBlacklister::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::UsdcUpdateBlacklisterCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                u_new_blacklister: decoded_call.u_new_blacklister,
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.usdc_call_update_master_minters.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == USDC_TRACKED_CONTRACT && abi::usdc_contract::functions::UpdateMasterMinter::match_call(call))
                .filter_map(|call| {
                    match abi::usdc_contract::functions::UpdateMasterMinter::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::UsdcUpdateMasterMinterCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                u_new_master_minter: decoded_call.u_new_master_minter,
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.usdc_call_update_pausers.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == USDC_TRACKED_CONTRACT && abi::usdc_contract::functions::UpdatePauser::match_call(call))
                .filter_map(|call| {
                    match abi::usdc_contract::functions::UpdatePauser::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::UsdcUpdatePauserCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                u_new_pauser: decoded_call.u_new_pauser,
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.usdc_call_update_rescuers.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == USDC_TRACKED_CONTRACT && abi::usdc_contract::functions::UpdateRescuer::match_call(call))
                .filter_map(|call| {
                    match abi::usdc_contract::functions::UpdateRescuer::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::UsdcUpdateRescuerCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                new_rescuer: decoded_call.new_rescuer,
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
    calls.usdc_call_upgrade_tos.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == USDC_TRACKED_CONTRACT && abi::usdc_contract::functions::UpgradeTo::match_call(call))
                .filter_map(|call| {
                    match abi::usdc_contract::functions::UpgradeTo::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::UsdcUpgradeToCall {
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
    calls.usdc_call_upgrade_to_and_calls.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == USDC_TRACKED_CONTRACT && abi::usdc_contract::functions::UpgradeToAndCall::match_call(call))
                .filter_map(|call| {
                    match abi::usdc_contract::functions::UpgradeToAndCall::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::UsdcUpgradeToAndCallCall {
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
    map_sdai_events(&blk, &mut events);
    map_dai_usds_converter_events(&blk, &mut events);
    map_dai_lite_psm_events(&blk, &mut events);
    map_usds_psm_wrapper_events(&blk, &mut events);
    map_susds_events(&blk, &mut events);
    map_mkr_sky_converter_events(&blk, &mut events);
    map_usdc_events(&blk, &mut events);
    Ok(events)
}
#[substreams::handlers::map]
fn map_calls(blk: eth::Block) -> Result<contract::Calls, substreams::errors::Error> {
let mut calls = contract::Calls::default();
    map_sdai_calls(&blk, &mut calls);
    map_dai_usds_converter_calls(&blk, &mut calls);
    map_dai_lite_psm_calls(&blk, &mut calls);
    map_usds_psm_wrapper_calls(&blk, &mut calls);
    map_susds_calls(&blk, &mut calls);
    map_mkr_sky_converter_calls(&blk, &mut calls);
    map_usdc_calls(&blk, &mut calls);
    Ok(calls)
}

