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

/// Map to extract protocol components for transactions in a block
#[substreams::handlers::map]
pub fn map_components(
    params: String,
    block: eth::v2::Block,
) -> Result<BlockTransactionProtocolComponents, anyhow::Error> {
    let restake_manager_address = hex::decode(&params)?;
    let token_addresses = vec![
        hex!("ae7ab96520DE3A18E5e111B5EaAb095312D7fE84"), // stETH
        hex!("a2E3356610840701BDf5611a53974510Ae27E2e1"), // wBETH
        hex!("0000000000000000000000000000000000000000"), // ETH
        hex!("bf5495Efe5DB9ce00f80364C8B423567e58d2110"), // ezETH
    ];

    let tx_components = block.transactions()
        .filter_map(|tx| {
            let components = tx.calls()
                .filter(|call| !call.call.state_reverted)
                .filter_map(|_| {
                    if is_deployment_tx(&tx, &restake_manager_address) {
                        Some(
                            ProtocolComponent::at_contract(&restake_manager_address, &tx.into())
                                .with_tokens(&token_addresses)
                                .as_swap_type("restake_manager", ImplementationType::Vm),
                        )
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();

            if components.is_empty() {
                None
            } else {
                Some(TransactionProtocolComponents {
                    tx: Some(tx.into()),
                    components,
                })
            }
        })
        .collect();

    Ok(BlockTransactionProtocolComponents { tx_components })
}

/// Store protocol components and associated tokens
#[substreams::handlers::store]
pub fn store_components(map: BlockTransactionProtocolComponents, store: StoreAddInt64) {
    for tx_components in &map.tx_components {
        for component in &tx_components.components {
            let component_key = format!("restake_manager:{}", component.id);
            store.add(0, &component_key, 1);

            for token in &component.tokens {
                let token_key = format!("{}:token:{}", component_key, hex::encode(token));
                store.add(0, &token_key, 1);
            }
        }
    }
}

/// Map balance changes caused by deposit and withdrawal events
#[substreams::handlers::map]
pub fn map_relative_balances(
    block: eth::v2::Block,
    store: StoreGetInt64,
) -> Result<BlockBalanceDeltas, anyhow::Error> {
    let ez_eth_address = hex!("bf5495Efe5DB9ce00f80364C8B423567e58d2110");
    let mut balance_deltas = Vec::new();

    for log in block.logs() {
        if let Some(ev) = abi::restake_manager_contract::events::Deposit::match_and_decode(log.log) {
            let address_hex = format!("0x{}", hex::encode(log.log.address.clone()));
            if let Some(_component_key) = store.get_last(format!("restake_manager:{}", address_hex)) {
                balance_deltas.push(BalanceDelta {
                    ord: log.log.ordinal,
                    tx: Some(log.receipt.transaction.into()),
                    token: ev.token.clone(),
                    delta: ev.amount.to_signed_bytes_be(),
                    component_id: ev.token.to_vec(),
                });

                // Handle the balance delta for minted ezETH
                balance_deltas.push(BalanceDelta {
                    ord: log.ordinal(),
                    tx: Some(log.receipt.transaction.into()),
                    token: ez_eth_address.to_vec(), // Use the declared ezETH address
                    delta: ev.ez_eth_minted.to_signed_bytes_be(),
                    component_id: ez_eth_address.to_vec(), // Use ezETH address as ID
                });
        
            }
        } else if let Some(ev) = abi::restake_manager_contract::events::UserWithdrawCompleted::match_and_decode(log.log) {
            let address_hex = format!("0x{}", hex::encode(log.log.address.clone()));
            if let Some(_component_key) = store.get_last(format!("restake_manager:{}", address_hex)) {
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
                    token: ez_eth_address.to_vec(), // Use the declared ezETH address
                    delta: ev.ez_eth_burned.neg().to_signed_bytes_be(),
                    component_id: ez_eth_address.to_vec(), // Use ezETH address as ID
                });
        
            }
        }
    }

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

            tx_change.balance_changes.extend(balances.into_values().flat_map(|map| map.into_values()));
        });

    extract_contract_changes(
        &block,
        |addr| components_store.get_last(format!("restake_manager:{}", hex::encode(addr))).is_some(),
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
fn is_deployment_tx(tx: &eth::v2::TransactionTrace, restake_manager_address: &[u8]) -> bool {
    tx.calls.iter().any(|call| {
        call.account_creations
            .iter()
            .any(|ac| ac.account.as_slice() == restake_manager_address)
    })
}
