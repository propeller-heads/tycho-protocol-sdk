use substreams::store::{StoreGet, StoreGetProto};
use substreams_ethereum::pb::eth::v2::{self as eth};
use tycho_substreams::prelude::*;
use tycho_substreams::models::{BalanceDelta, BlockBalanceDeltas, Transaction};
use std::collections::{HashSet, HashMap};
use crate::store_key::StoreKey;

const TRANSFER_TOPIC: [u8; 32] = hex_literal::hex!("ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef");

#[substreams::handlers::map]
pub fn map_balance_changes(
    block: eth::Block,
    pools_store: StoreGetProto<ProtocolComponent>,
    new_pools: BlockChanges,
) -> Result<BlockBalanceDeltas, substreams::errors::Error> {
    let mut balance_aggregates: HashMap<(String, Vec<u8>), (substreams::scalar::BigInt, u64, Option<Transaction>)> = HashMap::new();

    let created_pools: HashSet<String> = new_pools.changes.iter()
        .flat_map(|c| c.component_changes.iter().map(|pc| pc.id.clone()))
        .collect();

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

                    // Is the `from` address a pool?
                    let pool_match_from = pools_store.get_last(StoreKey::Pool.get_unique_pool_key(&from_hex));
                    let is_from_pool = pool_match_from
                        .as_ref()
                        .map(|pc| pc.tokens.contains(&token_address))
                        .unwrap_or(false) || created_pools.contains(&from_hex);

                    if pool_match_from.is_some() || created_pools.contains(&from_hex) {
                         substreams::log::debug!(
                            "Transfer FROM Pool: pool={} token={} match={} is_created={}",
                            from_hex,
                            format!("{}", substreams::Hex(&token_address)),
                            is_from_pool,
                            created_pools.contains(&from_hex)
                        );
                    }

                    if is_from_pool {
                        let key = (from_hex, token_address.clone());
                        let (current_delta, max_ord, _) = balance_aggregates.entry(key)
                            .or_insert((substreams::scalar::BigInt::from(0), 0, Some(current_tx.clone())));
                        *current_delta = current_delta.clone() - amount_bigint.clone();
                        *max_ord = std::cmp::max(*max_ord, log.ordinal);
                    }

                    // Is the `to` address a pool?
                    let pool_match_to = pools_store.get_last(StoreKey::Pool.get_unique_pool_key(&to_hex));
                    let is_to_pool = pool_match_to
                        .as_ref()
                        .map(|pc| pc.tokens.contains(&token_address))
                        .unwrap_or(false) || created_pools.contains(&to_hex);

                    if pool_match_to.is_some() || created_pools.contains(&to_hex) {
                         substreams::log::debug!(
                            "Transfer TO Pool: pool={} token={} match={} is_created={}",
                            to_hex,
                            format!("{}", substreams::Hex(&token_address)),
                            is_to_pool,
                            created_pools.contains(&to_hex)
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
