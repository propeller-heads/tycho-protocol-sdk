use crate::abi;
use anyhow::{Ok, Result};
use std::collections::HashMap;
use substreams::{
    hex,
    pb::substreams::StoreDeltas,
    store::{StoreAdd, StoreAddBigInt, StoreAddInt64, StoreGet, StoreGetInt64, StoreNew},
};
use substreams_ethereum::{
    pb::eth::{self},
    Event,
};
use tycho_substreams::{
    balances::aggregate_balances_changes, contract::extract_contract_changes, prelude::*,
};

pub const ETH_ADDRESS: [u8; 20] = [238u8; 20]; // 0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee

/// Map to extract protocol components for transactions in a block
#[substreams::handlers::map]
pub fn map_components(
    params: String,
    block: eth::v2::Block,
) -> Result<BlockTransactionProtocolComponents, anyhow::Error> {
    let component_address = hex::decode(params).unwrap();
    let component_token = find_deployed_underlying_address(&component_address).unwrap();
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
    store.add_many(
        0,
        &map.tx_components
            .iter()
            .flat_map(|tx_components| &tx_components.components)
            .map(|component| format!("pool:{0}", component.id))
            .collect::<Vec<_>>(),
        1,
    );
}


/// Map balance changes caused by deposit and withdrawal events
#[substreams::handlers::map]
pub fn map_relative_balances(
    block: eth::v2::Block,
    store: StoreGetInt64,
) -> Result<BlockBalanceDeltas, anyhow::Error> {
    let mut balance_deltas = Vec::new();

    for log in block.logs() {
        if let Some(ev) = abi::restake_manager_contract::events::Deposit::match_and_decode(log.log) {
            let _address_hex = format!("0x{}", hex::encode(log.log.address.clone()));
            if 1 == 1 { // let Some(_component_key) = store.get_last(format!("pool:{0}", address_hex)) {
                // let ez_eth_address = find_deployed_underlying_address(&log.log.address.clone());
                let ez_eth_address = hex!("bf5495Efe5DB9ce00f80364C8B423567e58d2110");
                let ez_eth_comp = ez_eth_address.to_vec();
                balance_deltas.push(BalanceDelta {
                    ord: log.log.ordinal,
                    tx: Some(log.receipt.transaction.into()),
                    token: ev.token.clone(),
                    delta: ev.amount.to_signed_bytes_be(),
                    component_id: ev.token.to_vec(),
                });

                // Handle the balance delta for minted ezETH
                balance_deltas.push(BalanceDelta {
                    ord: log.log.ordinal,
                    tx: Some(log.receipt.transaction.into()),
                    token: ez_eth_comp.clone(), // Use the declared ezETH address
                    delta: ev.ez_eth_minted.to_signed_bytes_be(),
                    component_id: ez_eth_comp, // Use ezETH address as ID
                });
        
            }
        } else if let Some(ev) = abi::restake_manager_contract::events::UserWithdrawCompleted::match_and_decode(log.log) {
            let address_hex = format!("0x{}", hex::encode(log.log.address.clone()));
            if let Some(_component_key) = store.get_last(format!("pool:{0}", address_hex)) {
                let ez_eth_address = find_deployed_underlying_address(&log.log.address.clone());
                let ez_eth_comp = ez_eth_address.unwrap().to_vec();
                balance_deltas.push(BalanceDelta {
                    ord: log.log.ordinal,
                    tx: Some(log.receipt.transaction.into()),
                    token: ev.token.clone(),
                    delta: ev.amount.neg().to_signed_bytes_be(),
                    component_id: ev.token.to_vec(),
                });

                // Handle the balance delta for minted ezETH
                balance_deltas.push(BalanceDelta {
                    ord: log.ordinal(),
                    tx: Some(log.receipt.transaction.into()),
                    token: ez_eth_comp.clone(), // Use the declared ezETH address
                    delta: ev.ez_eth_burned.neg().to_signed_bytes_be(),
                    component_id: ez_eth_comp, // Use ezETH address as ID
                });
        
            }
        }
    }

    Ok(BlockBalanceDeltas { balance_deltas })
}

