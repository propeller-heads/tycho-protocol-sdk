use crate::{
    abi::vault_contract::{
        events::PoolPausedStateChanged,
        functions::{Erc4626BufferWrapOrUnwrap, SendTo, Settle},
    },
    BATCH_ROUTER_ADDRESS, PERMIT_2_ADDRESS, VAULT_ADDRESS, VAULT_ADMIN, VAULT_EXPLORER,
    VAULT_EXTENSION_ADDRESS,
};
use anyhow::{Ok, Result};
use itertools::Itertools;
use keccak_hash::keccak;
use std::collections::HashMap;
use substreams::{
    pb::substreams::StoreDeltas,
    prelude::StoreGetString,
    scalar::BigInt,
    store::{StoreGet, StoreGetInt64, StoreGetProto},
};
use substreams_ethereum::{
    pb::eth::{self, v2::StorageChange},
    Event, Function,
};
use tycho_substreams::{
    abi, attributes::json_deserialize_address_list, balances::aggregate_balances_changes,
    block_storage::get_block_storage_changes, contract::extract_contract_changes_builder,
    entrypoint::create_entrypoint, models::entry_point_params::TraceData, prelude::*,
};

/// This is the main map that handles most of the indexing of this substream.
/// Every contract change is grouped by transaction index via the `transaction_changes`
///  map. Each block of code will extend the `TransactionChanges` struct with the
///  corresponding changes (balance, component, contract), inserting a new one if it doesn't exist.
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
            // initialize builder if not yet present for this tx
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
                        let wrapped_token = component.tokens[0].clone();
                        let token_decimals = get_token_decimals(&wrapped_token);
                        let trace_wrapped_token_amount = match token_decimals {
                            Some(decimals) => {
                                let base = BigInt::from(10);
                                base.pow(
                                    decimals
                                        .to_string()
                                        .parse::<u32>()
                                        .unwrap_or(18),
                                )
                            }
                            None => BigInt::from(1),
                        };

                        let asset_trace_data = TraceData::Rpc(RpcTraceData {
                            caller: None,
                            calldata: hex::decode("38d52e0f").unwrap(), // asset()
                        });
                        let (asset_entrypoint, asset_entrypoint_params) = create_entrypoint(
                            wrapped_token.clone(),
                            "asset()".to_string(),
                            component.id.clone(),
                            asset_trace_data,
                        );
                        builder.add_entrypoint(&asset_entrypoint);
                        builder.add_entrypoint_params(&asset_entrypoint_params);

                        let preview_deposit_trace_data = TraceData::Rpc(RpcTraceData {
                            caller: None,
                            calldata: build_entrypoint_calldata(
                                "ef8b30f7",
                                &trace_wrapped_token_amount,
                            ), // previewDeposit(uint256)(uint256)
                        });
                        let (preview_deposit_entrypoint, preview_deposit_entrypoint_params) =
                            create_entrypoint(
                                wrapped_token.clone(),
                                "previewDeposit(uint256)(uint256)".to_string(),
                                component.id.clone(),
                                preview_deposit_trace_data,
                            );
                        builder.add_entrypoint(&preview_deposit_entrypoint);
                        builder.add_entrypoint_params(&preview_deposit_entrypoint_params);

                        let preview_mint_trace_data = TraceData::Rpc(RpcTraceData {
                            caller: None,
                            calldata: build_entrypoint_calldata(
                                "b3d7f6b9",
                                &trace_wrapped_token_amount,
                            ), // previewMint(uint256)(uint256)
                        });
                        let (preview_mint_entrypoint, preview_mint_entrypoint_params) =
                            create_entrypoint(
                                wrapped_token.clone(),
                                "previewMint(uint256)(uint256)".to_string(),
                                component.id.clone(),
                                preview_mint_trace_data,
                            );
                        builder.add_entrypoint(&preview_mint_entrypoint);
                        builder.add_entrypoint_params(&preview_mint_entrypoint_params);

                        let preview_withdraw_trace_data = TraceData::Rpc(RpcTraceData {
                            caller: None,
                            calldata: build_entrypoint_calldata(
                                "0a28a477",
                                &trace_wrapped_token_amount,
                            ), // previewWithdraw(uint256)(uint256)
                        });
                        let (preview_withdraw_entrypoint, preview_withdraw_entrypoint_params) =
                            create_entrypoint(
                                wrapped_token.clone(),
                                "previewWithdraw(uint256)(uint256)".to_string(),
                                component.id.clone(),
                                preview_withdraw_trace_data,
                            );
                        builder.add_entrypoint(&preview_withdraw_entrypoint);
                        builder.add_entrypoint_params(&preview_withdraw_entrypoint_params);

                        let preview_redeem_trace_data = TraceData::Rpc(RpcTraceData {
                            caller: None,
                            calldata: build_entrypoint_calldata(
                                "4cdad506",
                                &trace_wrapped_token_amount,
                            ), // previewRedeem(uint256)(uint256)
                        });
                        let (preview_redeem_entrypoint, preview_redeem_entrypoint_params) =
                            create_entrypoint(
                                wrapped_token.clone(),
                                "previewRedeem(uint256)(uint256)".to_string(),
                                component.id.clone(),
                                preview_redeem_trace_data,
                            );
                        builder.add_entrypoint(&preview_redeem_entrypoint);
                        builder.add_entrypoint_params(&preview_redeem_entrypoint_params);
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
    // this block. Then, these balance changes are merged onto the existing map of tx contract
    // changes, inserting a new one if it doesn't exist.
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
                .is_some() ||
                addr.eq(VAULT_ADDRESS)
        },
        &mut transaction_changes,
    );

    // Extract token balances for balancer v3 vault
    block
        .transaction_traces
        .iter()
        .for_each(|tx| {
            let vault_balances = get_vault_reserves(tx, &tokens_store, &token_mapping_store);

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
                if let Some(underlying_token_hex) = token_mapping_store.get_last(&mapping_key) {
                    let underlying_token = hex::decode(&underlying_token_hex).unwrap();
                    for change in &call.storage_changes {
                        add_change_if_accounted(
                            &mut reserves_of,
                            change,
                            underlying_token.as_slice(),
                            token_store,
                        );
                    }
                }
            }
        });
    reserves_of
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

fn get_token_decimals(wrapped_token: &[u8]) -> Option<BigInt> {
    abi::erc20::functions::Decimals {}.call(wrapped_token.to_owned())
}

fn build_entrypoint_calldata(sig: &str, amount: &BigInt) -> Vec<u8> {
    let mut preview_deposit_calldata = hex::decode(sig).unwrap();
    let amount_bytes = amount.to_bytes_be().1;
    let mut padded_amount = vec![0u8; 32];
    let start_pos = 32 - amount_bytes.len();
    padded_amount[start_pos..].copy_from_slice(&amount_bytes);
    preview_deposit_calldata.extend_from_slice(&padded_amount);
    preview_deposit_calldata
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_preview_deposit_with_amount_1() {
        let function_selector = "ef8b30f7";
        let amount = BigInt::from(1);
        let result = build_entrypoint_calldata(function_selector, &amount);
        let expected =
            hex::decode("ef8b30f70000000000000000000000000000000000000000000000000000000000000001")
                .unwrap();
        assert_eq!(result, expected);
        assert_eq!(result.len(), 36);
    }
}
