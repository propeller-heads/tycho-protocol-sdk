use crate::abi::swap_adapter::functions::{Price, Swap2, SwapToPrice};
use crate::pb::adapter::v1::function_call::CallType;
use crate::pb::adapter::v1::*;
use substreams_ethereum::pb::eth;

use super::config::DeploymentConfig;
use super::helper_funcs::{canonicalize_addresses, format_address, key_from_tokens};

#[substreams::handlers::map]
pub fn map_function_calls(
    params: String, // adapter address in hex form
    block: eth::v2::Block,
) -> Result<FunctionCalls, anyhow::Error> {
    // Decode adapter address from parameters.
    let config: DeploymentConfig = serde_qs::from_str(params.as_str())?;
    let adapter_address = config.adapter_address;

    let mut func_calls = FunctionCalls { calls: vec![] };

    for tx in block.transactions() {
        // Iterate over the internal calls in the transaction.
        let fn_calls: Vec<FunctionCall> = tx
            .calls()
            // Filter calls directed to our adapter contract.
            .filter(|call| call.transaction.to == adapter_address)
            .filter(|call| !call.call.state_reverted)
            .filter_map(|call| {
                // Depending on the function selector, identify the call type.
                if Swap2::match_call(call.call) {
                    let swap_call = Swap2::decode(call.call).unwrap();
                    let result = Swap2::output_call(call.call).unwrap();

                    let price = result.2 .0.to_string();
                    let trade = Trade {
                        calculated_amount: result.0.to_string(),
                        gas_used: result.1.to_string(),
                        marginal_price_after_trade: price,
                    };

                    let (addr0, addr1) = canonicalize_addresses(
                        swap_call.sell_token.clone(),
                        swap_call.buy_token.clone(),
                    );
                    let call_id = key_from_tokens(&addr0, &addr1);
                    let sell_token_addr = format_address(&swap_call.sell_token);
                    let buy_token_addr = format_address(&swap_call.buy_token);

                    Some(FunctionCall {
                        id: call_id,
                        sell_token: sell_token_addr,
                        buy_token: buy_token_addr,
                        transaction: Some(Transaction {
                            hash: call.transaction.hash.clone(),
                            from: call.transaction.from.clone(),
                            to: call.transaction.to.clone(),
                            index: call.transaction.index,
                        }),
                        call_type: Some(CallType::Swap(SwapCallData {
                            side: swap_call.side.to_i32(),
                            specified_amount: swap_call.specified_amount.to_u64(),
                            result: Some(trade),
                        })),
                    })
                } else if SwapToPrice::match_call(&call.call) {
                    let swap_to_price_call = SwapToPrice::decode(call.call).unwrap();
                    let result = SwapToPrice::output_call(call.call).unwrap();
                    let price = result.2 .0.to_string();
                    let trade = Trade {
                        calculated_amount: result.0.to_string(),
                        gas_used: result.1.to_string(),
                        marginal_price_after_trade: price,
                    };

                    let (addr0, addr1) = canonicalize_addresses(
                        swap_to_price_call.sell_token.clone(),
                        swap_to_price_call.buy_token.clone(),
                    );

                    let call_id = key_from_tokens(&addr0, &addr1);
                    let sell_token_addr = format_address(&swap_to_price_call.sell_token);
                    let buy_token_addr = format_address(&swap_to_price_call.buy_token);

                    Some(FunctionCall {
                        id: call_id,
                        sell_token: sell_token_addr,
                        buy_token: buy_token_addr,
                        transaction: Some(Transaction {
                            hash: call.transaction.hash.clone(),
                            from: call.transaction.from.clone(),
                            to: call.transaction.to.clone(),
                            index: call.transaction.index,
                        }),
                        call_type: Some(CallType::SwapToPrice(SwapToPriceCallData {
                            limit_price: swap_to_price_call
                                .limit_price
                                .0
                                .to_string(),
                            result: Some(trade),
                        })),
                    })
                } else if Price::match_call(&call.call) {
                    let price_call = Price::decode(call.call).unwrap();
                    let result = Price::output_call(call.call).unwrap();

                    let specified_amounts = price_call
                        .specified_amounts
                        .iter()
                        .map(|amount| amount.to_bytes_be().1)
                        .collect();
                    let prices = result
                        .iter()
                        .map(|price| price.0.to_string())
                        .collect();

                    let (addr0, addr1) = canonicalize_addresses(
                        price_call.sell_token.clone(),
                        price_call.buy_token.clone(),
                    );
                    let call_id = key_from_tokens(&addr0, &addr1);
                    let sell_token_addr = format_address(&price_call.sell_token);
                    let buy_token_addr = format_address(&price_call.buy_token);

                    Some(FunctionCall {
                        id: call_id,
                        sell_token: sell_token_addr,
                        buy_token: buy_token_addr,
                        transaction: Some(Transaction {
                            hash: call.transaction.hash.clone(),
                            from: call.transaction.from.clone(),
                            to: call.transaction.to.clone(),
                            index: call.transaction.index,
                        }),
                        call_type: Some(CallType::Price(PriceCallData {
                            specified_amounts,
                            prices,
                        })),
                    })
                } else {
                    None
                }
            })
            .collect();

        func_calls.calls.extend(fn_calls);
    }

    Ok(func_calls)
}
