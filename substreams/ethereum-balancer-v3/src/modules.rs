use crate::{
    abi::vault_contract::{
        events::{
            LiquidityAdded, LiquidityAddedToBuffer, LiquidityRemoved, LiquidityRemovedFromBuffer,
            PoolPausedStateChanged, Swap, Unwrap, Wrap,
        },
        functions::{Erc4626BufferWrapOrUnwrap, SendTo, Settle},
    },
    pool_factories,
};
use anyhow::Result;
use itertools::Itertools;
use keccak_hash::keccak;
use std::{collections::HashMap, ops::Shr};
use substreams::{
    hex, log,
    pb::substreams::StoreDeltas,
    prelude::{StoreGetString, StoreSet},
    scalar::BigInt,
    store::{
        StoreAddBigInt, StoreGet, StoreGetInt64, StoreGetProto, StoreNew, StoreSetIfNotExists,
        StoreSetIfNotExistsInt64, StoreSetIfNotExistsProto, StoreSetString,
    },
};
use substreams_ethereum::{
    pb::eth::{self, v2::StorageChange},
    Event, Function,
};
use tycho_substreams::{
    attributes::json_deserialize_address_list, balances::aggregate_balances_changes,
    block_storage::get_block_storage_changes, contract::extract_contract_changes_builder,
    entrypoint::create_entrypoint, models::entry_point_params::TraceData, prelude::*,
};

pub const VAULT_ADDRESS: &[u8] = &hex!("bA1333333333a1BA1108E8412f11850A5C319bA9");
pub const VAULT_EXTENSION_ADDRESS: &[u8; 20] = &hex!("0E8B07657D719B86e06bF0806D6729e3D528C9A9");
pub const VAULT_EXPLORER: &[u8; 20] = &hex!("Fc2986feAB34713E659da84F3B1FA32c1da95832");
pub const VAULT_ADMIN: &[u8; 20] = &hex!("35fFB749B273bEb20F40f35EdeB805012C539864");
pub const BATCH_ROUTER_ADDRESS: &[u8; 20] = &hex!("136f1efcc3f8f88516b9e94110d56fdbfb1778d1");
pub const PERMIT_2_ADDRESS: &[u8; 20] = &hex!("000000000022D473030F116dDEE9F6B43aC78BA3");

