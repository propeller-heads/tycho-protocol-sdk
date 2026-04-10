use substreams::store::{StoreGet, StoreGetProto};
use substreams_ethereum::pb::eth::v2::{self as eth};
use tycho_substreams::prelude::*;
use tycho_substreams::models::{BalanceDelta, BlockBalanceDeltas, Transaction};
use std::collections::HashMap;
use crate::store_key::StoreKey;

const TRANSFER_TOPIC: [u8; 32] = hex_literal::hex!("ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef");

/// One side-effect of a Transfer event on a tracked pool's balance.
///
/// `delta` is **signed**: negative if the pool was the `from` side
/// (lost tokens) and positive if it was the `to` side (gained tokens).
/// A pool-to-pool transfer produces TWO effects, one for each side.
#[derive(Debug, Clone, PartialEq)]
pub struct TransferEffect {
    pub pool_id: String, // "0x..." lowercase
    pub token: Vec<u8>,
    pub delta: substreams::scalar::BigInt,
    pub log_ordinal: u64,
}

/// Classify a single ERC20 Transfer log into 0–2 balance-delta effects
/// on tracked pools. This is the pure, testable core of the
/// balance-delta pipeline; the `map_balance_changes` handler aggregates
/// these effects across all logs in a block.
///
/// Filtering rules:
///   1. The log must be a standard ERC20 `Transfer(address,address,uint256)` —
///      i.e. exactly 3 topics with the canonical Transfer signature in topic 0.
///   2. For each side (`from` / `to`), the address must resolve to a known
///      pool via `pool_lookup`, AND the moving token must be in that pool's
///      tokens list. Both conditions are required — see review finding H2:
///      without the token-membership check, an unrelated airdrop landing in
///      the pool's wallet would corrupt `component_balance`.
///
/// The `pool_lookup` closure takes a `0x`-prefixed lowercased hex address
/// (matching how `2_store_pools.rs` writes its keys) and returns the list
/// of tokens that pool tracks, or `None` if the address is not a known pool.
pub fn classify_transfer_log<F>(log: &eth::Log, pool_lookup: F) -> Vec<TransferEffect>
where
    F: Fn(&str) -> Option<Vec<Vec<u8>>>,
{
    let mut effects = Vec::new();

    // 1. Filter to standard Transfer(address,address,uint256) events.
    if log.topics.len() != 3 || log.topics[0] != TRANSFER_TOPIC {
        return effects;
    }
    // Defensive: a malformed Transfer log might have <32-byte topics.
    if log.topics[1].len() < 32 || log.topics[2].len() < 32 {
        return effects;
    }

    // 2. Extract from / to / amount.
    let from = &log.topics[1][12..32];
    let to = &log.topics[2][12..32];
    let token = log.address.clone();
    let amount = substreams::scalar::BigInt::from_unsigned_bytes_be(&log.data);

    // 3. Pool ids in `pools_store` are stored as `0x{hex}` (lowercased)
    //    by `2_store_pools.rs`. `substreams::Hex` produces UNPREFIXED hex,
    //    so the `0x` prefix here is required. (See review finding A5 — the
    //    original bug was a missing prefix that made every lookup miss.)
    let from_hex = format!("0x{}", substreams::Hex(from)).to_lowercase();
    let to_hex = format!("0x{}", substreams::Hex(to)).to_lowercase();

    // 4. FROM side: was the sender a tracked pool that holds this token?
    if let Some(tokens) = pool_lookup(&from_hex) {
        if tokens.contains(&token) {
            // BigInt doesn't impl Neg, so we negate via subtraction from zero.
            let neg_amount = substreams::scalar::BigInt::from(0) - amount.clone();
            effects.push(TransferEffect {
                pool_id: from_hex.clone(),
                token: token.clone(),
                delta: neg_amount,
                log_ordinal: log.ordinal,
            });
        }
    }

    // 5. TO side: was the receiver a tracked pool that holds this token?
    if let Some(tokens) = pool_lookup(&to_hex) {
        if tokens.contains(&token) {
            effects.push(TransferEffect {
                pool_id: to_hex,
                token,
                delta: amount,
                log_ordinal: log.ordinal,
            });
        }
    }

    effects
}

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
                // Delegate per-log classification to the pure helper.
                // The closure adapts `pools_store` (which returns a full
                // ProtocolComponent) into the `Vec<Vec<u8>>` token-list
                // shape the helper expects.
                let effects = classify_transfer_log(log, |addr_hex| {
                    pools_store
                        .get_last(StoreKey::Pool.get_unique_pool_key(addr_hex))
                        .map(|pc| pc.tokens.clone())
                });

                for effect in effects {
                    let key = (effect.pool_id, effect.token);
                    let (current_delta, max_ord, _) = balance_aggregates
                        .entry(key)
                        .or_insert((
                            substreams::scalar::BigInt::from(0),
                            0,
                            Some(current_tx.clone()),
                        ));
                    *current_delta = current_delta.clone() + effect.delta;
                    *max_ord = std::cmp::max(*max_ord, effect.log_ordinal);
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

#[cfg(test)]
mod tests {
    use super::*;
    use substreams::scalar::BigInt;
    use substreams_ethereum::pb::eth::v2::Log;

    /// Build a Transfer log from raw inputs.
    /// `from`/`to` should be 20-byte addresses; we pad them to 32 bytes
    /// in the topic encoding the same way the EVM does.
    fn make_transfer_log(
        token: Vec<u8>,
        from: [u8; 20],
        to: [u8; 20],
        amount: u64,
        ordinal: u64,
    ) -> Log {
        let mut from_topic = vec![0u8; 32];
        from_topic[12..32].copy_from_slice(&from);
        let mut to_topic = vec![0u8; 32];
        to_topic[12..32].copy_from_slice(&to);
        Log {
            address: token,
            topics: vec![TRANSFER_TOPIC.to_vec(), from_topic, to_topic],
            data: amount.to_be_bytes().to_vec(),
            index: 0,
            block_index: 0,
            ordinal,
        }
    }

    /// Build a "no pools known" lookup closure.
    fn no_pools() -> impl Fn(&str) -> Option<Vec<Vec<u8>>> {
        |_| None
    }

    /// Build a single-pool lookup closure that returns the given tokens
    /// when queried for `pool_addr_hex`, and None otherwise.
    fn one_pool(
        pool_addr_hex: String,
        tokens: Vec<Vec<u8>>,
    ) -> impl Fn(&str) -> Option<Vec<Vec<u8>>> {
        move |addr| {
            if addr == pool_addr_hex {
                Some(tokens.clone())
            } else {
                None
            }
        }
    }

    // ────────────────────────────────────────────────────────────────
    //  Filter behaviour: non-Transfer / malformed logs
    // ────────────────────────────────────────────────────────────────

    /// A log with a non-Transfer topic must produce zero effects.
    #[test]
    fn classify_non_transfer_log_yields_no_effects() {
        let log = Log {
            address: vec![0x42u8; 20],
            // Random non-Transfer topic
            topics: vec![vec![0xffu8; 32], vec![0u8; 32], vec![0u8; 32]],
            data: 1000u64.to_be_bytes().to_vec(),
            index: 0,
            block_index: 0,
            ordinal: 0,
        };
        let effects = classify_transfer_log(&log, no_pools());
        assert!(effects.is_empty());
    }

    /// A Transfer event with the wrong number of topics (e.g. a non-
    /// indexed Transfer variant from some non-standard token) must
    /// be skipped — we only handle the canonical 3-topic shape.
    #[test]
    fn classify_transfer_log_with_wrong_topic_count() {
        let log = Log {
            address: vec![0x42u8; 20],
            topics: vec![TRANSFER_TOPIC.to_vec(), vec![0u8; 32]], // only 2 topics
            data: 1000u64.to_be_bytes().to_vec(),
            index: 0,
            block_index: 0,
            ordinal: 0,
        };
        let effects = classify_transfer_log(&log, no_pools());
        assert!(effects.is_empty());
    }

    /// A Transfer event whose topics aren't the right length should
    /// not crash the helper — we just skip it.
    #[test]
    fn classify_transfer_log_with_short_topic() {
        let log = Log {
            address: vec![0x42u8; 20],
            topics: vec![
                TRANSFER_TOPIC.to_vec(),
                vec![0u8; 16], // truncated topic
                vec![0u8; 32],
            ],
            data: 1000u64.to_be_bytes().to_vec(),
            index: 0,
            block_index: 0,
            ordinal: 0,
        };
        let effects = classify_transfer_log(&log, no_pools());
        assert!(effects.is_empty(), "malformed topics must be silently dropped, not panic");
    }

    // ────────────────────────────────────────────────────────────────
    //  No-pool / unknown-token cases
    // ────────────────────────────────────────────────────────────────

    /// Transfer between two random EOAs that aren't pools → no effects.
    #[test]
    fn classify_transfer_between_non_pools_yields_no_effects() {
        let token = vec![0x11u8; 20];
        let log = make_transfer_log(token, [0xaau8; 20], [0xbbu8; 20], 1000, 5);
        let effects = classify_transfer_log(&log, no_pools());
        assert!(effects.is_empty());
    }

    /// Transfer FROM a known pool, but the moving token isn't in the
    /// pool's tracked tokens list. This is the airdrop case from H2:
    /// the helper must NOT emit an effect.
    #[test]
    fn classify_transfer_pool_with_unrelated_token_yields_no_effects() {
        let pool = [0xaau8; 20];
        let pool_hex = format!("0x{}", hex::encode(pool));
        let stranger_token = vec![0x99u8; 20]; // NOT in the pool's tokens
        let pool_token0 = vec![0x10u8; 20];
        let pool_token1 = vec![0x20u8; 20];

        let log = make_transfer_log(stranger_token, pool, [0xccu8; 20], 1000, 1);
        let effects = classify_transfer_log(
            &log,
            one_pool(pool_hex, vec![pool_token0, pool_token1]),
        );
        assert!(
            effects.is_empty(),
            "airdrop of an unrelated token to a tracked pool must NOT count as a balance delta"
        );
    }

    // ────────────────────────────────────────────────────────────────
    //  Single-side hits: positive (TO pool) and negative (FROM pool)
    // ────────────────────────────────────────────────────────────────

    /// Transfer TO a pool: the receiver must produce a single positive
    /// delta for the pool's token.
    #[test]
    fn classify_transfer_to_pool_emits_positive_delta() {
        let pool = [0xaau8; 20];
        let pool_hex = format!("0x{}", hex::encode(pool));
        let token = vec![0x10u8; 20];

        let log = make_transfer_log(token.clone(), [0x99u8; 20], pool, 1000, 7);
        let effects = classify_transfer_log(&log, one_pool(pool_hex.clone(), vec![token.clone()]));
        assert_eq!(effects.len(), 1);
        assert_eq!(effects[0].pool_id, pool_hex);
        assert_eq!(effects[0].token, token);
        assert_eq!(effects[0].delta, BigInt::from(1000));
        assert_eq!(effects[0].log_ordinal, 7);
    }

    /// Transfer FROM a pool: the sender must produce a single NEGATIVE
    /// delta for the pool's token.
    #[test]
    fn classify_transfer_from_pool_emits_negative_delta() {
        let pool = [0xaau8; 20];
        let pool_hex = format!("0x{}", hex::encode(pool));
        let token = vec![0x10u8; 20];

        let log = make_transfer_log(token.clone(), pool, [0x99u8; 20], 500, 3);
        let effects = classify_transfer_log(&log, one_pool(pool_hex.clone(), vec![token.clone()]));
        assert_eq!(effects.len(), 1);
        assert_eq!(effects[0].pool_id, pool_hex);
        assert_eq!(effects[0].token, token);
        assert_eq!(effects[0].delta, BigInt::from(-500));
        assert_eq!(effects[0].log_ordinal, 3);
    }

    // ────────────────────────────────────────────────────────────────
    //  Pool-to-pool transfer: both effects must be emitted
    // ────────────────────────────────────────────────────────────────

    /// A token transfer from one tracked pool directly to another
    /// (rare in practice, but possible — e.g. a flash-loan-driven
    /// rebalance). Both pools must record their respective deltas.
    #[test]
    fn classify_pool_to_pool_transfer_emits_both_effects() {
        let pool_a = [0xaau8; 20];
        let pool_b = [0xbbu8; 20];
        let token = vec![0x10u8; 20];
        let pool_a_hex = format!("0x{}", hex::encode(pool_a));
        let pool_b_hex = format!("0x{}", hex::encode(pool_b));

        // Custom lookup closure that knows about both pools.
        let lookup = {
            let a = pool_a_hex.clone();
            let b = pool_b_hex.clone();
            let t = token.clone();
            move |addr: &str| -> Option<Vec<Vec<u8>>> {
                if addr == a {
                    Some(vec![t.clone()])
                } else if addr == b {
                    Some(vec![t.clone()])
                } else {
                    None
                }
            }
        };

        let log = make_transfer_log(token.clone(), pool_a, pool_b, 12345, 99);
        let effects = classify_transfer_log(&log, lookup);
        assert_eq!(effects.len(), 2);

        // First effect: pool_a is FROM (negative)
        assert_eq!(effects[0].pool_id, pool_a_hex);
        assert_eq!(effects[0].delta, BigInt::from(-12345));

        // Second effect: pool_b is TO (positive)
        assert_eq!(effects[1].pool_id, pool_b_hex);
        assert_eq!(effects[1].delta, BigInt::from(12345));
    }

    // ────────────────────────────────────────────────────────────────
    //  The `0x` prefix bug from finding A5
    // ────────────────────────────────────────────────────────────────

    /// Pin the `0x` prefix bug from review finding A5: pool ids in the
    /// store are written as `0x{hex}` lowercased, so the lookup MUST
    /// be queried with the same prefix. This test checks that the
    /// helper passes the prefixed key to the closure — if a future
    /// refactor accidentally drops the prefix, the closure won't match
    /// and the test fails loudly.
    #[test]
    fn classify_transfer_uses_0x_prefixed_lowercase_hex_key() {
        use std::cell::RefCell;
        let pool = [0xaau8; 20];
        let token = vec![0x10u8; 20];
        let pool_hex_with_prefix = format!("0x{}", hex::encode(pool));

        // Build the log first so we don't have it borrowed when the closure runs.
        let log = make_transfer_log(token.clone(), pool, [0x99u8; 20], 1, 0);

        // Capture every key the closure was called with.
        let captured_keys = RefCell::new(Vec::<String>::new());
        {
            let token_for_closure = token.clone();
            let lookup = |addr: &str| -> Option<Vec<Vec<u8>>> {
                captured_keys.borrow_mut().push(addr.to_string());
                if addr == pool_hex_with_prefix {
                    Some(vec![token_for_closure.clone()])
                } else {
                    None
                }
            };
            let _ = classify_transfer_log(&log, lookup);
        }

        let keys = captured_keys.into_inner();
        assert!(!keys.is_empty(), "lookup should have been called at least once");
        for key in &keys {
            assert!(
                key.starts_with("0x"),
                "lookup must be queried with `0x`-prefixed key, got: {key}"
            );
            assert_eq!(
                *key,
                key.to_lowercase(),
                "lookup must be queried with lowercase hex, got: {key}"
            );
            assert_eq!(
                key.len(),
                42,
                "key must be `0x` + 40 hex chars, got len={}",
                key.len()
            );
        }
    }

    // ────────────────────────────────────────────────────────────────
    //  Amount edge cases
    // ────────────────────────────────────────────────────────────────

    /// Zero-amount transfer (legal under ERC20). Helper must still emit
    /// an effect, but with delta = 0. Aggregation downstream will treat
    /// this as a no-op, but emitting it preserves the audit trail.
    #[test]
    fn classify_zero_amount_transfer_still_emits() {
        let pool = [0xaau8; 20];
        let pool_hex = format!("0x{}", hex::encode(pool));
        let token = vec![0x10u8; 20];

        let log = make_transfer_log(token.clone(), [0x99u8; 20], pool, 0, 1);
        let effects = classify_transfer_log(&log, one_pool(pool_hex, vec![token]));
        assert_eq!(effects.len(), 1);
        assert_eq!(effects[0].delta, BigInt::from(0));
    }

    /// Very large amount (>2^63) that doesn't fit in a u64 — must
    /// be parsed as a BigInt without overflow.
    #[test]
    fn classify_large_amount_above_u64_max() {
        let pool = [0xaau8; 20];
        let pool_hex = format!("0x{}", hex::encode(pool));
        let token = vec![0x10u8; 20];

        // Max uint256 = 2^256 - 1, encoded as 32 bytes of 0xff.
        let mut from_topic = vec![0u8; 32];
        from_topic[12..32].copy_from_slice(&[0x99u8; 20]);
        let mut to_topic = vec![0u8; 32];
        to_topic[12..32].copy_from_slice(&pool);
        let log = Log {
            address: token.clone(),
            topics: vec![TRANSFER_TOPIC.to_vec(), from_topic, to_topic],
            data: vec![0xffu8; 32], // 2^256 - 1
            index: 0,
            block_index: 0,
            ordinal: 0,
        };

        let effects = classify_transfer_log(&log, one_pool(pool_hex, vec![token]));
        assert_eq!(effects.len(), 1);
        let max_uint256 = (BigInt::from(1) << 256) - 1;
        assert_eq!(effects[0].delta, max_uint256);
    }

    /// Token-membership check uses byte-exact equality. If the pool's
    /// token list contains a 19-byte address (malformed) and the log
    /// is for a 20-byte token, the membership check must fail.
    #[test]
    fn classify_token_membership_byte_exact() {
        let pool = [0xaau8; 20];
        let pool_hex = format!("0x{}", hex::encode(pool));
        let token_20 = vec![0x10u8; 20];
        let token_19 = vec![0x10u8; 19]; // wrong length

        let log = make_transfer_log(token_20, [0x99u8; 20], pool, 100, 0);
        let effects = classify_transfer_log(&log, one_pool(pool_hex, vec![token_19]));
        assert!(
            effects.is_empty(),
            "byte-length mismatch in token address must NOT match"
        );
    }
}
