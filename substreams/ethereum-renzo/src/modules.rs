use crate::abi;
use anyhow::{Ok, Result, anyhow};
use itertools::Itertools;
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

#[substreams::handlers::map]
pub fn map_components(
    params: String,
    block: eth::v2::Block,
) -> Result<BlockTransactionProtocolComponents, anyhow::Error> {
    let restake_manager_address = hex::decode(params)?;
    let locked_assets = find_deployed_underlying_addresses(&restake_manager_address)
        .ok_or_else(|| anyhow!("No underlying assets found for restake manager"))?;

    Ok(BlockTransactionProtocolComponents {
        tx_components: block
            .transactions()
            .filter_map(|tx| {
                let components = tx
                    .calls()
                    .filter(|call| !call.call.state_reverted)
                    .filter_map(|_| {
                        if is_deployment_tx(&tx, &restake_manager_address) {
                            Some(
                                locked_assets
                                    .iter()
                                    .map(|token| {
                                        ProtocolComponent::at_contract(&restake_manager_address, &tx.into())
                                            .with_tokens(&[token.to_vec()])
                                            .as_swap_type("renzo_vault", ImplementationType::Vm)
                                    })
                                    .collect::<Vec<_>>()
                                    .
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

#[substreams::handlers::store]
pub fn store_components(map: BlockTransactionProtocolComponents, store: StoreAddInt64) {
    for tx_components in &map.tx_components {
        for component in &tx_components.components {
            let component_id = component.id.clone(); // Use the actual component ID (e.g., the token address)
            for token in &component.tokens {
                let token_hex = hex::encode(token); // Convert token address to hex
                let key = format!("restake_manager:{}:{}", component_id, token_hex);

                substreams::log::debug!("Storing component: key = {}, component_id = {}", key, component_id);

                // Add the component ID and token to the store
                store.add(0, &key, 1);
            }
        }
    }
}



#[substreams::handlers::map]
pub fn map_relative_balances(
    block: eth::v2::Block,
    store: StoreGetInt64,
) -> Result<BlockBalanceDeltas, anyhow::Error> {
    let ez_eth_address = hex::decode("bf5495Efe5DB9ce00f80364C8B423567e58d2110")?; // Declare ezETH address

    let mut balance_deltas = Vec::new();

    for log in block.logs() {
        let address_bytes_be = log.address();
        let address_hex = format!("0x{}", hex::encode(address_bytes_be));

        // Check for Deposit event
        if let Some(ev) = abi::restake_manager_contract::events::Deposit::match_and_decode(log) {
            // Use the `restake_manager` address to find associated underlying token addresses
            if let Some(underlying_addresses) = find_deployed_underlying_addresses(address_bytes_be) {
                // Check if the store has the relevant entry for the Restake Manager
                if store.get_last(format!("restake_manager:{}", address_hex)).is_some() {
                    // For each token, handle balance delta separately
                    for token in underlying_addresses {
                        balance_deltas.push(BalanceDelta {
                            ord: log.ordinal(),
                            tx: Some(log.receipt.transaction.into()),
                            token: token.clone(),
                            delta: ev.amount.to_signed_bytes_be(),
                            component_id: hex::encode(token), // Use token address as component ID
                        });
                    }

                    // Handle the balance delta for minted ezETH
                    balance_deltas.push(BalanceDelta {
                        ord: log.ordinal(),
                        tx: Some(log.receipt.transaction.into()),
                        token: ez_eth_address.clone(), // Use the declared ezETH address
                        delta: ev.ez_eth_minted.parse::<BigInt>()?.to_signed_bytes_be(),
                        component_id: hex::encode(&ez_eth_address), // Use ezETH address as ID
                    });
                }
            }

            substreams::log::debug!(
                "Deposit: token: {}, amount: {}, ezETH minted: {}",
                hex::encode(&ev.token),
                ev.amount,
                ev.ez_eth_minted
            );
        }

        // Check for UserWithdrawCompleted event
        if let Some(ev) = abi::restake_manager_contract::events::UserWithdrawCompleted::match_and_decode(log) {
            if let Some(underlying_addresses) = find_deployed_underlying_addresses(address_bytes_be) {
                // Check if the store has the relevant entry for the Restake Manager
                if store.get_last(format!("restake_manager:{}", address_hex)).is_some() {
                    // For each token, handle balance delta separately
                    for token in underlying_addresses {
                        balance_deltas.push(BalanceDelta {
                            ord: log.ordinal(),
                            tx: Some(log.receipt.transaction.into()),
                            token: token.clone(),
                            delta: ev.amount.parse::<BigInt>()?.neg().to_signed_bytes_be(),
                            component_id: hex::encode(token), // Use token address as component ID
                        });
                    }

                    // Handle the balance delta for burned ezETH
                    balance_deltas.push(BalanceDelta {
                        ord: log.ordinal(),
                        tx: Some(log.receipt.transaction.into()),
                        token: ez_eth_address.clone(), // Use the declared ezETH address
                        delta: ev.ez_eth_burned.parse::<BigInt>()?.neg().to_signed_bytes_be(),
                        component_id: hex::encode(&ez_eth_address), // Use ezETH address as ID
                    });
                }
            }

            substreams::log::debug!(
                "Withdraw: token: {}, amount: {}, ezETH burned: {}",
                hex::encode(&ev.token),
                ev.amount,
                ev.ez_eth_burned
            );
        }
    }

    Ok(BlockBalanceDeltas { balance_deltas })
}


#[substreams::handlers::store]
pub fn store_balances(deltas: BlockBalanceDeltas, store: StoreAddBigInt) {
    store_balance_changes(deltas, store);
}

#[substreams::handlers::map]
pub fn map_protocol_changes(
    block: eth::v2::Block,
    grouped_components: BlockTransactionProtocolComponents,
    deltas: BlockBalanceDeltas,
    components_store: StoreGetInt64,
    balance_store: StoreDeltas,
) -> Result<BlockChanges, anyhow::Error> {
    let mut transaction_contract: HashMap<u64, TransactionChanges> = HashMap::new();

    // Iterate over transaction components to process protocol changes
    grouped_components.tx_components.iter().for_each(|tx_component| {
        let tx = tx_component.tx.as_ref().unwrap();
        // For each transaction, insert or extend protocol components
        transaction_contract
            .entry(tx.index)
            .or_insert_with(|| TransactionChanges::new(tx))
            .component_changes
            .extend_from_slice(&tx_component.components);
    });

    aggregate_balances_changes(balance_store, deltas)
        .into_iter()
        .for_each(|(_, (tx, balances))| {
            let tx_change = transaction_contract
                .entry(tx.index)
                .or_insert_with(|| TransactionChanges::new(&tx));

            balances
                .into_values()
                .for_each(|token_bc_map| {
                    tx_change
                        .balance_changes
                        .extend(token_bc_map.into_values());
                });
        });

    extract_contract_changes(
        &block,
        |addr| {
            components_store
                .get_last(format!("restake_manager:{}", hex::encode(addr)))
                .is_some()
        },
        &mut transaction_contract,
    );

    Ok(BlockChanges {
        block: Some((&block).into()),
        changes: transaction_contract
            .drain()
            .sorted_unstable_by_key(|(index, _)| *index)
            .filter_map(|(_, change)| {
                if change.contract_changes.is_empty() &&
                    change.component_changes.is_empty() &&
                    change.balance_changes.is_empty()
                {
                    None
                } else {
                    Some(change)
                }
            })
            .collect::<Vec<_>>(),
    })
}



fn find_deployed_underlying_addresses(restake_manager_address: &[u8]) -> Option<Vec<Vec<u8>>> {
    match restake_manager_address {
        hex!("74a09653A083691711cF8215a6ab074BB4e99ef5") => Some(vec![
            hex!("bf5495Efe5DB9ce00f80364C8B423567e58d2110").to_vec(),
            hex!("a2E3356610840701BDf5611a53974510Ae27E2e1").to_vec(),
            hex!("ae7ab96520DE3A18E5e111B5EaAb095312D7fE84").to_vec(),
            hex!("0000000000000000000000000000000000000000").to_vec(),
        ]),
        _ => None,
    }
}

fn is_deployment_tx(tx: &eth::v2::TransactionTrace, restake_manager_address: &[u8]) -> bool {
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
        return deployed_address.as_slice() == restake_manager_address;
    }
    false
}