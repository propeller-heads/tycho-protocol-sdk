use crate::{
    abi::vault_contract::events::{
        LiquidityAdded, LiquidityAddedToBuffer, LiquidityRemoved, LiquidityRemovedFromBuffer, Swap,
        Unwrap, Wrap,
    },
    pool_factories,
};
use anyhow::Result;
use itertools::Itertools;
use std::collections::HashMap;
use substreams::{
    hex, log,
    pb::substreams::StoreDeltas,
    store::{
        StoreAddBigInt, StoreGet, StoreGetProto, StoreNew, StoreSetIfNotExists,
        StoreSetIfNotExistsProto,
    },
};
use substreams_ethereum::{pb::eth, Event};
use tycho_substreams::{
    balances::aggregate_balances_changes, contract::extract_contract_changes_builder, prelude::*,
};

pub const VAULT_ADDRESS: &[u8] = &hex!("bA1333333333a1BA1108E8412f11850A5C319bA9");
pub const VAULT_EXTENSION_ADDRESS: &[u8; 20] = &hex!("0E8B07657D719B86e06bF0806D6729e3D528C9A9");
pub const BATCH_ROUTER_ADDRESS: &[u8; 20] = &hex!("136f1efcc3f8f88516b9e94110d56fdbfb1778d1");
pub const PERMIT_2_ADDRESS: &[u8; 20] = &hex!("000000000022D473030F116dDEE9F6B43aC78BA3");

