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

/*
// Add deployment transaction constants
pub const DAI_USDS_CONVERTER_DEPLOY_TX: &str =
    "b63d6f4cfb9945130ab32d914aaaafbad956be3718176771467b4154f9afab61";
pub const DAI_LITE_PSM_DEPLOY_TX: &str =
    "61e5d04f14d1fea9c505fb4dc9b6cf6e97bc83f2076b53cb7e92d0a2e88b6bbd";
pub const USDS_PSM_WRAPPER_DEPLOY_TX: &str =
    "43ddae74123936f6737b78fcf785547f7f6b7b27e280fe7fbf98c81b3c018585";
pub const SUSDS_DEPLOY_TX: &str =
    "e1be00c4ea3c21cf536b98ac082a5bba8485cf75d6b2b94f4d6e3edd06472c00";
pub const MKR_SKY_CONVERTER_DEPLOY_TX: &str =
    "bd89595dadba76ffb243cb446a355cfb833c1ea3cefbe427349f5b4644d5fa02";
*/

#[substreams::handlers::map]
pub fn map_components(block: eth::v2::Block) -> Result<BlockTransactionProtocolComponents> {
    let mut tx_components = Vec::new();

    // Check for deployment transactions of our tracked contracts
    for tx in block.transactions() {
        let mut components = Vec::new();

        // Check each contract deployment
        if is_deployment_tx(tx, SDAI_VAULT_ADDRESS) {
            substreams::log::info!("Found sDAI Vault deployment tx: {}", hex::encode(&tx.hash));
            components.push(
                ProtocolComponent::at_contract(SDAI_VAULT_ADDRESS, &tx.into())
                    .with_tokens(&[DAI_TOKEN_ADDRESS, SDAI_VAULT_ADDRESS])
                    .as_swap_type("sdai_vault", ImplementationType::Vm),
            );
        }

        // Check DAI-USDS Converter
        if is_deployment_tx(tx, DAI_USDS_CONVERTER_ADDRESS) {
            substreams::log::info!("Found DAI-USDS Converter deployment tx: {}", hex::encode(&tx.hash));
            components.push(
                ProtocolComponent::at_contract(DAI_USDS_CONVERTER_ADDRESS, &tx.into())
                    .with_tokens(&[DAI_TOKEN_ADDRESS, USDS_TOKEN_ADDRESS])
                    .as_swap_type("dai_usds_converter", ImplementationType::Vm),
            );
        }

        // Check DAI Lite PSM
        if is_deployment_tx(tx, DAI_LITE_PSM_ADDRESS) {
            substreams::log::info!("Found DAI Lite PSM deployment tx: {}", hex::encode(&tx.hash));
            components.push(
                ProtocolComponent::at_contract(DAI_LITE_PSM_ADDRESS, &tx.into())
                    .with_tokens(&[DAI_TOKEN_ADDRESS, USDS_TOKEN_ADDRESS])
                    .as_swap_type("dai_lite_psm", ImplementationType::Vm),
            );
        }

        // Check USDS PSM Wrapper
        if is_deployment_tx(tx, USDS_PSM_WRAPPER_ADDRESS) {
            substreams::log::info!("Found USDS PSM Wrapper deployment tx: {}", hex::encode(&tx.hash));
            components.push(
                ProtocolComponent::at_contract(USDS_PSM_WRAPPER_ADDRESS, &tx.into())
                    .with_tokens(&[USDS_TOKEN_ADDRESS, SUSDS_TOKEN_ADDRESS])
                    .as_swap_type("usds_psm_wrapper", ImplementationType::Vm),
            );
        }

        // Check sUSD Staking
        if is_deployment_tx(tx, SUSDS_ADDRESS) {
            substreams::log::info!("Found sUSD Staking deployment tx: {}", hex::encode(&tx.hash));
            components.push(
                ProtocolComponent::at_contract(SUSDS_ADDRESS, &tx.into())
                    .with_tokens(&[USDS_TOKEN_ADDRESS, SUSDS_TOKEN_ADDRESS])
                    .as_swap_type("susds_vault", ImplementationType::Vm),
            );
        }

        // Check MKR-SKY Converter
        if is_deployment_tx(tx, MKR_SKY_CONVERTER_ADDRESS) {
            substreams::log::info!("Found MKR-SKY Converter deployment tx: {}", hex::encode(&tx.hash));
            components.push(
                ProtocolComponent::at_contract(MKR_SKY_CONVERTER_ADDRESS, &tx.into())
                    .with_tokens(&[MKR_TOKEN_ADDRESS, SKY_TOKEN_ADDRESS])
                    .as_swap_type("mkr_sky_converter", ImplementationType::Vm),
            );
        }

        if !components.is_empty() {
            tx_components.push(TransactionProtocolComponents { tx: Some(tx.into()), components });
        }
    }

    Ok(BlockTransactionProtocolComponents { tx_components })
}