#[substreams::handlers::map]
pub fn map_components(block: eth::v2::Block) -> Result<BlockTransactionProtocolComponents> {
    let mut tx_components = Vec::new();
    for tx in block.transactions() {
        let mut components = Vec::new();
        for (log, call) in tx.logs_with_calls() {
            if let Some(component) =
                pool_factories::address_map(log.address.as_slice(), log, call.call)
            {
                components.push(component);
            }
        }
        if !components.is_empty() {
            tx_components.push(TransactionProtocolComponents { tx: Some(tx.into()), components });
        }
    }
    Ok(BlockTransactionProtocolComponents { tx_components })
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

/// Set of token that are used by BalancerV3. This is used to filter out account balances updates
/// for unknown tokens.
#[substreams::handlers::store]
pub fn store_token_set(map: BlockTransactionProtocolComponents, store: StoreSetIfNotExistsInt64) {
    map.tx_components
        .into_iter()
        .for_each(|tx_pc| {
            tx_pc
                .components
                .into_iter()
                .for_each(|pc| {
                    pc.tokens
                        .into_iter()
                        .for_each(|token| store.set_if_not_exists(0, hex::encode(token), &1));
                })
        });
}

#[substreams::handlers::store]
pub fn store_token_mapping(map: BlockTransactionProtocolComponents, store: StoreSetString) {
    map.tx_components
        .into_iter()
        .for_each(|tx_pc| {
            tx_pc
                .components
                .into_iter()
                .for_each(|pc| {
                    if let Some(pool_type) = pc.get_attribute_value("pool_type") {
                        if pool_type == "LiquidityBuffer".as_bytes() && pc.tokens.len() >= 2 {
                            let wrapped_token = hex::encode(&pc.tokens[0]);
                            let underlying_token = hex::encode(&pc.tokens[1]);
                            let mapping_key = format!("buffer_mapping_{}", wrapped_token);
                            store.set(0, mapping_key, &underlying_token);
                        }
                    }
                })
        });
}

#[substreams::handlers::map]
pub fn map_relative_balances(
    block: eth::v2::Block,
    store: StoreGetProto<ProtocolComponent>,
    token_mapping_store: StoreGetString,
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
            if let Some(added_to_buffer) = LiquidityAddedToBuffer::match_and_decode(vault_log.log) {
                let mapping_key = format!("buffer_mapping_{}", hex::encode(added_to_buffer.wrapped_token.as_slice()));
                if let Some(underlying_token_hex) = token_mapping_store.get_last(mapping_key) {
                    if let Ok(underlying_token) = hex::decode(&underlying_token_hex) {
                        if let Some(component) = store.get_last(format!("pool:{}", &format!("0x{}", hex::encode(&added_to_buffer.wrapped_token)))) {
                            let wrapped_delta = BalanceDelta {
                                ord: vault_log.ordinal(),
                                tx: Some(vault_log.receipt.transaction.into()),
                                token: added_to_buffer.wrapped_token.to_vec(),
                                delta: added_to_buffer.amount_wrapped.to_signed_bytes_be(),
                                component_id: component.id.as_bytes().to_vec(),
                            };
                            let underlying_delta = BalanceDelta {
                                ord: vault_log.ordinal(),
                                tx: Some(vault_log.receipt.transaction.into()),
                                token: underlying_token,
                                delta: added_to_buffer.amount_underlying.to_signed_bytes_be(),
                                component_id: component.id.as_bytes().to_vec(),
                            };
                            deltas.extend_from_slice(&[wrapped_delta, underlying_delta]);
                        }
                    }
                }
            }
            if let Some(remove_from_buffer) = LiquidityRemovedFromBuffer::match_and_decode(vault_log.log) {
                let mapping_key = format!("buffer_mapping_{}", hex::encode(remove_from_buffer.wrapped_token.as_slice()));
                if let Some(underlying_token_hex) = token_mapping_store.get_last(mapping_key) {
                    if let Ok(underlying_token) = hex::decode(&underlying_token_hex) {
                        if let Some(component) = store.get_last(format!("pool:{}", &format!("0x{}", hex::encode(&remove_from_buffer.wrapped_token)))) {
                            let wrapped_delta = BalanceDelta {
                                ord: vault_log.ordinal(),
                                tx: Some(vault_log.receipt.transaction.into()),
                                token: remove_from_buffer.wrapped_token.to_vec(),
                                delta: remove_from_buffer.amount_wrapped.neg().to_signed_bytes_be(),
                                component_id: component.id.as_bytes().to_vec(),
                            };
                            let underlying_delta = BalanceDelta {
                                ord: vault_log.ordinal(),
                                tx: Some(vault_log.receipt.transaction.into()),
                                token: underlying_token,
                                delta: remove_from_buffer.amount_underlying.neg().to_signed_bytes_be(),
                                component_id: component.id.as_bytes().to_vec(),
                            };
                            deltas.extend_from_slice(&[wrapped_delta, underlying_delta]);
                        }
                    }
                }
            }
            if let Some(wrap) = Wrap::match_and_decode(vault_log.log) {
                let mapping_key = format!("buffer_mapping_{}", hex::encode(wrap.wrapped_token.as_slice()));
                if let Some(underlying_token_hex) = token_mapping_store.get_last(mapping_key) {
                    if let Ok(underlying_token) = hex::decode(&underlying_token_hex) {
                        if let Some(component) = store.get_last(format!("pool:{}", &format!("0x{}", hex::encode(&wrap.wrapped_token)))) {
                            let wrapped_delta = BalanceDelta {
                                ord: vault_log.ordinal(),
                                tx: Some(vault_log.receipt.transaction.into()),
                                token: wrap.wrapped_token.to_vec(),
                                delta: wrap.minted_shares.neg().to_signed_bytes_be(),
                                component_id: component.id.as_bytes().to_vec(),
                            };
                            let underlying_delta = BalanceDelta {
                                ord: vault_log.ordinal(),
                                tx: Some(vault_log.receipt.transaction.into()),
                                token: underlying_token,
                                delta: wrap.deposited_underlying.to_signed_bytes_be(),
                                component_id: component.id.as_bytes().to_vec(),
                            };
                            deltas.extend_from_slice(&[wrapped_delta, underlying_delta]);
                        }
                    }
                }
            }
            if let Some(unwrap) = Unwrap::match_and_decode(vault_log.log) {
                let mapping_key = format!("buffer_mapping_{}", hex::encode(unwrap.wrapped_token.as_slice()));
                if let Some(underlying_token_hex) = token_mapping_store.get_last(mapping_key) {
                    if let Ok(underlying_token) = hex::decode(&underlying_token_hex) {
                        if let Some(component) = store.get_last(format!("pool:{}", &format!("0x{}", hex::encode(&unwrap.wrapped_token)))) {
                            let wrapped_delta = BalanceDelta {
                                ord: vault_log.ordinal(),
                                tx: Some(vault_log.receipt.transaction.into()),
                                token: unwrap.wrapped_token.to_vec(),
                                delta: unwrap.burned_shares.to_signed_bytes_be(),
                                component_id: component.id.as_bytes().to_vec(),
                            };
                            let underlying_delta = BalanceDelta {
                                ord: vault_log.ordinal(),
                                tx: Some(vault_log.receipt.transaction.into()),
                                token: underlying_token,
                                delta: unwrap.withdrawn_underlying.neg().to_signed_bytes_be(),
                                component_id: component.id.as_bytes().to_vec(),
                            };
                            deltas.extend_from_slice(&[wrapped_delta, underlying_delta]);
                        }
                    }
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
    tokens_store: StoreGetInt64,
    token_mapping_store: StoreGetString,
    balance_store: StoreDeltas, // Note, this map module is using the `deltas` mode for the store.
) -> Result<BlockChanges> {
    // We merge contract changes by transaction (identified by transaction index) making it easy to
    //  sort them at the very end.
    let mut transaction_changes: HashMap<_, TransactionChangesBuilder> = HashMap::new();
    // Handle pool pause state changes
    block
        .logs()
        .filter(|log| log.address() == VAULT_ADDRESS)
        .for_each(|log| {
            if let Some(PoolPausedStateChanged { pool, paused }) =
                PoolPausedStateChanged::match_and_decode(log)
            {
                let component_id = format!("0x{}", hex::encode(&pool));
                let tx: Transaction = log.receipt.transaction.into();
                if components_store
                    .get_last(format!("pool:{}", &component_id))
                    .is_some()
                {
                    let builder = transaction_changes
                        .entry(tx.index)
                        .or_insert_with(|| TransactionChangesBuilder::new(&tx));

                    let entity_change = EntityChanges {
                        component_id,
                        attributes: vec![Attribute {
                            name: "paused".to_string(),
                            value: vec![1u8],
                            change: if paused {
                                ChangeType::Creation.into()
                            } else {
                                ChangeType::Deletion.into()
                            },
                        }],
                    };
                    builder.add_entity_change(&entity_change);
                }
            }
        });

    // `ProtocolComponents` are gathered from `map_pools_created` which just need a bit of work to
    //   convert into `TransactionChanges`
    let default_attributes = vec![
        Attribute {
            // TODO: remove this and track account_balances instead
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
            name: "stateless_contract_addr_3".into(),
            value: address_to_bytes_with_0x(VAULT_EXPLORER),
            change: ChangeType::Creation.into(),
        },
        Attribute {
            name: "stateless_contract_addr_4".into(),
            value: address_to_bytes_with_0x(VAULT_ADMIN),
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
                    let rate_providers = component
                        .static_att
                        .iter()
                        .find(|att| att.name == "rate_providers")
                        .map(|att| json_deserialize_address_list(&att.value));

                    if let Some(rate_providers) = rate_providers {
                        for rate_provider in rate_providers {
                            let trace_data = TraceData::Rpc(RpcTraceData {
                                caller: None,
                                calldata: hex::decode("679aefce").unwrap(), // getRate()
                            });
                            let (entrypoint, entrypoint_params) = create_entrypoint(
                                rate_provider,
                                "getRate()".to_string(),
                                component.id.clone(),
                                trace_data,
                            );
                            builder.add_entrypoint(&entrypoint);
                            builder.add_entrypoint_params(&entrypoint_params);
                        }
                    }

                    let pool_type = component
                        .get_attribute_value("pool_type")
                        .expect("pool type not exist");
                    if pool_type == "LiquidityBuffer".as_bytes() {
                        let asset_trace_data = TraceData::Rpc(RpcTraceData {
                            caller: None,
                            calldata: hex::decode("38d52e0f").unwrap(), // asset()
                        });
                        let (asset_entrypoint, asset_entrypoint_params) = create_entrypoint(
                            component.tokens[0].clone(),
                            "asset()".to_string(),
                            component.id.clone(),
                            asset_trace_data,
                        );
                        builder.add_entrypoint(&asset_entrypoint);
                        builder.add_entrypoint_params(&asset_entrypoint_params);

                        let total_asset_trace_data = TraceData::Rpc(RpcTraceData {
                            caller: None,
                            calldata: hex::decode("01e1d114").unwrap(), // totalAssets()
                        });
                        let (total_asset_entrypoint, total_asset_entrypoint_params) =
                            create_entrypoint(
                                component.tokens[0].clone(),
                                "totalAssets()".to_string(),
                                component.id.clone(),
                                total_asset_trace_data,
                            );
                        builder.add_entrypoint(&total_asset_entrypoint);
                        builder.add_entrypoint_params(&total_asset_entrypoint_params);

                        let total_supply_trace_data = TraceData::Rpc(RpcTraceData {
                            caller: None,
                            calldata: hex::decode("18160ddd").unwrap(), // totalSupply()
                        });
                        let (total_supply_entrypoint, total_supply_entrypoint_params) =
                            create_entrypoint(
                                component.tokens[0].clone(),
                                "totalSupply()".to_string(),
                                component.id.clone(),
                                total_supply_trace_data,
                            );
                        builder.add_entrypoint(&total_supply_entrypoint);
                        builder.add_entrypoint_params(&total_supply_entrypoint_params);

                        let decimals_trace_data = TraceData::Rpc(RpcTraceData {
                            caller: None,
                            calldata: hex::decode("313ce567").unwrap(), // decimals()
                        });
                        let (decimals_entrypoint, decimals_entrypoint_params) = create_entrypoint(
                            component.tokens[0].clone(),
                            "decimals()".to_string(),
                            component.id.clone(),
                            decimals_trace_data,
                        );
                        builder.add_entrypoint(&decimals_entrypoint);
                        builder.add_entrypoint_params(&decimals_entrypoint_params);
                    }

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
        .iter()
        .for_each(|(_, (tx, balances))| {
            let builder = transaction_changes
                .entry(tx.index)
                .or_insert_with(|| TransactionChangesBuilder::new(tx));

            balances
                .values()
                .for_each(|token_bc_map| {
                    token_bc_map.values().for_each(|bc| {
                        builder.add_balance_change(bc);
                    })
                });
        });

    // Extract and insert any storage changes that happened for any of the components.
    extract_contract_changes_builder(
        &block,
        |addr| {
            components_store
                .get_last(format!("pool:0x{0}", hex::encode(addr)))
                .is_some()
                || addr.eq(VAULT_ADDRESS)
        },
        &mut transaction_changes,
    );

    // Extract token balances for balancer v3 vault
    block
        .transaction_traces
        .iter()
        .for_each(|tx| {
            let mut vault_balances = get_vault_reserves(tx, &tokens_store, &token_mapping_store);

            vault_balances.extend(get_vault_buffer_balances(
                &token_mapping_store,
                tx,
                &tokens_store,
            ));

            if !vault_balances.is_empty() {
                let tycho_tx = Transaction::from(tx);
                let builder = transaction_changes
                    .entry(tx.index.into())
                    .or_insert_with(|| TransactionChangesBuilder::new(&tycho_tx));

                let mut contract_changes = InterimContractChange::new(VAULT_ADDRESS, false);
                for (token_addr, reserve_value) in vault_balances {
                    contract_changes.upsert_token_balance(
                        token_addr.as_slice(),
                        reserve_value.value.as_slice(),
                    );
                }

                builder.add_contract_changes(&contract_changes);
            }
        });

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

    let block_storage_changes = get_block_storage_changes(&block);

    // Process all `transaction_changes` for final output in the `BlockChanges`,
    //  sorted by transaction index (the key).
    Ok(BlockChanges {
        block: Some((&block).into()),
        changes: transaction_changes
            .drain()
            .sorted_unstable_by_key(|(index, _)| *index)
            .filter_map(|(_, builder)| builder.build())
            .collect::<Vec<_>>(),
        storage_changes: block_storage_changes,
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

// function needed to match reservesOf in vault storage, which by definition
// they should always be equal to the `token.balanceOf(this)` except during unlock
fn get_vault_reserves(
    transaction: &eth::v2::TransactionTrace,
    token_store: &StoreGetInt64,
    token_mapping_store: &StoreGetString,
) -> HashMap<Vec<u8>, ReserveValue> {
    // reservesOf mapping for the current block Address -> Balance
    let mut reserves_of = HashMap::new();
    transaction
        .calls
        .iter()
        .filter(|call| !call.state_reverted)
        .filter(|call| call.address == VAULT_ADDRESS)
        .for_each(|call| {
            if let Some(Settle { token, .. }) = Settle::match_and_decode(call) {
                for change in &call.storage_changes {
                    add_change_if_accounted(
                        &mut reserves_of,
                        change,
                        token.as_slice(),
                        token_store,
                    );
                }
            }
            if let Some(SendTo { token, .. }) = SendTo::match_and_decode(call) {
                for change in &call.storage_changes {
                    add_change_if_accounted(
                        &mut reserves_of,
                        change,
                        token.as_slice(),
                        token_store,
                    );
                }
            }
            if let Some(params) = Erc4626BufferWrapOrUnwrap::match_and_decode(call) {
                let wrapped_token = params.params.2.as_slice();
                for change in &call.storage_changes {
                    add_change_if_accounted(&mut reserves_of, change, wrapped_token, token_store);
                }
                let mapping_key = format!("buffer_mapping_{}", hex::encode(wrapped_token));
                match token_mapping_store.get_last(&mapping_key) {
                    Some(underlying_token_hex) => match hex::decode(&underlying_token_hex) {
                        Ok(underlying_token) => {
                            for change in &call.storage_changes {
                                add_change_if_accounted(
                                    &mut reserves_of,
                                    change,
                                    underlying_token.as_slice(),
                                    token_store,
                                );
                            }
                        }
                        Err(e) => {
                            log::info!(
                                "Failed to decode underlying token hex: {}, error: {:?}",
                                underlying_token_hex,
                                e
                            );
                        }
                    },
                    None => {
                        log::info!("No mapping found for wrapped_token key: {}", mapping_key);
                    }
                }
            }
        });
    reserves_of
}

// function needed to match bufferTokenBalances in vault storage
fn get_vault_buffer_balances(
    token_mapping_store: &StoreGetString,
    transaction: &eth::v2::TransactionTrace,
    token_store: &StoreGetInt64,
) -> HashMap<Vec<u8>, ReserveValue> {
    let mut buffer_balance = HashMap::new();

    for (log, call_view) in transaction.logs_with_calls() {
        let wrapped_token_opt = if let Some(e) = LiquidityAddedToBuffer::match_and_decode(log) {
            Some(e.wrapped_token)
        } else if let Some(e) = LiquidityRemovedFromBuffer::match_and_decode(log) {
            Some(e.wrapped_token)
        } else if let Some(e) = Wrap::match_and_decode(log) {
            Some(e.wrapped_token)
        } else if let Some(e) = Unwrap::match_and_decode(log) {
            Some(e.wrapped_token)
        } else {
            None
        };

        if let Some(wrapped_token) = wrapped_token_opt {
            let mapping_key = format!("buffer_mapping_{}", hex::encode(wrapped_token.as_slice()));
            if let Some(underlying_token_hex) = token_mapping_store.get_last(mapping_key) {
                if let Ok(underlying_token) = hex::decode(&underlying_token_hex) {
                    for change in &call_view.call.storage_changes {
                        add_buffer_balance_change(
                            &mut buffer_balance,
                            change,
                            wrapped_token.as_slice(),
                            &underlying_token,
                            token_store,
                        );
                    }
                }
            }
        }
    }
    buffer_balance
}

struct ReserveValue {
    ordinal: u64,
    value: Vec<u8>,
}

fn add_change_if_accounted(
    reserves_of: &mut HashMap<Vec<u8>, ReserveValue>,
    change: &StorageChange,
    token_address: &[u8],
    token_store: &StoreGetInt64,
) {
    let slot_key = get_storage_key_for_token(token_address);
    // record changes happening on vault contract at reserves_of storage key
    if change.key == slot_key && token_store.has_last(hex::encode(token_address)) {
        reserves_of
            .entry(token_address.to_vec())
            .and_modify(|v| {
                if v.ordinal < change.ordinal {
                    v.value = change.new_value.clone();
                    v.ordinal = change.ordinal;
                }
            })
            .or_insert(ReserveValue { value: change.new_value.clone(), ordinal: change.ordinal });
    }
}

fn add_buffer_balance_change(
    buffer_balance: &mut HashMap<Vec<u8>, ReserveValue>,
    change: &StorageChange,
    wrapped_token: &[u8],
    underlying_token: &[u8],
    token_store: &StoreGetInt64,
) {
    let slot_key = get_storage_key_for_buffer_token(wrapped_token);
    // record changes happening on vault contract at _bufferTokenBalances storage key
    if change.key == slot_key && token_store.has_last(hex::encode(wrapped_token)) {
        let raw_balance = get_balance_raw(change.new_value.clone());
        let derived_balance = get_balance_derived(change.new_value.clone());

        buffer_balance
            .entry(wrapped_token.to_vec())
            .and_modify(|v| {
                if v.ordinal < change.ordinal {
                    v.value = derived_balance.clone();
                    v.ordinal = change.ordinal;
                }
            })
            .or_insert(ReserveValue { value: derived_balance.clone(), ordinal: change.ordinal });

        buffer_balance
            .entry(underlying_token.to_vec())
            .and_modify(|v| {
                if v.ordinal < change.ordinal {
                    v.value = raw_balance.clone();
                    v.ordinal = change.ordinal;
                }
            })
            .or_insert(ReserveValue { value: raw_balance.clone(), ordinal: change.ordinal });
    }
}

// token_addr -> keccak256(abi.encode(token_address, 8)) as 8 is the order in which reserves of are
// declared
fn get_storage_key_for_token(token_address: &[u8]) -> Vec<u8> {
    let mut input = [0u8; 64];
    input[12..32].copy_from_slice(token_address);
    input[63] = 8u8;
    let result = keccak(input.as_slice())
        .as_bytes()
        .to_vec();
    result
}

// token_addr -> keccak256(abi.encode(token_address, 11)) as 8 is the order in which
// bufferTokenBalances are declared
fn get_storage_key_for_buffer_token(token_address: &[u8]) -> Vec<u8> {
    let mut input = [0u8; 64];
    input[12..32].copy_from_slice(token_address);
    input[63] = 11u8;
    let result = keccak(input.as_slice())
        .as_bytes()
        .to_vec();
    result
}

fn max_balance() -> BigInt {
    // 2^128 - 1
    (BigInt::one().shr(128u8)) - BigInt::one()
}

// get balance raw value from the packed token balance
// https://github.com/balancer/balancer-v3-monorepo/blob/d32067a7ed47ec1680aa0b7ecf2e177e13a87fa4/pkg/solidity-utils/contracts/helpers/PackedTokenBalance.sol#L26-L28
fn get_balance_raw(value: Vec<u8>) -> Vec<u8> {
    (BigInt::from_unsigned_bytes_be(&value) & max_balance()).to_signed_bytes_be()
}

// get balance derived value from the packed token balance
// https://github.com/balancer/balancer-v3-monorepo/blob/d32067a7ed47ec1680aa0b7ecf2e177e13a87fa4/pkg/solidity-utils/contracts/helpers/PackedTokenBalance.sol#L30-L32
fn get_balance_derived(value: Vec<u8>) -> Vec<u8> {
    ((BigInt::from_unsigned_bytes_be(&value).shr(128u8)) & max_balance()).to_signed_bytes_be()
}
