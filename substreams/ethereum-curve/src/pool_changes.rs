use substreams::{
    scalar::BigInt,
    store::{StoreGet, StoreGetString},
};
use substreams_ethereum::{pb::eth::v2::TransactionTrace, Event};
use tycho_substreams::prelude::*;

use crate::{
    abi,
    consts::{ETH_ADDRESS, WETH_ADDRESS},
};

fn get_pool_tokens(pool_address: &Vec<u8>, tokens_store: &StoreGetString) -> Option<Vec<String>> {
    let pool_key = format!("pool:{}", hex::encode(pool_address));
    Some(
        tokens_store
            .get_last(pool_key)?
            .split(":")
            .map(|token| token.to_owned())
            .collect::<Vec<_>>(),
    )
}

/// Tracks `Transfers` in and out of tracked pools if it matches the specific tokens.
pub fn emit_deltas(tx: &TransactionTrace, tokens_store: &StoreGetString) -> Vec<BalanceDelta> {
    tx.logs_with_calls()
        .filter_map(|(log, _)| {
            let transfer = abi::erc20::events::Transfer::match_and_decode(log)?;
            let (component_id, pool_tokens, is_incoming) =
                if let Some(pool_tokens) = get_pool_tokens(&transfer.to, tokens_store) {
                    (hex::encode(&transfer.to), pool_tokens, true)
                } else if let Some(pool_tokens) = get_pool_tokens(&transfer.from, tokens_store) {
                    (hex::encode(&transfer.from), pool_tokens, false)
                } else {
                    return None;
                };

            let token_id = hex::encode(log.address.clone());
            if pool_tokens.contains(&token_id) {
                let delta = if is_incoming { transfer.value } else { transfer.value * -1 };
                Some(BalanceDelta {
                    ord: log.ordinal,
                    tx: Some(Transaction {
                        to: tx.to.clone(),
                        from: tx.from.clone(),
                        hash: tx.hash.clone(),
                        index: tx.index.into(),
                    }),
                    token: hex::decode(token_id).unwrap(),
                    delta: delta.to_signed_bytes_be(),
                    component_id: component_id.into(),
                })
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
}

/// Tracks ETH balance changes in and out of tracked pools if it matches the specific tokens.
/// Note: Pools might report as WETH or ETH. Some pools might even accept either WETH or ETH and
///  convert them on the fly (checkout pools with `WETHOptimized` in the name). It's a bit tricky
///  to figure this stuff out on the fly, but our rule of thumb is as follows:
///  - If a pool reports ETH in the `pool_tokens`, we use the fake ETH erc20 address.
///  - If a pool reports WETH, we report with the WETH erc20 address.
///  - If neither, it's likely an erroneous ETH transactions that many older pools don't reject.
pub fn emit_eth_deltas(tx: &TransactionTrace, tokens_store: &StoreGetString) -> Vec<BalanceDelta> {
    tx.calls()
        .flat_map(|call| {
            call.call
                .balance_changes
                .iter()
                .filter_map(|balance_change| {
                    if let Some(pool_tokens) =
                        get_pool_tokens(&balance_change.address, tokens_store)
                    {
                        let token = if pool_tokens.contains(&hex::encode(ETH_ADDRESS)) {
                            ETH_ADDRESS.to_vec()
                        } else if pool_tokens.contains(&hex::encode(WETH_ADDRESS)) {
                            WETH_ADDRESS.to_vec()
                        } else {
                            // The pool that was matched to the call doesn't contain either ETH
                            //  or WETH so found eth balance changes are erroneous.
                            return None;
                        };

                        // We need to convert to the usable `BigInt` type to be able to calculate
                        //  subtraction. This is seemingly the easiest way to do this.
                        let delta = BigInt::from_unsigned_bytes_be(
                            &balance_change
                                .new_value
                                .clone()
                                .unwrap_or_default()
                                .bytes,
                        ) - BigInt::from_unsigned_bytes_be(
                            &balance_change
                                .old_value
                                .clone()
                                .unwrap_or_default()
                                .bytes,
                        );
                        Some(BalanceDelta {
                            ord: call.call.end_ordinal,
                            tx: Some(Transaction {
                                to: tx.to.clone(),
                                from: tx.from.clone(),
                                hash: tx.hash.clone(),
                                index: tx.index.into(),
                            }),
                            token,
                            delta: delta.to_signed_bytes_be(),
                            component_id: hex::encode(balance_change.address.clone()).into(),
                        })
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>()
}