#[substreams::handlers::store]
pub fn store_components(map: BlockTransactionProtocolComponents, store: StoreSetString) {
    substreams::log::info!("Processing {} transactions with components", map.tx_components.len());

    map.tx_components
        .into_iter()
        .for_each(|tx_pc| {
            substreams::log::info!("Transaction has {} components", tx_pc.components.len());

            tx_pc
                .components
                .into_iter()
                .for_each(|pc| {
                    let key = format!("pool:{}", &pc.id[..42]);
                    substreams::log::info!(
                        "Storing component with key: {}, value: {}",
                        key,
                        &pc.id
                    );
                    store.set(0, key, &pc.id);
                });
        });
}

#[substreams::handlers::map]
pub fn map_relative_balances(
    block: eth::v2::Block,
    store: StoreGetString,
) -> Result<BlockBalanceDeltas> {
    let balance_deltas = block
        .logs()
        .filter(|log| {
            log.address() == DAI_USDS_CONVERTER_ADDRESS
                || log.address() == DAI_LITE_PSM_ADDRESS
                || log.address() == USDS_PSM_WRAPPER_ADDRESS
                || log.address() == SUSDS_ADDRESS
                || log.address() == MKR_SKY_CONVERTER_ADDRESS
        })
        .flat_map(|vault_log| {
            let mut deltas = Vec::new();

            // 1. DAI-USDS Converter Events
            if let Some(ev) =
                abi::dai_usds_converter_contract::events::DaiToUsds::match_and_decode(vault_log.log)
            {
                let component_id = format!("0x{}", hex::encode(DAI_USDS_CONVERTER_ADDRESS));
                if store
                    .get_last(format!("pool:{}", &component_id[..42]))
                    .is_some()
                {
                    deltas.extend_from_slice(&[
                        BalanceDelta {
                            ord: vault_log.ordinal(),
                            tx: Some(vault_log.receipt.transaction.into()),
                            token: DAI_TOKEN_ADDRESS.to_vec(),
                            delta: ev.wad.to_signed_bytes_be(),
                            component_id: component_id.clone().as_bytes().to_vec(),
                        },
                        BalanceDelta {
                            ord: vault_log.ordinal(),
                            tx: Some(vault_log.receipt.transaction.into()),
                            token: USDS_TOKEN_ADDRESS.to_vec(),
                            delta: ev.wad.neg().to_signed_bytes_be(),
                            component_id: component_id.clone().as_bytes().to_vec(),
                        },
                    ]);
                }
            } else if let Some(ev) =
                abi::dai_usds_converter_contract::events::UsdsToDai::match_and_decode(vault_log.log)
            {
                let component_id = format!("0x{}", hex::encode(DAI_USDS_CONVERTER_ADDRESS));
                if store
                    .get_last(format!("pool:{}", &component_id[..42]))
                    .is_some()
                {
                    deltas.extend_from_slice(&[
                        BalanceDelta {
                            ord: vault_log.ordinal(),
                            tx: Some(vault_log.receipt.transaction.into()),
                            token: USDS_TOKEN_ADDRESS.to_vec(),
                            delta: ev.wad.to_signed_bytes_be(),
                            component_id: component_id.clone().as_bytes().to_vec(),
                        },
                        BalanceDelta {
                            ord: vault_log.ordinal(),
                            tx: Some(vault_log.receipt.transaction.into()),
                            token: DAI_TOKEN_ADDRESS.to_vec(),
                            delta: ev.wad.neg().to_signed_bytes_be(),
                            component_id: component_id.clone().as_bytes().to_vec(),
                        },
                    ]);
                }
            } else if let Some(ev) =
                abi::dai_lite_psm_contract::events::BuyGem::match_and_decode(vault_log.log)
            {
                let (component_id, token_in, token_out) =
                    if vault_log.receipt.transaction.to == USDS_PSM_WRAPPER_ADDRESS {
                        (
                            format!("0x{}", hex::encode(USDS_PSM_WRAPPER_ADDRESS)),
                            USDS_TOKEN_ADDRESS, // USDS is spent
                            USDC_TOKEN_ADDRESS, // USDC is received
                        )
                    } else {
                        (
                            format!("0x{}", hex::encode(DAI_LITE_PSM_ADDRESS)),
                            DAI_TOKEN_ADDRESS,  // DAI is spent
                            USDC_TOKEN_ADDRESS, // USDC is received
                        )
                    };

                if store
                    .get_last(format!("pool:{}", &component_id[..42]))
                    .is_some()
                {
                    deltas.extend_from_slice(&[
                        BalanceDelta {
                            ord: vault_log.ordinal(),
                            tx: Some(vault_log.receipt.transaction.into()),
                            token: token_in.to_vec(),
                            delta: ev.value.to_signed_bytes_be(),
                            component_id: component_id.clone().as_bytes().to_vec(),
                        },
                        BalanceDelta {
                            ord: vault_log.ordinal(),
                            tx: Some(vault_log.receipt.transaction.into()),
                            token: token_out.to_vec(),
                            delta: ev.value.neg().to_signed_bytes_be(),
                            component_id: component_id.clone().as_bytes().to_vec(),
                        },
                    ]);
                }
            } else if let Some(ev) =
                abi::dai_lite_psm_contract::events::SellGem::match_and_decode(vault_log.log)
            {
                let (component_id, token_in, token_out) =
                    if vault_log.receipt.transaction.to == USDS_PSM_WRAPPER_ADDRESS {
                        (
                            format!("0x{}", hex::encode(USDS_PSM_WRAPPER_ADDRESS)),
                            USDC_TOKEN_ADDRESS, // USDC is spent
                            USDS_TOKEN_ADDRESS, // USDS is received
                        )
                    } else {
                        (
                            format!("0x{}", hex::encode(DAI_LITE_PSM_ADDRESS)),
                            USDC_TOKEN_ADDRESS, // USDC is spent
                            DAI_TOKEN_ADDRESS,  // DAI is received
                        )
                    };

                if store
                    .get_last(format!("pool:{}", &component_id[..42]))
                    .is_some()
                {
                    deltas.extend_from_slice(&[
                        BalanceDelta {
                            ord: vault_log.ordinal(),
                            tx: Some(vault_log.receipt.transaction.into()),
                            token: token_in.to_vec(),
                            delta: ev.value.to_signed_bytes_be(),
                            component_id: component_id.clone().as_bytes().to_vec(),
                        },
                        BalanceDelta {
                            ord: vault_log.ordinal(),
                            tx: Some(vault_log.receipt.transaction.into()),
                            token: token_out.to_vec(),
                            delta: ev.value.neg().to_signed_bytes_be(),
                            component_id: component_id.clone().as_bytes().to_vec(),
                        },
                    ]);
                }
            } else if let Some(ev) =
                abi::susds_contract::events::Deposit::match_and_decode(vault_log.log)
            {
                let component_id = format!("0x{}", hex::encode(SUSDS_ADDRESS));
                if store
                    .get_last(format!("pool:{}", &component_id[..42]))
                    .is_some()
                {
                    deltas.extend_from_slice(&[
                        BalanceDelta {
                            ord: vault_log.ordinal(),
                            tx: Some(vault_log.receipt.transaction.into()),
                            token: USDS_TOKEN_ADDRESS.to_vec(),
                            delta: ev.assets.to_signed_bytes_be(),
                            component_id: component_id.clone().as_bytes().to_vec(),
                        },
                        BalanceDelta {
                            ord: vault_log.ordinal(),
                            tx: Some(vault_log.receipt.transaction.into()),
                            token: SUSDS_TOKEN_ADDRESS.to_vec(),
                            delta: ev.shares.neg().to_signed_bytes_be(),
                            component_id: component_id.clone().as_bytes().to_vec(),
                        },
                    ]);
                }
            } else if let Some(ev) =
                abi::susds_contract::events::Withdraw::match_and_decode(vault_log.log)
            {
                let component_id = format!("0x{}", hex::encode(SUSDS_ADDRESS));
                if store
                    .get_last(format!("pool:{}", &component_id[..42]))
                    .is_some()
                {
                    deltas.extend_from_slice(&[
                        BalanceDelta {
                            ord: vault_log.ordinal(),
                            tx: Some(vault_log.receipt.transaction.into()),
                            token: USDS_TOKEN_ADDRESS.to_vec(),
                            delta: ev.assets.neg().to_signed_bytes_be(),
                            component_id: component_id.clone().as_bytes().to_vec(),
                        },
                        BalanceDelta {
                            ord: vault_log.ordinal(),
                            tx: Some(vault_log.receipt.transaction.into()),
                            token: SUSDS_TOKEN_ADDRESS.to_vec(),
                            delta: ev.shares.to_signed_bytes_be(),
                            component_id: component_id.clone().as_bytes().to_vec(),
                        },
                    ]);
                }
            } else if let Some(ev) =
                abi::mkr_sky_converter_contract::events::MkrToSky::match_and_decode(vault_log.log)
            {
                let component_id = format!("0x{}", hex::encode(MKR_SKY_CONVERTER_ADDRESS));
                if store
                    .get_last(format!("pool:{}", &component_id[..42]))
                    .is_some()
                {
                    deltas.extend_from_slice(&[
                        BalanceDelta {
                            ord: vault_log.ordinal(),
                            tx: Some(vault_log.receipt.transaction.into()),
                            token: MKR_TOKEN_ADDRESS.to_vec(),
                            delta: ev.mkr_amt.to_signed_bytes_be(),
                            component_id: component_id.clone().as_bytes().to_vec(),
                        },
                        BalanceDelta {
                            ord: vault_log.ordinal(),
                            tx: Some(vault_log.receipt.transaction.into()),
                            token: SKY_TOKEN_ADDRESS.to_vec(),
                            delta: ev.sky_amt.neg().to_signed_bytes_be(),
                            component_id: component_id.clone().as_bytes().to_vec(),
                        },
                    ]);
                }
            } else if let Some(ev) =
                abi::mkr_sky_converter_contract::events::SkyToMkr::match_and_decode(vault_log.log)
            {
                let component_id = format!("0x{}", hex::encode(MKR_SKY_CONVERTER_ADDRESS));
                if store
                    .get_last(format!("pool:{}", &component_id[..42]))
                    .is_some()
                {
                    deltas.extend_from_slice(&[
                        BalanceDelta {
                            ord: vault_log.ordinal(),
                            tx: Some(vault_log.receipt.transaction.into()),
                            token: SKY_TOKEN_ADDRESS.to_vec(),
                            delta: ev.sky_amt.to_signed_bytes_be(),
                            component_id: component_id.clone().as_bytes().to_vec(),
                        },
                        BalanceDelta {
                            ord: vault_log.ordinal(),
                            tx: Some(vault_log.receipt.transaction.into()),
                            token: MKR_TOKEN_ADDRESS.to_vec(),
                            delta: ev.mkr_amt.neg().to_signed_bytes_be(),
                            component_id: component_id.clone().as_bytes().to_vec(),
                        },
                    ]);
                }
            } else if let Some(ev) =
                abi::sdai_contract::events::Deposit::match_and_decode(vault_log.log)
            {
                let component_id = format!("0x{}", hex::encode(SDAI_VAULT_ADDRESS));
                if store
                    .get_last(format!("pool:{}", &component_id[..42]))
                    .is_some()
                {
                    deltas.extend_from_slice(&[
                        BalanceDelta {
                            ord: vault_log.ordinal(),
                            tx: Some(vault_log.receipt.transaction.into()),
                            token: DAI_TOKEN_ADDRESS.to_vec(),
                            delta: ev.assets.to_signed_bytes_be(),
                            component_id: component_id.clone().as_bytes().to_vec(),
                        },
                        BalanceDelta {
                            ord: vault_log.ordinal(),
                            tx: Some(vault_log.receipt.transaction.into()),
                            token: SDAI_VAULT_ADDRESS.to_vec(),
                            delta: ev.shares.neg().to_signed_bytes_be(),
                            component_id: component_id.clone().as_bytes().to_vec(),
                        },
                    ]);
                }
            } else if let Some(ev) =
                abi::sdai_contract::events::Withdraw::match_and_decode(vault_log.log)
            {
                let component_id = format!("0x{}", hex::encode(SDAI_VAULT_ADDRESS));
                if store
                    .get_last(format!("pool:{}", &component_id[..42]))
                    .is_some()
                {
                    deltas.extend_from_slice(&[
                        BalanceDelta {
                            ord: vault_log.ordinal(),
                            tx: Some(vault_log.receipt.transaction.into()),
                            token: DAI_TOKEN_ADDRESS.to_vec(),
                            delta: ev.assets.neg().to_signed_bytes_be(),
                            component_id: component_id.clone().as_bytes().to_vec(),
                        },
                        BalanceDelta {
                            ord: vault_log.ordinal(),
                            tx: Some(vault_log.receipt.transaction.into()),
                            token: SDAI_VAULT_ADDRESS.to_vec(),
                            delta: ev.shares.to_signed_bytes_be(),
                            component_id: component_id.clone().as_bytes().to_vec(),
                        },
                    ]);
                }
            }

            deltas
        })
        .collect::<Vec<_>>();

    Ok(BlockBalanceDeltas { balance_deltas })
}