// #[substreams::handlers::map]
// pub fn map_relative_balances(
//     block: eth::v2::Block,
//     store: StoreGetInt64,
// ) -> Result<BlockBalanceDeltas, anyhow::Error> {
//     let balance_deltas = block
//         .logs()
//         .flat_map(|component_log| {
//             let mut deltas = Vec::new();

//             if let Some(ev) =
//             abi::restake_manager_contract::events::Deposit::match_and_decode(component_log.log)
//             {
//                 let address_bytes_be = vault_log.address();
//                 let address_hex = format!("0x{}", hex::encode(address_bytes_be));
//                 if store
//                     .get_last(format!("pool:{}", address_hex))
//                     .is_some()
//                 {
//                     deltas.extend_from_slice(&[
//                         BalanceDelta {
//                             ord: vault_log.ordinal(),
//                             tx: Some(vault_log.receipt.transaction.into()),
//                             token: find_deployed_underlying_address(address_bytes_be)
//                                 .unwrap()
//                                 .to_vec(),
//                             delta: ev.assets.neg().to_signed_bytes_be(),
//                             component_id: address_hex.as_bytes().to_vec(),
//                         },
//                         BalanceDelta {
//                             ord: vault_log.ordinal(),
//                             tx: Some(vault_log.receipt.transaction.into()),
//                             token: address_bytes_be.to_vec(),
//                             delta: ev.shares.neg().to_signed_bytes_be(),
//                             component_id: address_hex.as_bytes().to_vec(),
//                         },
//                     ]);
//                     substreams::log::debug!(
//                         "Withdraw: vault: {}, frax:- {}, sfrax:- {}",
//                         address_hex,
//                         ev.assets,
//                         ev.shares
//                     );
//                 }
//             } else if let Some(ev) =
//                 abi::stakedfrax_contract::events::Deposit::match_and_decode(vault_log.log)
//             {
//                 let address_bytes_be = vault_log.address();
//                 let address_hex = format!("0x{}", hex::encode(address_bytes_be));

//                 if store
//                     .get_last(format!("pool:{}", address_hex))
//                     .is_some()
//                 {
//                     deltas.extend_from_slice(&[
//                         BalanceDelta {
//                             ord: vault_log.ordinal(),
//                             tx: Some(vault_log.receipt.transaction.into()),
//                             token: find_deployed_underlying_address(address_bytes_be)
//                                 .unwrap()
//                                 .to_vec(),
//                             delta: ev.assets.to_signed_bytes_be(),
//                             component_id: address_hex.as_bytes().to_vec(),
//                         },
//                         BalanceDelta {
//                             ord: vault_log.ordinal(),
//                             tx: Some(vault_log.receipt.transaction.into()),
//                             token: address_bytes_be.to_vec(),
//                             delta: ev.shares.to_signed_bytes_be(),
//                             component_id: address_hex.as_bytes().to_vec(),
//                         },
//                     ]);
//                     substreams::log::debug!(
//                         "Deposit: vault: {}, frax:+ {}, sfrax:+ {}",
//                         address_hex,
//                         ev.assets,
//                         ev.shares
//                     );
//                 }
//             } 
            
//             deltas
//         })
//         .collect::<Vec<_>>();

//     Ok(BlockBalanceDeltas { balance_deltas })
// }


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

            tx_change.balance_changes.extend(balances.into_values().flat_map(|map| map.into_values()));
        });

    extract_contract_changes(
        &block,
        |addr| components_store.get_last(format!("pool:0x{0}", hex::encode(addr))).is_some(),
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
    match component_address {
        hex!("74a09653A083691711cF8215a6ab074BB4e99ef5") => {
            Some(hex!("bf5495Efe5DB9ce00f80364C8B423567e58d2110"))
        }
        _ => None,
    }
}