#[substreams::handlers::map]
pub fn map_components(block: eth::v2::Block) -> Result<BlockTransactionProtocolComponents> {
    // Gather contract changes by indexing `PoolCreated` events and analysing the `Create` call
    // We store these as a hashmap by tx hash since we need to agg by tx hash later
    Ok(BlockTransactionProtocolComponents {
        tx_components: block
            .transactions()
            .filter_map(|tx| {
                let components = tx
                    .logs_with_calls()
                    .filter_map(|(log, call)| {
                        pool_factories::address_map(
                            call.call.address.as_slice(),
                            log,
                            call.call,
                            tx,
                        )
                        .or_else(|| pool_factories::buffer_map(log, tx))
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

/// Simply stores the `ProtocolComponent`s with the pool address as the key and the pool id as value
#[substreams::handlers::store]
pub fn store_components(
    map: BlockTransactionProtocolComponents,
    store: StoreSetIfNotExistsProto<ProtocolComponent>,
) {
    map.tx_components
        .into_iter()
        .for_each(|tx_pc| {
            tx_pc
                .components
                .into_iter()
                .for_each(|pc| store.set_if_not_exists(0, format!("pool:{0}", &pc.id), &pc))
        });
}

#[substreams::handlers::map]
pub fn map_relative_balances(
    block: eth::v2::Block,
    store: StoreGetProto<ProtocolComponent>,
) -> Result<BlockBalanceDeltas, anyhow::Error> {
    let balance_deltas = block
        .logs()
        .filter(|log| log.address() == VAULT_ADDRESS)
        .flat_map(|vault_log| {
            let mut deltas = Vec::new();

            if let Some(Swap { pool, token_in, token_out, amount_in, amount_out, .. }) =
                Swap::match_and_decode(vault_log.log)
            {
                let component_id = format!("0x{}", hex::encode(pool));
                log::info!(
                    "swap at component id: {:?} with key: {:?}",
                    component_id,
                    format!("pool:{}", &component_id)
                );

                if store
                    .get_last(format!("pool:{}", &component_id))
                    .is_some()
                {
                    deltas.extend_from_slice(&[
                        BalanceDelta {
                            ord: vault_log.ordinal(),
                            tx: Some(vault_log.receipt.transaction.into()),
                            token: token_in.to_vec(),
                            delta: amount_in.to_signed_bytes_be(),
                            component_id: component_id.as_bytes().to_vec(),
                        },
                        BalanceDelta {
                            ord: vault_log.ordinal(),
                            tx: Some(vault_log.receipt.transaction.into()),
                            token: token_out.to_vec(),
                            delta: amount_out.neg().to_signed_bytes_be(),
                            component_id: component_id.as_bytes().to_vec(),
                        },
                    ]);
                }
            }
            if let Some(LiquidityAdded { pool, amounts_added_raw, .. }) =
                LiquidityAdded::match_and_decode(vault_log.log)
            {
                let component_id = format!("0x{}", hex::encode(pool));
                if let Some(component) = store.get_last(format!("pool:{}", &component_id)) {
                    if component.tokens.len() != amounts_added_raw.len() {
                        panic!(
                            "liquidity added to pool with different number of tokens than expected"
                        );
                    }
                    log::info!(
                        "liquidity added at component id: {:?} with key: {:?} with tokens: {:?}",
                        component_id,
                        format!("pool:{}", &component_id),
                        component.tokens
                    );
                    let deltas_from_added_liquidity = amounts_added_raw
                        .into_iter()
                        .zip(component.tokens.iter())
                        .map(|(amount, token)| BalanceDelta {
                            ord: vault_log.ordinal(),
                            tx: Some(vault_log.receipt.transaction.into()),
                            token: token.to_vec(),
                            delta: amount.to_signed_bytes_be(),
                            component_id: component_id.as_bytes().to_vec(),
                        })
                        .collect::<Vec<_>>();
                    deltas.extend_from_slice(&deltas_from_added_liquidity);
                }
            }
            if let Some(LiquidityRemoved { pool, amounts_removed_raw, .. }) =
                LiquidityRemoved::match_and_decode(vault_log.log)
            {
                let component_id = format!("0x{}", hex::encode(pool));
                log::info!(
                    "liquidity removed at component id: {:?} with key: {:?}",
                    component_id,
                    format!("pool:{}", &component_id)
                );
                if let Some(component) = store.get_last(format!("pool:{}", &component_id)) {
                    if component.tokens.len() != amounts_removed_raw.len() {
                        panic!(
                            "liquidity removed from pool with different number of tokens than expected"
                        );
                    }
                    let deltas_from_removed_liquidity = amounts_removed_raw
                        .into_iter()
                        .zip(component.tokens.iter())
                        .map(|(amount, token)| BalanceDelta {
                            ord: vault_log.ordinal(),
                            tx: Some(vault_log.receipt.transaction.into()),
                            token: token.to_vec(),
                            delta: amount.neg().to_signed_bytes_be(),
                            component_id: component_id.as_bytes().to_vec(),
                        })
                        .collect::<Vec<_>>();
                    deltas.extend_from_slice(&deltas_from_removed_liquidity);
                }
            }
            if let Some(LiquidityAddedToBuffer { wrapped_token, amount_underlying, amount_wrapped, .. }) =
                LiquidityAddedToBuffer::match_and_decode(vault_log.log)
            {
                let component_id = format!("0x{}", hex::encode(&wrapped_token));
                if let Some(ProtocolComponent{ tokens,..}) = store.get_last(format!("pool:{}", &component_id)) {
                    let underlying_token = tokens[1].to_owned();
                    deltas.extend_from_slice(&[BalanceDelta {
                            ord: vault_log.ordinal(),
                            tx: Some(vault_log.receipt.transaction.into()),
                            token: wrapped_token.to_vec(),
                            delta: amount_wrapped.to_signed_bytes_be(),
                            component_id: component_id.as_bytes().to_vec(),
                            },
                        BalanceDelta {
                            ord: vault_log.ordinal(),
                            tx: Some(vault_log.receipt.transaction.into()),
                            token: underlying_token.to_vec(),
                            delta: amount_underlying.to_signed_bytes_be(),
                            component_id: component_id.as_bytes().to_vec(),
                        },
                    ]);
                }
            }
            if let Some(LiquidityRemovedFromBuffer { wrapped_token, amount_underlying, amount_wrapped, .. }) =
                LiquidityRemovedFromBuffer::match_and_decode(vault_log.log)
            {
                let component_id = format!("0x{}", hex::encode(&wrapped_token));
                if let Some(ProtocolComponent{ tokens,..}) = store.get_last(format!("pool:{}", &component_id)) {
                    let underlying_token = tokens[1].to_owned();
                    deltas.extend_from_slice(&[BalanceDelta {
                            ord: vault_log.ordinal(),
                            tx: Some(vault_log.receipt.transaction.into()),
                            token: wrapped_token.to_vec(),
                            delta: amount_wrapped.neg().to_signed_bytes_be(),
                            component_id: component_id.as_bytes().to_vec(),
                            },
                        BalanceDelta {
                            ord: vault_log.ordinal(),
                            tx: Some(vault_log.receipt.transaction.into()),
                            token: underlying_token.to_vec(),
                            delta: amount_underlying.neg().to_signed_bytes_be(),
                            component_id: component_id.as_bytes().to_vec(),
                        },
                    ]);
                }
            }
            if let Some(Wrap { wrapped_token, deposited_underlying, minted_shares, .. }) = Wrap::match_and_decode(vault_log.log) {
                let component_id = format!("0x{}", hex::encode(&wrapped_token));
                if let Some(ProtocolComponent{ tokens,..}) = store.get_last(format!("pool:{}", &component_id)) {
                    let underlying_token = tokens[1].to_owned();
                    deltas.extend_from_slice(&[BalanceDelta {
                        ord: vault_log.ordinal(),
                        tx: Some(vault_log.receipt.transaction.into()),
                        token: underlying_token.to_vec(),
                        delta: deposited_underlying.to_signed_bytes_be(),
                        component_id: component_id.as_bytes().to_vec(),
                    },
                    BalanceDelta {
                        ord: vault_log.ordinal(),
                        tx: Some(vault_log.receipt.transaction.into()),
                        token: wrapped_token.to_vec(),
                        delta: minted_shares.to_signed_bytes_be(),
                        component_id: component_id.as_bytes().to_vec(),
                    },
                    ]);
                }
            }
            if let Some(Unwrap { wrapped_token, burned_shares, withdrawn_underlying, .. }) = Unwrap::match_and_decode(vault_log.log) {
                let component_id = format!("0x{}", hex::encode(&wrapped_token));
                if let Some(ProtocolComponent{ tokens,..}) = store.get_last(format!("pool:{}", &component_id)) {
                    let underlying_token = tokens[1].to_owned();
                    deltas.extend_from_slice(&[BalanceDelta {
                        ord: vault_log.ordinal(),
                        tx: Some(vault_log.receipt.transaction.into()),
                        token: underlying_token.to_vec(),
                        delta: withdrawn_underlying.neg().to_signed_bytes_be(),
                        component_id: component_id.as_bytes().to_vec(),
                    },
                    BalanceDelta {
                        ord: vault_log.ordinal(),
                        tx: Some(vault_log.receipt.transaction.into()),
                        token: wrapped_token.to_vec(),
                        delta: burned_shares.neg().to_signed_bytes_be(),
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

/// It's significant to include both the `pool_id` and the `token_id` for each balance delta as the
///  store key to ensure that there's a unique balance being tallied for each.
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
    components_store: StoreGetProto<ProtocolComponent>,
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
            value: VAULT_ADDRESS.to_vec(),
            change: ChangeType::Creation.into(),
        },
        Attribute {
            name: "stateless_contract_addr_0".into(),
            value: address_to_bytes_with_0x(VAULT_EXTENSION_ADDRESS),
            change: ChangeType::Creation.into(),
        },
        Attribute {
            name: "stateless_contract_addr_1".into(),
            value: address_to_bytes_with_0x(BATCH_ROUTER_ADDRESS),
            change: ChangeType::Creation.into(),
        },
        Attribute {
            name: "stateless_contract_addr_2".into(),
            value: address_to_bytes_with_0x(PERMIT_2_ADDRESS),
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
                addr.eq(VAULT_ADDRESS)
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
                    if address != VAULT_ADDRESS {
                        // We reconstruct the component_id from the address here
                        let id = components_store
                            .get_last(format!("pool:0x{}", hex::encode(address)))
                            .map(|c| c.id)
                            .unwrap(); // Shouldn't happen because we filter by known components
                                       // in `extract_contract_changes_builder`
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

/// Converts address bytes into a Vec<u8> containing a leading `0x`.
fn address_to_bytes_with_0x(address: &[u8; 20]) -> Vec<u8> {
    address_to_string_with_0x(address).into_bytes()
}

/// Converts address bytes into a string containing a leading `0x`.
fn address_to_string_with_0x(address: &[u8]) -> String {
    format!("0x{}", hex::encode(address))
}