/// Store balances for each token in each component
#[substreams::handlers::store]
pub fn store_balances(deltas: BlockBalanceDeltas, store: StoreAddBigInt) {
    tycho_substreams::balances::store_balance_changes(deltas, store);
}

#[substreams::handlers::map]
pub fn map_protocol_changes(
    block: eth::v2::Block,
    grouped_components: BlockTransactionProtocolComponents,
    deltas: BlockBalanceDeltas,
    components_store: StoreGetString,
    balance_store: StoreDeltas,
) -> Result<BlockChanges> {
    let mut transaction_changes: HashMap<_, TransactionChangesBuilder> = HashMap::new();

    // Process components
    grouped_components
        .tx_components
        .iter()
        .for_each(|tx_component| {
            let tx = tx_component.tx.as_ref().unwrap();
            let builder = transaction_changes
                .entry(tx.index)
                .or_insert_with(|| TransactionChangesBuilder::new(tx));

            tx_component
                .components
                .iter()
                .for_each(|component| {
                    // Each component is its own balance owner
                    let default_attributes = vec![
                        Attribute {
                            name: "balance_owner".to_string(),
                            value: hex::decode(&component.id[2..42]).unwrap(), // Use component's own address
                            change: ChangeType::Creation.into(),
                        },
                        Attribute {
                            name: "update_marker".to_string(),
                            value: vec![1u8],
                            change: ChangeType::Creation.into(),
                        },
                    ];

                    builder.add_protocol_component(component);
                    let entity_change = EntityChanges {
                        component_id: component.id.clone(),
                        attributes: default_attributes,
                    };
                    builder.add_entity_change(&entity_change)
                });
        });

    // Process balance changes
    aggregate_balances_changes(balance_store, deltas)
        .into_iter()
        .for_each(|(_, (tx, balances))| {
            let builder = transaction_changes
                .entry(tx.index)
                .or_insert_with(|| TransactionChangesBuilder::new(&tx));
            balances
                .values()
                .for_each(|token_bc_map| {
                    token_bc_map
                        .values()
                        .for_each(|bc| builder.add_balance_change(bc))
                });
        });

    // Extract contract changes
    extract_contract_changes_builder(
        &block,
        |addr| {
            components_store
                .get_last(format!("pool:0x{0}", hex::encode(addr)))
                .is_some()
        },
        &mut transaction_changes,
    );

    // Mark updated components
    transaction_changes
        .iter_mut()
        .for_each(|(_, change)| {
            let addresses = change
                .changed_contracts()
                .map(|e| e.to_vec())
                .collect::<Vec<_>>();
            addresses
                .into_iter()
                .for_each(|address| {
                    let id = components_store
                        .get_last(format!("pool:0x{}", hex::encode(address)))
                        .unwrap();
                    change.mark_component_as_updated(&id);
                })
        });

    // Sort and build final changes
    Ok(BlockChanges {
        block: Some((&block).into()),
        changes: transaction_changes
            .drain()
            .sorted_unstable_by_key(|(index, _)| *index)
            .filter_map(|(_, builder)| builder.build())
            .collect::<Vec<_>>(),
    })
}

