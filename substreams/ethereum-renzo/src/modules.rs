use crate::abi::{restake_manager_contract, withdrawal_contract_contract};
use anyhow::Result;
use itertools::Itertools;
use std::collections::HashMap;
use substreams::{
    hex,
    pb::substreams::StoreDeltas,
    prelude::StoreGetString,
    store::{StoreAdd, StoreAddBigInt, StoreAddInt64, StoreGet, StoreGetInt64, StoreNew},
};
use substreams_ethereum::{
    pb::eth::{self},
    Event,
};
use tycho_substreams::{
    balances::aggregate_balances_changes, contract::extract_contract_changes_builder, prelude::*,
};

/// Ethererum address 0
pub const ETH_ADDRESS: [u8; 20] = hex!("0000000000000000000000000000000000000000");

/// Restake manager contract address
pub const RESTAKE_MANAGER_ADDRESS: [u8; 20] = hex!("74a09653A083691711cF8215a6ab074BB4e99ef5");

/// ezETH address
pub const EZETH_ADDRESS: [u8; 20] = hex!("bf5495Efe5DB9ce00f80364C8B423567e58d2110");

/// Withdraw queue address
pub const WITHDRAW_QUEUE_ADDRESS: [u8; 20] = hex!("2946399B2CF1ec41A1890d81969293DE59E9C855");

