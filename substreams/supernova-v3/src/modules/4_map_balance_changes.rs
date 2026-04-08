use substreams::store::{StoreGet, StoreGetProto};
use substreams_ethereum::pb::eth::v2::{self as eth};
use tycho_substreams::prelude::*;
use tycho_substreams::models::{BalanceDelta, BlockBalanceDeltas, Transaction};
use std::collections::HashMap;
use crate::store_key::StoreKey;

const TRANSFER_TOPIC: [u8; 32] = hex_literal::hex!("ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef");

#[substreams::handlers::map]
pub fn map_balance_changes(
    block: eth::Block,
    pools_store: StoreGetProto<ProtocolComponent>,
    new_pools: BlockChanges,
) -> Result<BlockBalanceDeltas, substreams::errors::Error> {
    let mut balance_aggregates: HashMap<(String, Vec<u8>), (substreams::scalar::BigInt, u64, Option<Transaction>)> = HashMap::new();

    // Note: we no longer build a separate `created_pools` set here. Newly
    // created pools are written to `pools_store` by `2_store_pools.rs` in the
    // same block, so they are reachable via `pools_store.get_last(...)` even
    // for transfers that happen later in the same block.
    let _ = &new_pools; // intentionally unused — kept in the signature for the substreams pipeline.

    for trx in block.transactions() {
        if trx.status != 1 {
            continue;
        }
        let current_tx = Transaction {
            to: trx.to.clone(),
            from: trx.from.clone(),
            hash: trx.hash.clone(),
            index: trx.index.into(),
        };

        if let Some(receipt) = trx.receipt.as_ref() {
            for log in &receipt.logs {
                if log.topics.len() == 3 && log.topics[0] == TRANSFER_TOPIC {
                    let from = &log.topics[1][12..32];
                    let to = &log.topics[2][12..32];
                    let token_address = log.address.clone();
                    let amount_bigint = substreams::scalar::BigInt::from_unsigned_bytes_be(&log.data);
                    
                    // IMPORTANT: pool component ids are stored as `0x{hex}` (lowercased)
                    // in `1_map_pool_created.rs` and the store in `2_store_pools.rs`.
                    // `substreams::Hex` produces UNPREFIXED hex, so we must add the `0x`
                    // prefix here to match. Without this prefix, every lookup misses and
                    // no balance deltas are ever emitted -> component_balance stays empty
                    // -> tycho-simulation's token mocks return 0 for balanceOf(pool)
                    // -> swap simulation fails.
                    let from_hex = format!("0x{}", substreams::Hex(from)).to_lowercase();
                    let to_hex = format!("0x{}", substreams::Hex(to)).to_lowercase();

                    // Is the `from` address a pool, AND is the moving token one of
                    // its tracked tokens? We require BOTH — it's not enough that the
                    // address belongs to a known pool. Without the token-membership
                    // check, an unrelated airdrop landing in the pool's wallet would
                    // be recorded as a balance delta and corrupt component_balance.
                    //
                    // We deliberately do NOT short-circuit on `created_pools` here:
                    // newly created pools are also written to `pools_store` in this
                    // same block by `2_store_pools.rs`, so the membership lookup
                    // already covers them.
                    let pool_match_from = pools_store.get_last(StoreKey::Pool.get_unique_pool_key(&from_hex));
                    let is_from_pool = pool_match_from
                        .as_ref()
                        .map(|pc| pc.tokens.contains(&token_address))
                        .unwrap_or(false);

                    if pool_match_from.is_some() {
                         substreams::log::debug!(
                            "Transfer FROM Pool: pool={} token={} match={}",
                            from_hex,
                            format!("{}", substreams::Hex(&token_address)),
                            is_from_pool,
                        );
                    }

                    if is_from_pool {
                        let key = (from_hex, token_address.clone());
                        let (current_delta, max_ord, _) = balance_aggregates.entry(key)
                            .or_insert((substreams::scalar::BigInt::from(0), 0, Some(current_tx.clone())));
                        *current_delta = current_delta.clone() - amount_bigint.clone();
                        *max_ord = std::cmp::max(*max_ord, log.ordinal);
                    }

                    // Same membership requirement as the FROM branch — see comment
                    // above. We require both pool-membership in the store AND
                    // token-membership in the pool's `tokens` list.
                    let pool_match_to = pools_store.get_last(StoreKey::Pool.get_unique_pool_key(&to_hex));
                    let is_to_pool = pool_match_to
                        .as_ref()
                        .map(|pc| pc.tokens.contains(&token_address))
                        .unwrap_or(false);

                    if pool_match_to.is_some() {
                         substreams::log::debug!(
                            "Transfer TO Pool: pool={} token={} match={}",
                            to_hex,
                            format!("{}", substreams::Hex(&token_address)),
                            is_to_pool,
                        );
                    }

                    if is_to_pool {
                        let key = (to_hex, token_address.clone());
                        let (current_delta, max_ord, _) = balance_aggregates.entry(key)
                            .or_insert((substreams::scalar::BigInt::from(0), 0, Some(current_tx.clone())));
                        *current_delta = current_delta.clone() + amount_bigint.clone();
                        *max_ord = std::cmp::max(*max_ord, log.ordinal);
                    }
                }
            }
        }
    }

    let balance_deltas = balance_aggregates.into_iter()
        .map(|((component_id, token), (delta, ord, tx))| BalanceDelta {
            ord,
            tx,
            token,
            delta: delta.to_signed_bytes_be(),
            component_id: component_id.into_bytes(),
        })
        .collect();

    Ok(BlockBalanceDeltas { balance_deltas })
}