fn is_deployment_tx(tx: &eth::v2::TransactionTrace, contract_address: &[u8]) -> bool {
    match contract_address {
        SDAI_VAULT_ADDRESS => {
            tx.hash == hex!("a2f51048265f2fe9ffaf69b94cb5a2a4113be49bdecd2040d530dd6f68facc42")
        }

        DAI_USDS_CONVERTER_ADDRESS => {
            tx.hash == hex!("b63d6f4cfb9945130ab32d914aaaafbad956be3718176771467b4154f9afab61")
        }
        DAI_LITE_PSM_ADDRESS => {
            tx.hash == hex!("61e5d04f14d1fea9c505fb4dc9b6cf6e97bc83f2076b53cb7e92d0a2e88b6bbd")
        }
        USDS_PSM_WRAPPER_ADDRESS => {
            tx.hash == hex!("43ddae74123936f6737b78fcf785547f7f6b7b27e280fe7fbf98c81b3c018585")
        }
        SUSDS_ADDRESS => {
            tx.hash == hex!("e1be00c4ea3c21cf536b98ac082a5bba8485cf75d6b2b94f4d6e3edd06472c00")
        }
        MKR_SKY_CONVERTER_ADDRESS => {
            tx.hash == hex!("bd89595dadba76ffb243cb446a355cfb833c1ea3cefbe427349f5b4644d5fa02")
        }
        _ => false,
    }
}
