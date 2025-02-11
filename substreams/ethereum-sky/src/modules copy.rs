use crate::{abi, addresses::SkyAddresses};
use anyhow::Result;
use itertools::Itertools;
use std::collections::HashMap;
use substreams::{
    hex,
    pb::substreams::StoreDeltas,
    scalar::BigInt,
    store::{
        StoreAdd, StoreAddBigInt, StoreGet, StoreGetString, StoreNew, StoreSet, StoreSetString,
    },
};
use substreams_ethereum::{pb::eth, Event};
use tycho_substreams::{
    balances::aggregate_balances_changes, contract::extract_contract_changes_builder, prelude::*,
};

fn is_deployment_tx(tx: &eth::v2::TransactionTrace, address: &[u8]) -> bool {
    let created_accounts = tx
        .calls
        .iter()
        .flat_map(|call| {
            call.account_creations
                .iter()
                .map(|ac| ac.account.to_owned())
        })
        .collect::<Vec<_>>();

    created_accounts
        .first()
        .map_or(false, |deployed| deployed.as_slice() == address)
}

#[substreams::handlers::map]
pub fn map_components(
    params: String,
    block: eth::v2::Block,
) -> Result<BlockTransactionProtocolComponents> {
    let addresses = SkyAddresses::from_params(&params)?;
    let mut tx_components = Vec::new();

    // Check for deployment transactions of our tracked contracts
    for tx in block.transactions() {
        let mut components = Vec::new();

        // Check each contract deployment
        if is_deployment_tx(tx, &addresses.sdai) {
            components.push(
                ProtocolComponent::at_contract(&addresses.sdai, &tx.into())
                    .with_tokens(&[&addresses.dai, &addresses.sdai])
                    .as_swap_type("sdai_vault", ImplementationType::Vm),
            );
        }

        // Check DAI-USDS Converter
        if is_deployment_tx(tx, &addresses.dai_usds_converter) {
            components.push(
                ProtocolComponent::at_contract(&addresses.dai_usds_converter, &tx.into())
                    .with_tokens(&[&addresses.dai, &addresses.usds])
                    .as_swap_type("dai_usds_converter", ImplementationType::Vm),
            );
        }

        // Check DAI Lite PSM
        if is_deployment_tx(tx, &addresses.dai_lite_psm) {
            components.push(
                ProtocolComponent::at_contract(&addresses.dai_lite_psm, &tx.into())
                    .with_tokens(&[&addresses.dai, &addresses.usds])
                    .as_swap_type("dai_lite_psm", ImplementationType::Vm),
            );
        }

        // Check USDS PSM Wrapper
        if is_deployment_tx(tx, &addresses.usds_psm_wrapper) {
            components.push(
                ProtocolComponent::at_contract(&addresses.usds_psm_wrapper, &tx.into())
                    .with_tokens(&[&addresses.usds, &addresses.susds])
                    .as_swap_type("usds_psm_wrapper", ImplementationType::Vm),
            );
        }

        // Check sUSD Staking
        if is_deployment_tx(tx, &addresses.susds) {
            components.push(
                ProtocolComponent::at_contract(&addresses.susds, &tx.into())
                    .with_tokens(&[&addresses.usds, &addresses.susds])
                    .with_state_contracts(&[
                        &addresses.usds_psm_wrapper,
                        &addresses.usds_psm_wrapper,
                    ])
                    .as_swap_type("susds_vault", ImplementationType::Vm),
            );
        }

        // Check MKR-SKY Converter
        if is_deployment_tx(tx, &addresses.mkr_sky_converter) {
            components.push(
                ProtocolComponent::at_contract(&addresses.mkr_sky_converter, &tx.into())
                    .with_tokens(&[&addresses.mkr, &addresses.sky])
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
    map.tx_components
        .into_iter()
        .for_each(|tx_pc| {
            tx_pc
                .components
                .into_iter()
                .for_each(|pc| {
                    let key = format!("pool:{}", &pc.id[..42]);
                    store.set(0, key, &pc.id);
                });
        });
}

#[substreams::handlers::map]
pub fn map_relative_balances(
    params: String,
    block: eth::v2::Block,
    store: StoreGetString,
) -> Result<BlockBalanceDeltas> {
    let addresses = SkyAddresses::from_params(&params)?;

    let balance_deltas = block
        .logs()
        .filter(|log| {
            let is_relevant = log.address() == addresses.dai_usds_converter.as_slice()
                || log.address() == addresses.dai_lite_psm.as_slice()
                || log.address() == addresses.usds_psm_wrapper.as_slice()
                || log.address() == addresses.susds.as_slice()
                || log.address() == addresses.mkr_sky_converter.as_slice();

            if is_relevant {
                substreams::log::info!(
                    "Found relevant log from address: 0x{}",
                    hex::encode(log.address())
                );
            }

            is_relevant
        })
        .flat_map(|vault_log| {
            let mut deltas = Vec::new();

            // DAI-USDS Converter Events
            if let Some(ev) =
                abi::dai_usds_converter_contract::events::DaiToUsds::match_and_decode(vault_log.log)
            {
                let component_id = format!("0x{}", hex::encode(&addresses.dai_usds_converter));
                if store
                    .get_last(format!("pool:{}", &component_id[..42]))
                    .is_some()
                {
                    deltas.extend_from_slice(&[
                        BalanceDelta {
                            ord: vault_log.ordinal(),
                            tx: Some(vault_log.receipt.transaction.into()),
                            token: addresses.dai.to_vec(),
                            delta: ev.dai_amount.to_signed_bytes_be(),
                            component_id: component_id.clone().as_bytes().to_vec(),
                        },
                        BalanceDelta {
                            ord: vault_log.ordinal(),
                            tx: Some(vault_log.receipt.transaction.into()),
                            token: addresses.usds.to_vec(),
                            delta: ev
                                .usds_amount
                                .neg()
                                .to_signed_bytes_be(),
                            component_id: component_id.clone().as_bytes().to_vec(),
                        },
                    ]);
                }
            } else if let Some(ev) =
                abi::dai_usds_converter_contract::events::UsdsToDai::match_and_decode(vault_log.log)
            {
                let component_id = format!("0x{}", hex::encode(&addresses.dai_usds_converter));
                if store
                    .get_last(format!("pool:{}", &component_id[..42]))
                    .is_some()
                {
                    deltas.extend_from_slice(&[
                        BalanceDelta {
                            ord: vault_log.ordinal(),
                            tx: Some(vault_log.receipt.transaction.into()),
                            token: addresses.usds.to_vec(),
                            delta: ev.usds_amount.to_signed_bytes_be(),
                            component_id: component_id.clone().as_bytes().to_vec(),
                        },
                        BalanceDelta {
                            ord: vault_log.ordinal(),
                            tx: Some(vault_log.receipt.transaction.into()),
                            token: addresses.dai.to_vec(),
                            delta: ev
                                .usds_amount
                                .neg()
                                .to_signed_bytes_be(),
                            component_id: component_id.clone().as_bytes().to_vec(),
                        },
                    ]);
                }
            } else if let Some(ev) =
                abi::dai_lite_psm_contract::events::BuyGem::match_and_decode(vault_log.log)
            {
                let (component_id, token_in, token_out) =
                    if vault_log.receipt.transaction.to == addresses.usds_psm_wrapper {
                        (
                            format!("0x{}", hex::encode(addresses.usds_psm_wrapper)),
                            addresses.usds,
                            addresses.usdc,
                        )
                    } else {
                        (
                            format!("0x{}", hex::encode(addresses.dai_lite_psm)),
                            addresses.dai,
                            addresses.usdc,
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
                    if vault_log.receipt.transaction.to == addresses.usds_psm_wrapper {
                        (
                            format!("0x{}", hex::encode(addresses.usds_psm_wrapper)),
                            addresses.usdc,
                            addresses.usds,
                        )
                    } else {
                        (
                            format!("0x{}", hex::encode(addresses.dai_lite_psm)),
                            addresses.usdc,
                            addresses.dai,
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
                let component_id = format!("0x{}", hex::encode(&addresses.susds));
                if store
                    .get_last(format!("pool:{}", &component_id[..42]))
                    .is_some()
                {
                    deltas.extend_from_slice(&[
                        BalanceDelta {
                            ord: vault_log.ordinal(),
                            tx: Some(vault_log.receipt.transaction.into()),
                            token: addresses.usds.to_vec(),
                            delta: ev.assets.to_signed_bytes_be(),
                            component_id: component_id.clone().as_bytes().to_vec(),
                        },
                        BalanceDelta {
                            ord: vault_log.ordinal(),
                            tx: Some(vault_log.receipt.transaction.into()),
                            token: addresses.susds.to_vec(),
                            delta: ev.shares.neg().to_signed_bytes_be(),
                            component_id: component_id.clone().as_bytes().to_vec(),
                        },
                    ]);
                }
            } else if let Some(ev) =
                abi::susds_contract::events::Withdraw::match_and_decode(vault_log.log)
            {
                let component_id = format!("0x{}", hex::encode(&addresses.susds));
                if store
                    .get_last(format!("pool:{}", &component_id[..42]))
                    .is_some()
                {
                    deltas.extend_from_slice(&[
                        BalanceDelta {
                            ord: vault_log.ordinal(),
                            tx: Some(vault_log.receipt.transaction.into()),
                            token: addresses.usds.to_vec(),
                            delta: ev.assets.neg().to_signed_bytes_be(),
                            component_id: component_id.clone().as_bytes().to_vec(),
                        },
                        BalanceDelta {
                            ord: vault_log.ordinal(),
                            tx: Some(vault_log.receipt.transaction.into()),
                            token: addresses.susds.to_vec(),
                            delta: ev.shares.to_signed_bytes_be(),
                            component_id: component_id.clone().as_bytes().to_vec(),
                        },
                    ]);
                }
            } else if let Some(ev) =
                abi::mkr_sky_converter_contract::events::MkrToSky::match_and_decode(vault_log.log)
            {
                let component_id = format!("0x{}", hex::encode(&addresses.mkr_sky_converter));
                if store
                    .get_last(format!("pool:{}", &component_id[..42]))
                    .is_some()
                {
                    deltas.extend_from_slice(&[
                        BalanceDelta {
                            ord: vault_log.ordinal(),
                            tx: Some(vault_log.receipt.transaction.into()),
                            token: addresses.mkr.to_vec(),
                            delta: ev.mkr_amt.to_signed_bytes_be(),
                            component_id: component_id.clone().as_bytes().to_vec(),
                        },
                        BalanceDelta {
                            ord: vault_log.ordinal(),
                            tx: Some(vault_log.receipt.transaction.into()),
                            token: addresses.sky.to_vec(),
                            delta: ev.sky_amt.neg().to_signed_bytes_be(),
                            component_id: component_id.clone().as_bytes().to_vec(),
                        },
                    ]);
                }
            } else if let Some(ev) =
                abi::mkr_sky_converter_contract::events::SkyToMkr::match_and_decode(vault_log.log)
            {
                let component_id = format!("0x{}", hex::encode(&addresses.mkr_sky_converter));
                if store
                    .get_last(format!("pool:{}", &component_id[..42]))
                    .is_some()
                {
                    deltas.extend_from_slice(&[
                        BalanceDelta {
                            ord: vault_log.ordinal(),
                            tx: Some(vault_log.receipt.transaction.into()),
                            token: addresses.sky.to_vec(),
                            delta: ev.sky_amt.to_signed_bytes_be(),
                            component_id: component_id.clone().as_bytes().to_vec(),
                        },
                        BalanceDelta {
                            ord: vault_log.ordinal(),
                            tx: Some(vault_log.receipt.transaction.into()),
                            token: addresses.mkr.to_vec(),
                            delta: ev.sky_amt.neg().to_signed_bytes_be(),
                            component_id: component_id.clone().as_bytes().to_vec(),
                        },
                    ]);
                }
            } else if let Some(ev) =
                abi::sdai_contract::events::Deposit::match_and_decode(vault_log.log)
            {
                let component_id = format!("0x{}", hex::encode(&addresses.sdai));
                if store
                    .get_last(format!("pool:{}", &component_id[..42]))
                    .is_some()
                {
                    deltas.extend_from_slice(&[
                        BalanceDelta {
                            ord: vault_log.ordinal(),
                            tx: Some(vault_log.receipt.transaction.into()),
                            token: addresses.dai.to_vec(),
                            delta: ev.assets.to_signed_bytes_be(),
                            component_id: component_id.clone().as_bytes().to_vec(),
                        },
                        BalanceDelta {
                            ord: vault_log.ordinal(),
                            tx: Some(vault_log.receipt.transaction.into()),
                            token: addresses.sdai.to_vec(),
                            delta: ev.shares.neg().to_signed_bytes_be(),
                            component_id: component_id.clone().as_bytes().to_vec(),
                        },
                    ]);
                }
            } else if let Some(ev) =
                abi::sdai_contract::events::Withdraw::match_and_decode(vault_log.log)
            {
                let component_id = format!("0x{}", hex::encode(&addresses.sdai));
                if store
                    .get_last(format!("pool:{}", &component_id[..42]))
                    .is_some()
                {
                    deltas.extend_from_slice(&[
                        BalanceDelta {
                            ord: vault_log.ordinal(),
                            tx: Some(vault_log.receipt.transaction.into()),
                            token: addresses.dai.to_vec(),
                            delta: ev.assets.neg().to_signed_bytes_be(),
                            component_id: component_id.clone().as_bytes().to_vec(),
                        },
                        BalanceDelta {
                            ord: vault_log.ordinal(),
                            tx: Some(vault_log.receipt.transaction.into()),
                            token: addresses.sdai.to_vec(),
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
pub fn store_balances(
    params: String,
    block: eth::v2::Block,
    deltas: BlockBalanceDeltas,
    store: StoreAddBigInt,
) -> Result<()> {
    let addresses = SkyAddresses::from_params(&params)?;

    for delta in deltas.balance_deltas {
        let component_id = hex::encode(&delta.component_id);
        let start_block = match component_id.as_str() {
            id if id == hex::encode(addresses.sdai) => 16_932_340,
            id if id == hex::encode(addresses.dai_usds_converter) => 20_770_195,
            id if id == hex::encode(addresses.dai_lite_psm) => 20_535_921,
            id if id == hex::encode(addresses.usds_psm_wrapper) => 20_791_763,
            id if id == hex::encode(addresses.susds) => 20_771_188,
            id if id == hex::encode(addresses.mkr_sky_converter) => 20_770_588,
            _ => 0,
        };

        if block.number >= start_block {
            let key = format!(
                "balance:{}:{}:{}",
                hex::encode(&delta.token),
                hex::encode(&delta.component_id),
                block.number
            );
            store.add(delta.ord, key, BigInt::from_signed_bytes_be(&delta.delta));

            substreams::log::info!(
                "Storing balance at block {} - Token: 0x{}, Component: {}, Delta: {}",
                block.number,
                hex::encode(&delta.token),
                component_id,
                hex::encode(&delta.delta)
            );
        }
    }
    Ok(())
}

fn format_component_id(address: &[u8]) -> String {
    format!("0x{}", hex::encode(address))
}

fn format_pool_key(component_id: &str) -> String {
    format!("pool:{}", &component_id[..42])
}

#[substreams::handlers::map]
pub fn map_protocol_changes(
    params: String,
    block: eth::v2::Block,
    grouped_components: BlockTransactionProtocolComponents,
    deltas: BlockBalanceDeltas,
    components_store: StoreGetString,
    balance_store: StoreDeltas,
) -> Result<BlockChanges> {
    let addresses = SkyAddresses::from_params(&params)?;
    let mut transaction_changes: HashMap<_, TransactionChangesBuilder> = HashMap::new();

    // Process components
    grouped_components
        .tx_components
        .iter()
        .for_each(|tx_component| {
            let tx = tx_component
                .tx
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("Missing transaction"))
                .unwrap();

            let builder = transaction_changes
                .entry(tx.index)
                .or_insert_with(|| TransactionChangesBuilder::new(tx));

            tx_component
                .components
                .iter()
                .for_each(|component| {
                    let default_attributes = vec![
                        Attribute {
                            name: "balance_owner".to_string(),
                            value: hex::decode(&component.id[2..42])
                                .expect("Invalid component ID hex"),
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
                    token_bc_map.values().for_each(|bc| {
                        builder.add_balance_change(bc);
                    });
                });
        });

    // Extract contract changes
    extract_contract_changes_builder(
        &block,
        |addr| components_store
            .get_last(format!("pool:0x{0}", hex::encode(addr)))
            .is_some(),
        &mut transaction_changes,
    )?;

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
                        .ok_or_else(|| anyhow::anyhow!("Missing component ID"))
                        .unwrap();
                    change.mark_component_as_updated(&id);
                });
        });

    // Sort and build final changes
    Ok(BlockChanges {
        block: Some((&block).into()),
        changes: transaction_changes
            .drain()
            .sorted_unstable_by_key(|(index, _)| *index)
            .filter_map(|(_index, builder)| -> Option<TransactionChanges> {
                match builder.build() {
                    Some(changes) if !changes.is_empty() => Some(changes),
                    _ => None,
                }
            })
            .collect::<Vec<_>>(),
    })
}
