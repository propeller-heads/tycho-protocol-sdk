use crate::abi;
use crate::pb::contract::v1 as contract;
use substreams::Hex;
use substreams_ethereum::pb::eth::v2 as eth;
use substreams_ethereum::Event;

pub fn map_sdai_events(blk: &eth::Block, events: &mut contract::Events) {
    // Approvals
    events.sdai_approvals.append(
        &mut blk
            .receipts()
            .flat_map(|view| {
                view.receipt
                    .logs
                    .iter()
                    .filter(|log| log.address == crate::SDAI_TRACKED_CONTRACT)
                    .filter_map(|log| {
                        if let Some(event) =
                            abi::sdai_contract::events::Approval::match_and_decode(log)
                        {
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
            .collect(),
    );

    // Deposits
    events.sdai_deposits.append(
        &mut blk
            .receipts()
            .flat_map(|view| {
                view.receipt
                    .logs
                    .iter()
                    .filter(|log| log.address == crate::SDAI_TRACKED_CONTRACT)
                    .filter_map(|log| {
                        if let Some(event) =
                            abi::sdai_contract::events::Deposit::match_and_decode(log)
                        {
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
            .collect(),
    );

    // Transfers
    events.sdai_transfers.append(
        &mut blk
            .receipts()
            .flat_map(|view| {
                view.receipt
                    .logs
                    .iter()
                    .filter(|log| log.address == crate::SDAI_TRACKED_CONTRACT)
                    .filter_map(|log| {
                        if let Some(event) =
                            abi::sdai_contract::events::Transfer::match_and_decode(log)
                        {
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
            .collect(),
    );

    // Withdraws
    events.sdai_withdraws.append(
        &mut blk
            .receipts()
            .flat_map(|view| {
                view.receipt
                    .logs
                    .iter()
                    .filter(|log| log.address == crate::SDAI_TRACKED_CONTRACT)
                    .filter_map(|log| {
                        if let Some(event) =
                            abi::sdai_contract::events::Withdraw::match_and_decode(log)
                        {
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
            .collect(),
    );
}

pub fn map_dai_usds_converter_events(blk: &eth::Block, events: &mut contract::Events) {
    // DaiToUsds events
    events
        .dai_usds_converter_dai_to_usds
        .append(
            &mut blk
                .receipts()
                .flat_map(|view| {
                    view.receipt
                        .logs
                        .iter()
                        .filter(|log| log.address == crate::DAI_USDS_CONVERTER_TRACKED_CONTRACT)
                        .filter_map(|log| {
                            if let Some(event) =
                            abi::dai_usds_converter_contract::events::DaiToUsds::match_and_decode(
                                log,
                            )
                        {
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
                .collect(),
        );

    // UsdsToDai events
    events
        .dai_usds_converter_usds_to_dais
        .append(
            &mut blk
                .receipts()
                .flat_map(|view| {
                    view.receipt
                        .logs
                        .iter()
                        .filter(|log| log.address == crate::DAI_USDS_CONVERTER_TRACKED_CONTRACT)
                        .filter_map(|log| {
                            if let Some(event) =
                            abi::dai_usds_converter_contract::events::UsdsToDai::match_and_decode(
                                log,
                            )
                        {
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
                .collect(),
        );
}

pub fn map_dai_lite_psm_events(blk: &eth::Block, events: &mut contract::Events) {
    // BuyGem events
    events.dai_lite_psm_buy_gems.append(
        &mut blk
            .receipts()
            .flat_map(|view| {
                view.receipt
                    .logs
                    .iter()
                    .filter(|log| log.address == crate::DAI_LITE_PSM_TRACKED_CONTRACT)
                    .filter_map(|log| {
                        if let Some(event) =
                            abi::dai_lite_psm_contract::events::BuyGem::match_and_decode(log)
                        {
                            return Some(contract::DaiLitePsmBuyGem {
                                evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                                evt_index: log.block_index,
                                evt_block_time: Some(blk.timestamp().to_owned()),
                                evt_block_number: blk.number,
                                owner: event.owner,
                                value: event.value.to_string(),
                                fee: event.fee.to_string(),
                            });
                        }
                        None
                    })
            })
            .collect(),
    );

    // SellGem events
    events.dai_lite_psm_sell_gems.append(
        &mut blk
            .receipts()
            .flat_map(|view| {
                view.receipt
                    .logs
                    .iter()
                    .filter(|log| log.address == crate::DAI_LITE_PSM_TRACKED_CONTRACT)
                    .filter_map(|log| {
                        if let Some(event) =
                            abi::dai_lite_psm_contract::events::SellGem::match_and_decode(log)
                        {
                            return Some(contract::DaiLitePsmSellGem {
                                evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                                evt_index: log.block_index,
                                evt_block_time: Some(blk.timestamp().to_owned()),
                                evt_block_number: blk.number,
                                owner: event.owner,
                                value: event.value.to_string(),
                                fee: event.fee.to_string(),
                            });
                        }
                        None
                    })
            })
            .collect(),
    );
}

pub fn map_usds_psm_wrapper_events(blk: &eth::Block, events: &mut contract::Events) {
    // BuyGem events
    events.usds_psm_wrapper_buy_gems.append(
        &mut blk
            .receipts()
            .flat_map(|view| {
                view.receipt
                    .logs
                    .iter()
                    .filter(|log| log.address == crate::USDS_PSM_WRAPPER_TRACKED_CONTRACT)
                    .filter_map(|log| {
                        if let Some(event) =
                            abi::usds_psm_wrapper_contract::events::BuyGem::match_and_decode(log)
                        {
                            return Some(contract::UsdsPsmWrapperBuyGem {
                                evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                                evt_index: log.block_index,
                                evt_block_time: Some(blk.timestamp().to_owned()),
                                evt_block_number: blk.number,
                                usr: event.usr,
                                gem_amt: event.gem_amt.to_string(),
                                usds_in_wad: event.usds_in_wad.to_string(),
                            });
                        }
                        None
                    })
            })
            .collect(),
    );

    // SellGem events
    events
        .usds_psm_wrapper_sell_gems
        .append(
            &mut blk
                .receipts()
                .flat_map(|view| {
                    view.receipt
                        .logs
                        .iter()
                        .filter(|log| log.address == crate::USDS_PSM_WRAPPER_TRACKED_CONTRACT)
                        .filter_map(|log| {
                            if let Some(event) =
                                abi::usds_psm_wrapper_contract::events::SellGem::match_and_decode(
                                    log,
                                )
                            {
                                return Some(contract::UsdsPsmWrapperSellGem {
                                    evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                                    evt_index: log.block_index,
                                    evt_block_time: Some(blk.timestamp().to_owned()),
                                    evt_block_number: blk.number,
                                    usr: event.usr,
                                    gem_amt: event.gem_amt.to_string(),
                                    usds_out_wad: event.usds_out_wad.to_string(),
                                });
                            }
                            None
                        })
                })
                .collect(),
        );
}

pub fn map_susds_events(blk: &eth::Block, events: &mut contract::Events) {
    // Deposit events
    events.susds_deposits.append(
        &mut blk
            .receipts()
            .flat_map(|view| {
                view.receipt
                    .logs
                    .iter()
                    .filter(|log| log.address == crate::SUSDS_TRACKED_CONTRACT)
                    .filter_map(|log| {
                        if let Some(event) =
                            abi::susds_contract::events::Deposit::match_and_decode(log)
                        {
                            return Some(contract::SusdsDeposit {
                                evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                                evt_index: log.block_index,
                                evt_block_time: Some(blk.timestamp().to_owned()),
                                evt_block_number: blk.number,
                                assets: event.assets.to_string(),
                                shares: event.shares.to_string(),
                                owner: event.owner,
                                sender: event.sender,
                            });
                        }
                        None
                    })
            })
            .collect(),
    );

    // Withdraw events
    events.susds_withdraws.append(
        &mut blk
            .receipts()
            .flat_map(|view| {
                view.receipt
                    .logs
                    .iter()
                    .filter(|log| log.address == crate::SUSDS_TRACKED_CONTRACT)
                    .filter_map(|log| {
                        if let Some(event) =
                            abi::susds_contract::events::Withdraw::match_and_decode(log)
                        {
                            return Some(contract::SusdsWithdraw {
                                evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                                evt_index: log.block_index,
                                evt_block_time: Some(blk.timestamp().to_owned()),
                                evt_block_number: blk.number,
                                assets: event.assets.to_string(),
                                shares: event.shares.to_string(),
                                owner: event.owner,
                                receiver: event.receiver,
                            });
                        }
                        None
                    })
            })
            .collect(),
    );
}

pub fn map_mkr_sky_converter_events(blk: &eth::Block, events: &mut contract::Events) {
    // MkrToSky events
    events
        .mkr_sky_converter_mkr_to_skies
        .append(
            &mut blk
                .receipts()
                .flat_map(|view| {
                    view.receipt
                        .logs
                        .iter()
                        .filter(|log| log.address == crate::MKR_SKY_CONVERTER_TRACKED_CONTRACT)
                        .filter_map(|log| {
                            if let Some(event) =
                                abi::mkr_sky_converter_contract::events::MkrToSky::match_and_decode(
                                    log,
                                )
                            {
                                return Some(contract::MkrSkyConverterMkrToSky {
                                    evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                                    evt_index: log.block_index,
                                    evt_block_time: Some(blk.timestamp().to_owned()),
                                    evt_block_number: blk.number,
                                    caller: event.caller,
                                    usr: event.usr,
                                    mkr_amt: event.mkr_amt.to_string(),
                                    sky_amt: event.sky_amt.to_string(),
                                });
                            }
                            None
                        })
                })
                .collect(),
        );

    // SkyToMkr events
    events
        .mkr_sky_converter_sky_to_mkrs
        .append(
            &mut blk
                .receipts()
                .flat_map(|view| {
                    view.receipt
                        .logs
                        .iter()
                        .filter(|log| log.address == crate::MKR_SKY_CONVERTER_TRACKED_CONTRACT)
                        .filter_map(|log| {
                            if let Some(event) =
                                abi::mkr_sky_converter_contract::events::SkyToMkr::match_and_decode(
                                    log,
                                )
                            {
                                return Some(contract::MkrSkyConverterSkyToMkr {
                                    evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                                    evt_index: log.block_index,
                                    evt_block_time: Some(blk.timestamp().to_owned()),
                                    evt_block_number: blk.number,
                                    caller: event.caller,
                                    usr: event.usr,
                                    sky_amt: event.sky_amt.to_string(),
                                    mkr_amt: event.mkr_amt.to_string(),
                                });
                            }
                            None
                        })
                })
                .collect(),
        );
}

// Add other event mapping functions for each contract...
