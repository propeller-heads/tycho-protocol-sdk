use crate::abi;
use anyhow::Result;
use itertools::Itertools;
use std::collections::HashMap;
use substreams::{
    hex,
    pb::substreams::StoreDeltas,
    store::{StoreAddBigInt, StoreGet, StoreGetString, StoreNew, StoreSet, StoreSetString},
};
use substreams_ethereum::{pb::eth, Event};
use tycho_substreams::{
    balances::aggregate_balances_changes, contract::extract_contract_changes_builder, prelude::*,
};

pub const SDAI_VAULT_ADDRESS: &[u8] = &hex!("83F20F44975D03b1b09e64809B757c47f942BEeA");
pub const DAI_USDS_CONVERTER_ADDRESS: &[u8] = &hex!("3225737a9Bbb6473CB4a45b7244ACa2BeFdB276A");
pub const DAI_LITE_PSM_ADDRESS: &[u8] = &hex!("f6e72Db5454dd049d0788e411b06CfAF16853042");
pub const USDS_PSM_WRAPPER_ADDRESS: &[u8] = &hex!("A188EEC8F81263234dA3622A406892F3D630f98c");
pub const SUSDS_ADDRESS: &[u8] = &hex!("a3931d71877C0E7a3148CB7Eb4463524FEc27fbD");
pub const MKR_SKY_CONVERTER_ADDRESS: &[u8] = &hex!("BDcFCA946b6CDd965f99a839e4435Bcdc1bc470B");
pub const USDC_TOKEN_ADDRESS: &[u8] = &hex!("A0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48");
pub const DAI_TOKEN_ADDRESS: &[u8] = &hex!("6B175474E89094C44Da98b954EedeAC495271d0F");
pub const USDS_TOKEN_ADDRESS: &[u8] = &hex!("dC035D45d973E3EC169d2276DDab16f1e407384F");
pub const SUSDS_TOKEN_ADDRESS: &[u8] = &hex!("a3931d71877C0E7a3148CB7Eb4463524FEc27fbD");
pub const MKR_TOKEN_ADDRESS: &[u8] = &hex!("9f8F72aA9304c8B593d555F12eF6589cC3A579A2");
pub const SKY_TOKEN_ADDRESS: &[u8] = &hex!("56072C95FAA701256059aa122697B133aDEd9279");

// Add deployment transaction constants
pub const DAI_USDS_CONVERTER_DEPLOY_TX: &str =
    "0xb63d6f4cfb9945130ab32d914aaaafbad956be3718176771467b4154f9afab61";
pub const DAI_LITE_PSM_DEPLOY_TX: &str =
    "0x61e5d04f14d1fea9c505fb4dc9b6cf6e97bc83f2076b53cb7e92d0a2e88b6bbd";
pub const USDS_PSM_WRAPPER_DEPLOY_TX: &str =
    "0x43ddae74123936f6737b78fcf785547f7f6b7b27e280fe7fbf98c81b3c018585";
pub const SUSDS_DEPLOY_TX: &str =
    "0xe1be00c4ea3c21cf536b98ac082a5bba8485cf75d6b2b94f4d6e3edd06472c00";
pub const MKR_SKY_CONVERTER_DEPLOY_TX: &str =
    "0xbd89595dadba76ffb243cb446a355cfb833c1ea3cefbe427349f5b4644d5fa02";

