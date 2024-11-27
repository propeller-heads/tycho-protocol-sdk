use crate::abi::restake_manager_contract;
use anyhow::Result;
use std::collections::HashMap;
use substreams::{
    hex,
    pb::substreams::StoreDeltas,
    scalar::BigInt as ScalarBigInt,
    store::{StoreAdd, StoreAddBigInt, StoreAddInt64, StoreGet, StoreGetInt64, StoreNew},
};
use substreams_ethereum::{
    pb::eth::{self},
    Event,
};
use tycho_substreams::{
    balances::aggregate_balances_changes, contract::extract_contract_changes, prelude::*,
};

// hex!("ae7ab96520DE3A18E5e111B5EaAb095312D7fE84") stETH
// hex!("a2E3356610840701BDf5611a53974510Ae27E2e1") wBETH
// hex!("0000000000000000000000000000000000000000") ETH
// hex!("bf5495Efe5DB9ce00f80364C8B423567e58d2110") ezETH

/// Ethereum native token address representation
pub const ETH_ADDRESS: [u8; 20] = hex!("0000000000000000000000000000000000000000");

#[substreams::handlers::map]
pub fn map_components(
    params: String,
    block: eth::v2::Block,
) -> Result<BlockTransactionProtocolComponents, anyhow::Error> {
    let component_address =
        hex::decode(params).map_err(|e| anyhow::anyhow!("Failed to decode params: {}", e))?;

    let component_token = find_deployed_underlying_address(&component_address)
        .ok_or_else(|| anyhow::anyhow!("Failed to find deployed underlying address"))?;

    // We store these as a hashmap by tx hash since we need to agg by tx hash later
    Ok(BlockTransactionProtocolComponents {
        tx_components: block
            .transactions()
            .filter_map(|tx| {
                let components = tx
                    .calls()
                    .filter(|call| !call.call.state_reverted)
                    .filter_map(|call| {
                        // address doesn't exist before contract deployment, hence the first tx with
                        // a log.address = component_address is the deployment tx
                        if is_deployment_call(call.call, &component_address) {
                            Some(
                                ProtocolComponent::at_contract(&component_address, &tx.into())
                                    .with_tokens(&[
                                        component_token.as_slice(),
                                        ETH_ADDRESS.as_slice(),
                                    ])
                                    .as_swap_type("restake_manager", ImplementationType::Vm),
                            )
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();

                if !components.is_empty() {
                    Some(TransactionProtocolComponents { tx: Some(tx.into()), components })
                } else {
                    None
                }
            })
            .collect::<Vec<_>>(),
    })
}

/// Store protocol components and associated tokens
#[substreams::handlers::store]
pub fn store_components(map: BlockTransactionProtocolComponents, store: StoreAddInt64) {
    let components: Vec<String> = map
        .tx_components
        .iter()
        .flat_map(|tx_components| &tx_components.components)
        .map(|component| format!("pool:{0}", component.id))
        .collect();

    if !components.is_empty() {
        store.add_many(
            map.tx_components
                .first()
                .and_then(|tx| tx.tx.as_ref())
                .map(|tx| tx.index)
                .unwrap_or(0),
            &components,
            1,
        );
    }
}

/// Map balance changes caused by deposit and withdrawal events
#[substreams::handlers::map]
pub fn map_relative_balances(
    block: eth::v2::Block,
    store: StoreGetInt64,
) -> Result<BlockBalanceDeltas, anyhow::Error> {
    let balance_deltas = block
        .logs()
        .filter(|log| log.address() == VAULT_ADDRESS) // TODO: use deployed restake manager address
        .flat_map(|log| {
            let mut deltas = Vec::new();

            // Try to decode as deposit
            if let Some(ev) = restake_manager_contract::events::Deposit::match_and_decode(&log) {
                let address_hex = format!("0x{}", hex::encode(&log.address));
                // substreams::log::info!(
                //     "Found deposit event: address={}, token={}, amount={}, ez_eth_minted={}",
                //     address_hex,
                //     hex::encode(&ev.token),
                //     ev.amount,
                //     ev.ez_eth_minted
                // );

                // Check if this is a tracked component
                let store_key = format!("pool:{0}", address_hex);
                let is_tracked = store.get_last(&store_key).is_some();
                // substreams::log::debug!("Component tracked status for {}: {}", store_key,
                // is_tracked);

                if store.get_last(&store_key).is_some() {
                    let ez_eth_address = hex!("bf5495Efe5DB9ce00f80364C8B423567e58d2110").to_vec();
                    balance_deltas.extend_from_slice(&[
                        BalanceDelta {
                            ord: log.log.ordinal,
                            tx: Some(log.receipt.transaction.into()),
                            token: ev.token.to_vec(),
                            delta: ev.amount.to_signed_bytes_be(),
                            component_id: ev.token.to_vec(),
                        },
                        BalanceDelta {
                            ord: log.log.ordinal,
                            tx: Some(log.receipt.transaction.into()),
                            token: ez_eth_address.clone(),
                            delta: ev.ez_eth_minted.to_signed_bytes_be(),
                            component_id: ez_eth_address,
                        },
                    ]);
                }
            } else if let Some(ev) =
                restake_manager_contract::events::UserWithdrawCompleted::match_and_decode(&log)
            {
                let address_hex = format!("0x{}", hex::encode(log.log.address.clone()));

                substreams::log::info!(
                    "Found withdrawal event: address={}, token={}, amount={}, ez_eth_burned={}",
                    address_hex,
                    hex::encode(&ev.token),
                    ev.amount,
                    ev.ez_eth_burned
                );

                if let Some(_component_key) = store.get_last(format!("pool:{0}", address_hex)) {
                    let ez_eth_address = hex!("bf5495Efe5DB9ce00f80364C8B423567e58d2110").to_vec();

                    // Log withdrawal delta creation
                    substreams::log::info!(
                        "Creating withdrawal delta: token={}, amount=-{}, component_id={}",
                        hex::encode(&ev.token),
                        ev.amount,
                        hex::encode(&ev.token)
                    );

                    // Handle the balance delta for withdrawn token
                    balance_deltas.push(BalanceDelta {
                        ord: log.log.ordinal,
                        tx: Some(log.receipt.transaction.into()),
                        token: ev.token.clone(),
                        delta: ev.amount.neg().to_signed_bytes_be(),
                        component_id: ev.token.to_vec(),
                    });

                    // Log ezETH burning delta creation
                    substreams::log::info!(
                        "Creating ezETH burn delta: token={}, amount=-{}, component_id={}",
                        hex::encode(&ez_eth_address),
                        ev.ez_eth_burned,
                        hex::encode(&ez_eth_address)
                    );

                    // Handle the balance delta for burned ezETH
                    balance_deltas.push(BalanceDelta {
                        ord: log.ordinal(),
                        tx: Some(log.receipt.transaction.into()),
                        token: ez_eth_address.clone(),
                        delta: ev
                            .ez_eth_burned
                            .neg()
                            .to_signed_bytes_be(),
                        component_id: ez_eth_address,
                    });
                }
            }

            deltas
        })
        .collect::<Vec<_>>();

    // Log summary of processed deltas
    substreams::log::info!(
        "Block {} processing complete. Created {} balance deltas",
        block.number,
        balance_deltas.len()
    );

    Ok(BlockBalanceDeltas { balance_deltas })
}

/// Store aggregated balance changes
#[substreams::handlers::store]
pub fn store_balances(deltas: BlockBalanceDeltas, store: StoreAddBigInt) {
    tycho_substreams::balances::store_balance_changes(deltas, store);
}

/// Map protocol changes across transactions and balances
#[substreams::handlers::map]
pub fn map_protocol_changes(
    block: eth::v2::Block,
    grouped_components: BlockTransactionProtocolComponents,
    deltas: BlockBalanceDeltas,
    components_store: StoreGetInt64,
    balance_store: StoreDeltas,
) -> Result<BlockChanges, anyhow::Error> {
    let mut transaction_contract = HashMap::new();

    for tx_component in &grouped_components.tx_components {
        let tx = tx_component.tx.as_ref().unwrap();
        transaction_contract
            .entry(tx.index)
            .or_insert_with(|| TransactionChanges::new(tx))
            .component_changes
            .extend(tx_component.components.clone());
    }

    aggregate_balances_changes(balance_store, deltas)
        .into_iter()
        .for_each(|(_, (tx, balances))| {
            let tx_change = transaction_contract
                .entry(tx.index)
                .or_insert_with(|| TransactionChanges::new(&tx));

            tx_change.balance_changes.extend(
                balances
                    .into_values()
                    .flat_map(|map| map.into_values()),
            );
        });

    extract_contract_changes(
        &block,
        |addr| {
            components_store
                .get_last(format!("pool:0x{0}", hex::encode(addr)))
                .is_some()
        },
        &mut transaction_contract,
    );

    Ok(BlockChanges {
        block: Some((&block).into()),
        changes: transaction_contract
            .into_values()
            .filter(|change| !change.component_changes.is_empty())
            .collect(),
    })
}

/// Determine if a transaction deploys the Restake Manager
fn is_deployment_call(call: &eth::v2::Call, component_address: &[u8]) -> bool {
    call.account_creations
        .iter()
        .any(|ac| ac.account.as_slice() == component_address)
}

fn find_deployed_underlying_address(component_address: &[u8]) -> Option<[u8; 20]> {
    let result = match component_address {
        hex!("74a09653A083691711cF8215a6ab074BB4e99ef5") => {
            Some(hex!("bf5495Efe5DB9ce00f80364C8B423567e58d2110"))
        }
        _ => {
            substreams::log::info!(
                "Unknown component address: 0x{}",
                hex::encode(component_address)
            );
            None
        }
    };
    result
}
