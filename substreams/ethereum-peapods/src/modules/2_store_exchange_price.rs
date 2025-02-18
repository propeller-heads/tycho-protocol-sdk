use prost::Message;
use substreams::store::*;

use crate::pb::adapter::v1::{function_call::CallType, *};

use super::helper_funcs::{canonicalize_addresses, compute_exchange_price};

#[substreams::handlers::store]
pub fn store_exchange_price(func_calls: FunctionCalls, store: StoreSetBigInt) {
    // Map token pairs from function calls
    for call in func_calls.calls {
        match call.call_type {
            Some(CallType::Swap(swap_call_data)) => {
                let (token0, _) = canonicalize_addresses(
                    call.sell_token.encode_to_vec(),
                    call.buy_token.encode_to_vec(),
                );

                store.set(
                    0,
                    call.id,
                    &compute_exchange_price(
                        token0.clone(),
                        call.sell_token.encode_to_vec(),
                        swap_call_data
                            .result
                            .expect("No Trade result")
                            .marginal_price_after_trade,
                    ),
                );
            }
            Some(CallType::SwapToPrice(swap_to_price_call_data)) => {
                let (token0, _) = canonicalize_addresses(
                    call.sell_token.encode_to_vec(),
                    call.buy_token.encode_to_vec(),
                );

                store.set(
                    0,
                    call.id,
                    &compute_exchange_price(
                        token0.clone(),
                        call.sell_token.encode_to_vec(),
                        swap_to_price_call_data
                            .result
                            .expect("No Trade result")
                            .marginal_price_after_trade,
                    ),
                );
            }
            // Skip Price call because it is a view func and we can't get view func calls from the block
            // Some(CallType::Price(price_call_data)) => {
            //     let (token0, _) = canonicalize_addresses(
            //         call.sell_token.encode_to_vec(),
            //         call.buy_token.encode_to_vec(),
            //     );

            //     let last_price = price_call_data
            //         .prices
            //         .last()
            //         .expect("No price at the last index of prices");
            //     let last_specified_amount = price_call_data
            //         .specified_amounts
            //         .last()
            //         .expect("No specified amount at the last index of specified amounts");

            //     let denominator = U256::from_big_endian(&last_price.denominator);
            //     let specified_amount = U256::from_big_endian(last_specified_amount);

            //     // Perform multiplication safely with U256
            //     let new_denominator = denominator
            //         .checked_mul(specified_amount)
            //         .expect("Multiplication overflow");

            //     let actual_price = Fraction {
            //         numerator: last_price.numerator.clone(),
            //         denominator: {
            //             let mut result = vec![0u8; 32];
            //             new_denominator.to_big_endian(&mut result);
            //             result
            //         },
            //     };

            //     store.set(
            //         0,
            //         call.id,
            //         &compute_exchange_price(
            //             token0.clone(),
            //             call.sell_token.encode_to_vec(),
            //             actual_price,
            //         ),
            //     );
            // }
            _ => {}
        }
    }
}