#[substreams::handlers::map]
pub fn map_components(block: eth::v2::Block) -> Result<BlockTransactionProtocolComponents> {
    let mut tx_components = Vec::new();

    // Check for deployment transactions of our tracked contracts
    for tx in block.transactions() {
        let mut components = Vec::new();
        let tx_hash = hex::encode(&tx.hash);

        // Check DAI-USDS Converter
        if is_deployment_tx(tx, DAI_USDS_CONVERTER_ADDRESS) {
            components.push(
                ProtocolComponent::at_contract(DAI_USDS_CONVERTER_ADDRESS, &tx.into())
                    .with_tokens(&[DAI_TOKEN_ADDRESS, USDS_TOKEN_ADDRESS])
                    .with_creation_tx(&tx_hash)
                    .as_swap_type("dai_usds_converter", ImplementationType::Vm),
            );
        }

        // Check DAI Lite PSM
        if is_deployment_tx(tx, DAI_LITE_PSM_ADDRESS) {
            components.push(
                ProtocolComponent::at_contract(DAI_LITE_PSM_ADDRESS, &tx.into())
                    .with_tokens(&[DAI_TOKEN_ADDRESS, USDS_TOKEN_ADDRESS])
                    .with_creation_tx(&tx_hash)
                    .as_swap_type("dai_lite_psm", ImplementationType::Vm),
            );
        }

        // Check USDS PSM Wrapper
        if is_deployment_tx(tx, USDS_PSM_WRAPPER_ADDRESS) {
            components.push(
                ProtocolComponent::at_contract(USDS_PSM_WRAPPER_ADDRESS, &tx.into())
                    .with_tokens(&[USDS_TOKEN_ADDRESS, SUSDS_TOKEN_ADDRESS])
                    .with_creation_tx(&tx_hash)
                    .as_swap_type("usds_psm_wrapper", ImplementationType::Vm),
            );
        }

        // Check sUSD Staking
        if is_deployment_tx(tx, SUSDS_ADDRESS) {
            components.push(
                ProtocolComponent::at_contract(SUSDS_ADDRESS, &tx.into())
                    .with_tokens(&[USDS_TOKEN_ADDRESS, SUSDS_TOKEN_ADDRESS])
                    .with_creation_tx(&tx_hash)
                    .as_swap_type("susds_staking", ImplementationType::Vm),
            );
        }

        // Check MKR-SKY Converter
        if is_deployment_tx(tx, MKR_SKY_CONVERTER_ADDRESS) {
            components.push(
                ProtocolComponent::at_contract(MKR_SKY_CONVERTER_ADDRESS, &tx.into())
                    .with_tokens(&[MKR_TOKEN_ADDRESS, SKY_TOKEN_ADDRESS])
                    .with_creation_tx(&tx_hash)
                    .as_swap_type("mkr_sky_converter", ImplementationType::Vm),
            );
        }

        if !components.is_empty() {
            tx_components.push(TransactionProtocolComponents { tx: Some(tx.into()), components });
        }
    }

    Ok(BlockTransactionProtocolComponents { tx_components })
}
/*
If we have a component with:
- Contract address: 0x3225737a9Bbb6473CB4a45b7244ACa2BeFdB276A (DAI_USDS_CONVERTER)
- Component ID: 0x3225737a9Bbb6473CB4a45b7244ACa2BeFdB276A (same as address in this case)

The store will create an entry:
Key: "pool:0x3225737a9Bbb6473CB4a45b7244ACa2BeFdB276A"
Value: "0x3225737a9Bbb6473CB4a45b7244ACa2BeFdB276A"

The [..42] in the key format ensures we only use the contract address part
(0x + 40 hex chars = 42 chars) if the ID contains additional data.
*/
/// Simply stores the `ProtocolComponent`s with the pool address as the key and the pool id as value
#[substreams::handlers::store]
pub fn store_components(map: BlockTransactionProtocolComponents, store: StoreSetString) {
    map.tx_components
        .into_iter()
        .for_each(|tx_pc| {
            tx_pc
                .components
                .into_iter()
                .for_each(|pc| store.set(0, format!("pool:{0}", &pc.id[..42]), &pc.id))
        });
}

