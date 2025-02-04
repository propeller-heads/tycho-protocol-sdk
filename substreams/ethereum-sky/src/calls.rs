use crate::abi;
use crate::pb::contract::v1 as contract;
use substreams::Hex;
use substreams_ethereum::pb::eth::v2 as eth;

pub fn map_sdai_calls(blk: &eth::Block, calls: &mut contract::Calls) {
    // Approve calls
    calls.sdai_call_approves.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == crate::SDAI_TRACKED_CONTRACT && 
                    abi::sdai_contract::functions::Approve::match_call(call))
                .filter_map(|call| {
                    match abi::sdai_contract::functions::Approve::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::SdaiApproveCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                spender: decoded_call.spender,
                                value: decoded_call.value.to_string(),
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());

    // Deposit calls
    calls.sdai_call_deposits.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == crate::SDAI_TRACKED_CONTRACT && 
                    abi::sdai_contract::functions::Deposit::match_call(call))
                .filter_map(|call| {
                    match abi::sdai_contract::functions::Deposit::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::SdaiDepositCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                assets: decoded_call.assets.to_string(),
                                receiver: decoded_call.receiver,
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());

    // Withdraw calls
    calls.sdai_call_withdraws.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == crate::SDAI_TRACKED_CONTRACT && 
                    abi::sdai_contract::functions::Withdraw::match_call(call))
                .filter_map(|call| {
                    match abi::sdai_contract::functions::Withdraw::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::SdaiWithdrawCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                assets: decoded_call.assets.to_string(),
                                receiver: decoded_call.receiver,
                                owner: decoded_call.owner,
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
}

pub fn map_dai_usds_converter_calls(blk: &eth::Block, calls: &mut contract::Calls) {
    // DaiToUsds calls
    calls.dai_usds_converter_call_dai_to_usds.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == crate::DAI_USDS_CONVERTER_TRACKED_CONTRACT && 
                    abi::dai_usds_converter_contract::functions::DaiToUsds::match_call(call))
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

    // UsdsToDai calls
    calls.dai_usds_converter_call_usds_to_dais.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == crate::DAI_USDS_CONVERTER_TRACKED_CONTRACT && 
                    abi::dai_usds_converter_contract::functions::UsdsToDai::match_call(call))
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

pub fn map_dai_lite_psm_calls(blk: &eth::Block, calls: &mut contract::Calls) {
    // BuyGem calls
    calls.dai_lite_psm_call_buy_gems.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == crate::DAI_LITE_PSM_TRACKED_CONTRACT && 
                    abi::dai_lite_psm_contract::functions::BuyGem::match_call(call))
                .filter_map(|call| {
                    match abi::dai_lite_psm_contract::functions::BuyGem::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::DaiLitePsmBuyGemCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                usr: decoded_call.usr,
                                value: decoded_call.value.to_string(),
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());

    // SellGem calls
    calls.dai_lite_psm_call_sell_gems.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == crate::DAI_LITE_PSM_TRACKED_CONTRACT && 
                    abi::dai_lite_psm_contract::functions::SellGem::match_call(call))
                .filter_map(|call| {
                    match abi::dai_lite_psm_contract::functions::SellGem::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::DaiLitePsmSellGemCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                usr: decoded_call.usr,
                                value: decoded_call.value.to_string(),
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
}

pub fn map_usds_psm_wrapper_calls(blk: &eth::Block, calls: &mut contract::Calls) {
    // BuyGem calls
    calls.usds_psm_wrapper_call_buy_gems.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == crate::USDS_PSM_WRAPPER_TRACKED_CONTRACT && 
                    abi::usds_psm_wrapper_contract::functions::BuyGem::match_call(call))
                .filter_map(|call| {
                    match abi::usds_psm_wrapper_contract::functions::BuyGem::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::UsdsPsmWrapperBuyGemCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                usr: decoded_call.usr,
                                gem_amt: decoded_call.gem_amt.to_string(),
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());

    // SellGem calls
    calls.usds_psm_wrapper_call_sell_gems.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == crate::USDS_PSM_WRAPPER_TRACKED_CONTRACT && 
                    abi::usds_psm_wrapper_contract::functions::SellGem::match_call(call))
                .filter_map(|call| {
                    match abi::usds_psm_wrapper_contract::functions::SellGem::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::UsdsPsmWrapperSellGemCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                usr: decoded_call.usr,
                                gem_amt: decoded_call.gem_amt.to_string(),
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
}

pub fn map_susds_calls(blk: &eth::Block, calls: &mut contract::Calls) {
    // Deposit calls
    calls.susds_call_deposits.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == crate::SUSDS_TRACKED_CONTRACT && 
                    abi::susds_contract::functions::Deposit::match_call(call))
                .filter_map(|call| {
                    match abi::susds_contract::functions::Deposit::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::SusdsDepositCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                assets: decoded_call.assets.to_string(),
                                receiver: decoded_call.receiver,
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());

    // Withdraw calls
    calls.susds_call_withdraws.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == crate::SUSDS_TRACKED_CONTRACT && 
                    abi::susds_contract::functions::Withdraw::match_call(call))
                .filter_map(|call| {
                    match abi::susds_contract::functions::Withdraw::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::SusdsWithdrawCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                assets: decoded_call.assets.to_string(),
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

pub fn map_mkr_sky_converter_calls(blk: &eth::Block, calls: &mut contract::Calls) {
    // MkrToSky calls
    calls.mkr_sky_converter_call_mkr_to_skies.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == crate::MKR_SKY_CONVERTER_TRACKED_CONTRACT && 
                    abi::mkr_sky_converter_contract::functions::MkrToSky::match_call(call))
                .filter_map(|call| {
                    match abi::mkr_sky_converter_contract::functions::MkrToSky::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::MkrSkyConverterMkrToSkyCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                usr: decoded_call.usr,
                                mkr_amt: decoded_call.mkr_amt.to_string(),
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());

    // SkyToMkr calls
    calls.mkr_sky_converter_call_sky_to_mkrs.append(&mut blk
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter()
                .filter(|call| call.address == crate::MKR_SKY_CONVERTER_TRACKED_CONTRACT && 
                    abi::mkr_sky_converter_contract::functions::SkyToMkr::match_call(call))
                .filter_map(|call| {
                    match abi::mkr_sky_converter_contract::functions::SkyToMkr::decode(call) {
                        Ok(decoded_call) => {
                            Some(contract::MkrSkyConverterSkyToMkrCall {
                                call_tx_hash: Hex(&tx.hash).to_string(),
                                call_block_time: Some(blk.timestamp().to_owned()),
                                call_block_number: blk.number,
                                call_ordinal: call.begin_ordinal,
                                call_success: !call.state_reverted,
                                usr: decoded_call.usr,
                                sky_amt: decoded_call.sky_amt.to_string(),
                            })
                        },
                        Err(_) => None,
                    }
                })
        })
        .collect());
}

// ... Continue with other contracts' call mappings 