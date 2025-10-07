use crate::{
    abi::gsp::events::{BuyShares, DodoSwap, SellShares},
    modules::utils::fn_selector,
    pb::dodo::v2::Pool,
};
use anyhow::{Ok, Result};
use substreams::{prelude::StoreGet, store::StoreGetProto};
use substreams_ethereum::{
    pb::eth::v2::{Block, Log, TransactionTrace},
    Event,
};
use substreams_helper::hex::Hexable;
use tycho_substreams::prelude::*;

#[substreams::handlers::map]
pub fn map_relative_balances(
    block: Block,
    pools_store: StoreGetProto<Pool>,
) -> Result<BlockBalanceDeltas, anyhow::Error> {
    let mut balance_deltas = Vec::new();
    for trx in block.transactions() {
        let mut tx_deltas = Vec::new();
        for log in trx
            .calls
            .iter()
            .filter(|call| !call.state_reverted)
            .flat_map(|call| &call.logs)
        {
            if let Some(balance_deltas) = decode_event_balance_deltas(log, trx, &pools_store) {
                tx_deltas.extend(balance_deltas);
            }
        }
        if !tx_deltas.is_empty() {
            balance_deltas.extend(tx_deltas);
        }
    }
    Ok(BlockBalanceDeltas { balance_deltas })
}

/// Decodes a DODO pool event into a list of token balance changes (deltas).
///
/// This function processes different types of events (swap, buy shares, sell shares) from a DODO
/// pool, and converts them into `BalanceDelta` objects that represent token balance changes.
///
/// # Arguments
/// - `event`: The emitted log event from the DODO pool contract.
/// - `tx_trace`: The transaction trace containing call data (used for buy/sell shares events).
/// - `pools_store`: A store to fetch pool information (base/quote tokens, address, etc.).
///
/// # Returns
/// - `Some(Vec<BalanceDelta>)`: If the event is successfully decoded into balance changes.
/// - `None`: If the event is not recognized or if decoding fails.
///
/// # Notes
/// - For `Swap` events: The token deltas are derived directly from the event logs.
/// - For `BuyShares`/`SellShares` events: The deltas are extracted from the call return data.
fn decode_event_balance_deltas(
    event: &Log,
    tx_trace: &TransactionTrace,
    pools_store: &StoreGetProto<Pool>,
) -> Option<Vec<BalanceDelta>> {
    let tx = Transaction {
        to: tx_trace.to.clone(),
        from: tx_trace.from.clone(),
        hash: tx_trace.hash.clone(),
        index: tx_trace.index.into(),
    };

    let pool = pools_store.get_last(format!("Pool:{}", event.address.to_hex()))?;

    if let Some(swap) = DodoSwap::match_and_decode(event) {
        let (token_in, token_out, amount_in, amount_out) = if swap.from_token == pool.base_token {
            (&pool.base_token, &pool.quote_token, &swap.from_amount, &swap.to_amount)
        } else {
            (&pool.quote_token, &pool.base_token, &swap.from_amount, &swap.to_amount)
        };

        return Some(vec![
            BalanceDelta {
                ord: event.ordinal,
                tx: Some(tx.clone()),
                token: token_in.clone(),
                delta: amount_in.to_signed_bytes_be(),
                component_id: pool
                    .address
                    .to_hex()
                    .as_bytes()
                    .to_vec(),
            },
            BalanceDelta {
                ord: event.ordinal,
                tx: Some(tx.clone()),
                token: token_out.clone(),
                delta: amount_out.neg().to_signed_bytes_be(),
                component_id: pool
                    .address
                    .to_hex()
                    .as_bytes()
                    .to_vec(),
            },
        ]);
    }

    if BuyShares::match_and_decode(event).is_some() {
        let buy_share_selector = fn_selector("buyShares(address)");
        for call in &tx_trace.calls {
            if call.address == event.address &&
                call.input
                    .starts_with(&buy_share_selector)
            {
                let output_result = crate::abi::gsp::functions::BuyShares::output_call(call);
                if let Result::Ok((_, base_input, quote_input)) = output_result {
                    return Some(vec![
                        BalanceDelta {
                            ord: event.ordinal,
                            tx: Some(tx.clone()),
                            token: pool.base_token.clone(),
                            delta: base_input.to_signed_bytes_be(),
                            component_id: pool
                                .address
                                .to_hex()
                                .as_bytes()
                                .to_vec(),
                        },
                        BalanceDelta {
                            ord: event.ordinal,
                            tx: Some(tx.clone()),
                            token: pool.quote_token.clone(),
                            delta: quote_input.to_signed_bytes_be(),
                            component_id: pool
                                .address
                                .to_hex()
                                .as_bytes()
                                .to_vec(),
                        },
                    ]);
                }
            }
        }
    }

    if SellShares::match_and_decode(event).is_some() {
        let sell_share_selector =
            fn_selector("sellShares(uint256,address,uint256,uint256,bytes,uint256)");
        for call in &tx_trace.calls {
            if call.address == event.address &&
                call.input
                    .starts_with(&sell_share_selector)
            {
                let output_result = crate::abi::gsp::functions::SellShares::output_call(call);
                if let Result::Ok((base_output, quote_output)) = output_result {
                    return Some(vec![
                        BalanceDelta {
                            ord: event.ordinal,
                            tx: Some(tx.clone()),
                            token: pool.base_token.clone(),
                            delta: base_output.neg().to_signed_bytes_be(),
                            component_id: pool
                                .address
                                .to_hex()
                                .as_bytes()
                                .to_vec(),
                        },
                        BalanceDelta {
                            ord: event.ordinal,
                            tx: Some(tx.clone()),
                            token: pool.quote_token.clone(),
                            delta: quote_output.neg().to_signed_bytes_be(),
                            component_id: pool
                                .address
                                .to_hex()
                                .as_bytes()
                                .to_vec(),
                        },
                    ]);
                }
            }
        }
    }
    None
}