#[substreams::handlers::map]
pub fn map_relative_balances(
    block: eth::v2::Block,
    components_store: StoreGetString,
) -> Result<BlockBalanceDeltas> {
    let mut tx_balance_changes = Vec::new();

    // Process each transaction in the block
    for tx in block.transactions() {
        let mut balance_changes = Vec::new();

        // Process each log in the transaction
        for log in tx.logs() {
            // Check if this log is from a tracked component
            if let Some(component_id) =
                components_store.get_last(format!("pool:{}", hex::encode(&log.address)))
            {
                // Handle sDAI events
                if log.address == SDAI_VAULT_ADDRESS {
                    if let Some(event) = abi::sdai_contract::events::Deposit::match_and_decode(log)
                    {
                        balance_changes.extend(
                            extract_contract_changes_builder()
                                .component_id(&component_id)
                                .token(DAI_TOKEN_ADDRESS)
                                .add_from_user(&event.sender, event.assets)
                                .build(),
                        );
                    }
                    if let Some(event) = abi::sdai_contract::events::Withdraw::match_and_decode(log)
                    {
                        balance_changes.extend(
                            extract_contract_changes_builder()
                                .component_id(&component_id)
                                .token(DAI_TOKEN_ADDRESS)
                                .sub_from_user(&event.sender, event.assets)
                                .build(),
                        );
                    }
                }

                // Handle DAI-USDS Converter events
                if log.address == DAI_USDS_CONVERTER_ADDRESS {
                    if let Some(event) =
                        abi::dai_usds_converter_contract::events::DaiToUsds::match_and_decode(log)
                    {
                        balance_changes.extend(
                            extract_contract_changes_builder()
                                .component_id(&component_id)
                                .token(DAI_TOKEN_ADDRESS)
                                .add_from_user(&event.caller, event.wad)
                                .token(USDS_TOKEN_ADDRESS)
                                .sub_from_user(&event.caller, event.wad)
                                .build(),
                        );
                    }
                    if let Some(event) =
                        abi::dai_usds_converter_contract::events::UsdsToDai::match_and_decode(log)
                    {
                        balance_changes.extend(
                            extract_contract_changes_builder()
                                .component_id(&component_id)
                                .token(USDS_TOKEN_ADDRESS)
                                .add_from_user(&event.caller, event.wad)
                                .token(DAI_TOKEN_ADDRESS)
                                .sub_from_user(&event.caller, event.wad)
                                .build(),
                        );
                    }
                }

                // Handle MKR-SKY Converter events
                if log.address == MKR_SKY_CONVERTER_ADDRESS {
                    if let Some(event) =
                        abi::mkr_sky_converter_contract::events::MkrToSky::match_and_decode(log)
                    {
                        balance_changes.extend(
                            extract_contract_changes_builder()
                                .component_id(&component_id)
                                .token(MKR_TOKEN_ADDRESS)
                                .add_from_user(&event.caller, event.mkr_amt)
                                .token(SKY_TOKEN_ADDRESS)
                                .sub_from_user(&event.caller, event.sky_amt)
                                .build(),
                        );
                    }
                    if let Some(event) =
                        abi::mkr_sky_converter_contract::events::SkyToMkr::match_and_decode(log)
                    {
                        balance_changes.extend(
                            extract_contract_changes_builder()
                                .component_id(&component_id)
                                .token(SKY_TOKEN_ADDRESS)
                                .add_from_user(&event.caller, event.sky_amt)
                                .token(MKR_TOKEN_ADDRESS)
                                .sub_from_user(&event.caller, event.mkr_amt)
                                .build(),
                        );
                    }
                }

                // Handle DAI Lite PSM events
                if log.address == DAI_LITE_PSM_ADDRESS {
                    if let Some(event) =
                        abi::dai_lite_psm_contract::events::BuyGem::match_and_decode(log)
                    {
                        balance_changes.extend(
                            extract_contract_changes_builder()
                                .component_id(&component_id)
                                .token(DAI_TOKEN_ADDRESS)
                                .add_from_user(&event.owner, event.value)
                                .token(USDS_TOKEN_ADDRESS)
                                .sub_from_user(&event.owner, event.value.sub(event.fee))
                                .build(),
                        );
                    }
                    if let Some(event) =
                        abi::dai_lite_psm_contract::events::SellGem::match_and_decode(log)
                    {
                        balance_changes.extend(
                            extract_contract_changes_builder()
                                .component_id(&component_id)
                                .token(USDS_TOKEN_ADDRESS)
                                .add_from_user(&event.owner, event.value)
                                .token(DAI_TOKEN_ADDRESS)
                                .sub_from_user(&event.owner, event.value.sub(event.fee))
                                .build(),
                        );
                    }
                }

                // Handle USDS PSM Wrapper events
                if log.address == USDS_PSM_WRAPPER_ADDRESS {
                    if let Some(event) =
                        abi::usds_psm_wrapper_contract::events::BuyGem::match_and_decode(log)
                    {
                        balance_changes.extend(
                            extract_contract_changes_builder()
                                .component_id(&component_id)
                                .token(USDS_TOKEN_ADDRESS)
                                .add_from_user(&event.usr, event.gem_amt)
                                .token(SUSDS_TOKEN_ADDRESS)
                                .sub_from_user(&event.usr, event.usds_in_wad)
                                .build(),
                        );
                    }
                    if let Some(event) =
                        abi::usds_psm_wrapper_contract::events::SellGem::match_and_decode(log)
                    {
                        balance_changes.extend(
                            extract_contract_changes_builder()
                                .component_id(&component_id)
                                .token(SUSDS_TOKEN_ADDRESS)
                                .add_from_user(&event.usr, event.usds_out_wad)
                                .token(USDS_TOKEN_ADDRESS)
                                .sub_from_user(&event.usr, event.gem_amt)
                                .build(),
                        );
                    }
                }

                // Handle sUSDS events
                if log.address == SUSDS_ADDRESS {
                    if let Some(event) = abi::susds_contract::events::Deposit::match_and_decode(log)
                    {
                        balance_changes.extend(
                            extract_contract_changes_builder()
                                .component_id(&component_id)
                                .token(USDS_TOKEN_ADDRESS)
                                .add_from_user(&event.sender, event.assets)
                                .token(SUSDS_TOKEN_ADDRESS)
                                .sub_from_user(&event.owner, event.shares)
                                .build(),
                        );
                    }
                    if let Some(event) =
                        abi::susds_contract::events::Withdraw::match_and_decode(log)
                    {
                        balance_changes.extend(
                            extract_contract_changes_builder()
                                .component_id(&component_id)
                                .token(SUSDS_TOKEN_ADDRESS)
                                .add_from_user(&event.sender, event.shares)
                                .token(USDS_TOKEN_ADDRESS)
                                .sub_from_user(&event.receiver, event.assets)
                                .build(),
                        );
                    }
                }
            }
        }

        if !balance_changes.is_empty() {
            tx_balance_changes.push(TransactionBalanceDeltas {
                tx: Some(tx.into()),
                balance_changes: aggregate_balances_changes(balance_changes),
            });
        }
    }

    Ok(BlockBalanceDeltas { tx_balance_deltas: tx_balance_changes })
}

fn is_deployment_tx(tx: &eth::v2::TransactionTrace, contract_address: &[u8]) -> bool {
    let created_accounts = tx
        .calls
        .iter()
        .flat_map(|call| {
            call.account_creations
                .iter()
                .map(|ac| ac.account.to_owned())
        })
        .collect::<Vec<_>>();

    if let Some(deployed_address) = created_accounts.first() {
        return deployed_address.as_slice() == contract_address;
    }
    false
}