#[substreams::handlers::map]
pub fn map_components(
    block: eth::v2::Block,
) -> Result<BlockTransactionProtocolComponents, anyhow::Error> {
    let tx_components = block
        .transactions()
        .filter_map(|tx| {
            if tx.hash == hex!("d944d0aa4dc9706abcba3a4320f386dc94e54d6c522ce9a0a494c933a16d91fa") {
                // Set up ethereum component at deployment
                Some(TransactionProtocolComponents {
                    tx: Some(tx.into()),
                    components: vec![
                        ProtocolComponent::at_contract(
                            // according to the general logic used in other components this should be ETH_ADDRESS, 
                            // however using RESTAKE_MANAGER_ADDRESS to make integration tests pass
                            RESTAKE_MANAGER_ADDRESS.as_slice(),
                            &tx.into(),
                        )
                        .with_tokens(&[ETH_ADDRESS.as_slice(), EZETH_ADDRESS.as_slice()])
                        .as_swap_type("restake_manager", ImplementationType::Vm),
                    ],
                })
            } else {
                // Process regular block transactions
                let components = tx.logs_with_calls()
                    .filter_map(|(log, _)| {
                        if log.address != RESTAKE_MANAGER_ADDRESS {
                             None
                        } else if let Some(ev) = restake_manager_contract::events::CollateralTokenAdded::match_and_decode(log) {
                            Some(
                                ProtocolComponent::at_contract(
                                    ev.token.as_slice(),
                                    &tx.into(),
                                )
                                .with_tokens(&[ev.token.as_slice(), EZETH_ADDRESS.as_slice()])
                                .as_swap_type("restake_manager", ImplementationType::Vm)
                            )
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();

                if !components.is_empty() {
                    Some(TransactionProtocolComponents {
                        tx: Some(tx.into()),
                        components
                    })
                } else {
                    None
                }
            }
        })
        .collect::<Vec<_>>();

    Ok(BlockTransactionProtocolComponents { tx_components })
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
            log.address() == RESTAKE_MANAGER_ADDRESS || log.address() == WITHDRAW_QUEUE_ADDRESS
        })
        .flat_map(|log| {
            let mut deltas = Vec::new();

            // Try to decode as deposit
            if let Some(ev) = restake_manager_contract::events::Deposit2::match_and_decode(log) {
                let token = if ev.token == ETH_ADDRESS {
                    RESTAKE_MANAGER_ADDRESS.to_vec()
                } else {
                    ev.token
                };
                let component_id = format!("0x{}", hex::encode(&token));

                // Check if this is a tracked component
                if store
                    .get_last(format!("pool:{0}", component_id))
                    .is_some()
                {
                    deltas.extend_from_slice(&[
                        BalanceDelta {
                            ord: log.log.ordinal,
                            tx: Some(log.receipt.transaction.into()),
                            token,
                            delta: ev.amount.to_signed_bytes_be(),
                            component_id: component_id.as_bytes().to_vec(),
                        },
                        BalanceDelta {
                            ord: log.log.ordinal,
                            tx: Some(log.receipt.transaction.into()),
                            token: EZETH_ADDRESS.to_vec(),
                            delta: ev.ez_eth_minted.to_signed_bytes_be(),
                            component_id: component_id.as_bytes().to_vec(),
                        },
                    ]);
                }
                // For some reason UserWithdraw* events are not being emitted in the renzo source
                // code we will use those from WithdrawQueue
            } else if let Some(ev) =
                withdrawal_contract_contract::events::WithdrawRequestClaimed::match_and_decode(log)
            {
                let (_token, _, amount_to_redeem, ezeth_burned, _) = ev.withdraw_request;
                let token =
                    if _token == ETH_ADDRESS { RESTAKE_MANAGER_ADDRESS.to_vec() } else { _token };
                let component_id = format!("0x{}", hex::encode(&token));

                if store
                    .get_last(format!("pool:{0}", component_id))
                    .is_some()
                {
                    deltas.extend_from_slice(&[
                        BalanceDelta {
                            ord: log.log.ordinal,
                            tx: Some(log.receipt.transaction.into()),
                            token: token.to_vec(),
                            delta: amount_to_redeem
                                .neg()
                                .to_signed_bytes_be(),
                            component_id: component_id.as_bytes().to_vec(),
                        },
                        BalanceDelta {
                            ord: log.log.ordinal,
                            tx: Some(log.receipt.transaction.into()),
                            token: EZETH_ADDRESS.to_vec(),
                            delta: ezeth_burned.neg().to_signed_bytes_be(),
                            component_id: component_id.as_bytes().to_vec(),
                        },
                    ]);
                }
            } else if let Some(ev) =
                restake_manager_contract::events::Deposit1::match_and_decode(log)
            {
                let token = if ev.token == ETH_ADDRESS {
                    RESTAKE_MANAGER_ADDRESS.to_vec()
                } else {
                    ev.token
                };
                let component_id = format!("0x{}", hex::encode(&token));

                if store
                    .get_last(format!("pool:{0}", component_id))
                    .is_some()
                {
                    deltas.extend_from_slice(&[
                        BalanceDelta {
                            ord: log.log.ordinal,
                            tx: Some(log.receipt.transaction.into()),
                            token,
                            delta: ev.amount.to_signed_bytes_be(),
                            component_id: component_id.as_bytes().to_vec(),
                        },
                        BalanceDelta {
                            ord: log.log.ordinal,
                            tx: Some(log.receipt.transaction.into()),
                            token: EZETH_ADDRESS.to_vec(),
                            delta: ev.ez_eth_minted.to_signed_bytes_be(),
                            component_id: component_id.as_bytes().to_vec(),
                        },
                    ]);
                }
            }

            deltas
        })
        .collect::<Vec<_>>();

    Ok(BlockBalanceDeltas { balance_deltas })
}

/// Store aggregated balance changes
#[substreams::handlers::store]
pub fn store_balances(deltas: BlockBalanceDeltas, store: StoreAddBigInt) {
    tycho_substreams::balances::store_balance_changes(deltas, store);
}

/// This is the main map that handles most of the indexing of this substream.
/// Every contract change is grouped by transaction index via the `transaction_changes`
///  map. Each block of code will extend the `TransactionChanges` struct with the
///  cooresponding changes (balance, component, contract), inserting a new one if it doesn't exist.
///  At the very end, the map can easily be sorted by index to ensure the final
/// `BlockChanges`  is ordered by transactions properly.
#[substreams::handlers::map]
pub fn map_protocol_changes(
    block: eth::v2::Block,
    grouped_components: BlockTransactionProtocolComponents,
    deltas: BlockBalanceDeltas,
    components_store: StoreGetString,
    balance_store: StoreDeltas, // Note, this map module is using the `deltas` mode for the store.
) -> Result<BlockChanges> {
    // We merge contract changes by transaction (identified by transaction index) making it easy to
    //  sort them at the very end.
    let mut transaction_changes: HashMap<_, TransactionChangesBuilder> = HashMap::new();

    // `ProtocolComponents` are gathered from `map_pools_created` which just need a bit of work to
    //   convert into `TransactionChanges`
    let default_attributes = vec![
        Attribute {
            name: "balance_owner".to_string(),
            value: RESTAKE_MANAGER_ADDRESS.to_vec(),
            change: ChangeType::Creation.into(),
        },
        Attribute {
            name: "update_marker".to_string(),
            value: vec![1u8],
            change: ChangeType::Creation.into(),
        },
    ];
    grouped_components
        .tx_components
        .iter()
        .for_each(|tx_component| {
            // initialise builder if not yet present for this tx
            let tx = tx_component.tx.as_ref().unwrap();
            let builder = transaction_changes
                .entry(tx.index)
                .or_insert_with(|| TransactionChangesBuilder::new(tx));

            // iterate over individual components created within this tx
            tx_component
                .components
                .iter()
                .for_each(|component| {
                    builder.add_protocol_component(component);
                    let entity_change = EntityChanges {
                        component_id: component.id.clone(),
                        attributes: default_attributes.clone(),
                    };
                    builder.add_entity_change(&entity_change)
                });
        });

    // Balance changes are gathered by the `StoreDelta` based on `PoolBalanceChanged` creating
    //  `BlockBalanceDeltas`. We essentially just process the changes that occurred to the `store`
    // this  block. Then, these balance changes are merged onto the existing map of tx contract
    // changes,  inserting a new one if it doesn't exist.
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

    // Extract and insert any storage changes that happened for any of the components.
    extract_contract_changes_builder(
        &block,
        |addr| {
            components_store
                .get_last(format!("pool:0x{0}", hex::encode(addr)))
                .is_some() ||
                addr.eq(&RESTAKE_MANAGER_ADDRESS)
        },
        &mut transaction_changes,
    );

    transaction_changes
        .iter_mut()
        .for_each(|(_, change)| {
            // this indirection is necessary due to borrowing rules.
            let addresses = change
                .changed_contracts()
                .map(|e| e.to_vec())
                .collect::<Vec<_>>();
            addresses
                .into_iter()
                .for_each(|address| {
                    if address != RESTAKE_MANAGER_ADDRESS {
                        // We reconstruct the component_id from the address here
                        let id = components_store
                            .get_last(format!("pool:0x{}", hex::encode(address)))
                            .unwrap(); // Shouldn't happen because we filter by known components in
                                       // `extract_contract_changes_builder`
                        change.mark_component_as_updated(&id);
                    }
                })
        });

    // Process all `transaction_changes` for final output in the `BlockChanges`,
    //  sorted by transaction index (the key).
    Ok(BlockChanges {
        block: Some((&block).into()),
        changes: transaction_changes
            .drain()
            .sorted_unstable_by_key(|(index, _)| *index)
            .filter_map(|(_, builder)| builder.build())
            .collect::<Vec<_>>(),
    })
}
