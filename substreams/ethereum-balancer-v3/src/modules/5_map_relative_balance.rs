use crate::{
    abi::vault_contract::events::{
        LiquidityAdded, LiquidityAddedToBuffer, LiquidityRemoved, LiquidityRemovedFromBuffer, Swap,
        Unwrap, Wrap,
    },
    VAULT_ADDRESS,
};
use anyhow::{Ok, Result};
use substreams::{
    log,
    prelude::StoreGetString,
    store::{StoreGet, StoreGetProto},
};
use substreams_ethereum::{pb::eth, Event};
use tycho_substreams::prelude::*;
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
                    let underlying_token = hex::decode(&underlying_token_hex).unwrap();
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
            if let Some(remove_from_buffer) = LiquidityRemovedFromBuffer::match_and_decode(vault_log.log) {
                let mapping_key = format!("buffer_mapping_{}", hex::encode(remove_from_buffer.wrapped_token.as_slice()));
                if let Some(underlying_token_hex) = token_mapping_store.get_last(mapping_key) {
                    let underlying_token = hex::decode(&underlying_token_hex).unwrap();
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
            if let Some(wrap) = Wrap::match_and_decode(vault_log.log) {
                let mapping_key = format!("buffer_mapping_{}", hex::encode(wrap.wrapped_token.as_slice()));
                if let Some(underlying_token_hex) = token_mapping_store.get_last(mapping_key) {
                    let underlying_token = hex::decode(&underlying_token_hex).unwrap();
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
            if let Some(unwrap) = Unwrap::match_and_decode(vault_log.log) {
                let mapping_key = format!("buffer_mapping_{}", hex::encode(unwrap.wrapped_token.as_slice()));
                if let Some(underlying_token_hex) = token_mapping_store.get_last(mapping_key) {
                    let underlying_token = hex::decode(&underlying_token_hex).unwrap();
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
            deltas
        })
        .collect::<Vec<_>>();

    Ok(BlockBalanceDeltas { balance_deltas })
}
