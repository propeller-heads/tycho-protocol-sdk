use crate::abi::{restake_manager_contract, withdrawal_contract_contract};
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

/// Ethereum native token address representation
pub const ETH_ADDRESS: [u8; 20] = hex!("0000000000000000000000000000000000000000");

/// Restake manager contract address
pub const VAULT_ADDRESS: [u8; 20] = hex!("74a09653A083691711cF8215a6ab074BB4e99ef5");

/// ezETH address
pub const EZETH_ADDRESS: [u8; 20] = hex!("bf5495Efe5DB9ce00f80364C8B423567e58d2110");

/// Withdraw queue address
pub const WITHDRAW_QUEUE_ADDRESS: [u8; 20] = hex!("2946399B2CF1ec41A1890d81969293DE59E9C855");

#[substreams::handlers::map]
pub fn map_components(
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
                    .filter_map(|tx| {
                        // address doesn't exist before contract deployment, hence the first tx with
                        // a log.address = component_address is the deployment tx
                        if is_deployment_call(&tx) {
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
        .filter(|log| {
            log.address() == &RESTAKE_MANAGER_ADDRESS || log.address() == &WITHDRAW_QUEUE_ADDRESS
        })
        .flat_map(|log| {
            let mut deltas = Vec::new();

            // Try to decode as deposit
            if let Some(ev) = restake_manager_contract::events::Deposit::match_and_decode(&log) {
                let component_id = format!("0x{}", hex::encode(&ev.token));

                // Check if this is a tracked component
                if store
                    .get_last(&format!("pool:{0}", component_id))
                    .is_some()
                {
                    balance_deltas.extend_from_slice(&[
                        BalanceDelta {
                            ord: log.log.ordinal,
                            tx: Some(log.receipt.transaction.into()),
                            token: ev.token.to_vec(),
                            delta: ev.amount.to_signed_bytes_be(),
                            component_id: component_id.to_vec(),
                        },
                        BalanceDelta {
                            ord: log.log.ordinal,
                            tx: Some(log.receipt.transaction.into()),
                            token: ez_eth_address.clone(),
                            delta: ev.ez_eth_minted.to_signed_bytes_be(),
                            component_id: component_id.to_vec(),
                        },
                    ]);
                }
                // For some reason UserWithdraw* events are not being emitted in the renzo source
                // code we will use those from WithdrawQueue
            } else if let Some(ev) =
                withdrawal_contract_contract::events::WithdrawRequestClaimed::match_and_decode(&log)
            {
                let (token, _, amount_to_redeem, ezeth_burned, _) = ev.withdraw_request;
                let component_id = format!("0x{}", hex::encode(&token));

                if store
                    .get_last(&format!("pool:{0}", component_id))
                    .is_some()
                {
                    balance_deltas.extend_from_slice(&[
                        BalanceDelta {
                            ord: log.log.ordinal,
                            tx: Some(log.receipt.transaction.into()),
                            token: ev.token.clone(),
                            delta: amount_to_redeem
                                .neg()
                                .to_signed_bytes_be(),
                            component_id: component_id.to_vec(),
                        },
                        BalanceDelta {
                            ord: log.log.ordinal,
                            tx: Some(log.receipt.transaction.into()),
                            token: EZETH_ADDRESS.to_vec(),
                            delta: ezeth_burned.neg().to_signed_bytes_be(),
                            component_id: component_id.to_vec(),
                        },
                    ]);
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
fn is_deployment_call(tx: &eth::v2::Transaction) -> bool {
    // https://etherscan.io/tx/0xd944d0aa4dc9706abcba3a4320f386dc94e54d6c522ce9a0a494c933a16d91fa
    tx.hash() == hex!("d944d0aa4dc9706abcba3a4320f386dc94e54d6c522ce9a0a494c933a16d91fa")
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
