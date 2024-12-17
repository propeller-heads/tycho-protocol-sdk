use alloy_primitives::{aliases::I24, Uint, I256, U160, U256};
use num_bigint::BigInt;
use std::error::Error;
use uniswap_v3_sdk::{
    prelude::*,
    utils::{SqrtPriceMath, TickMath},
};

/// Calculates the amounts of token0 and token1 for a given position
///
/// Source: https://github.com/Uniswap/v4-core/blob/main/src/libraries/Pool.sol
/// Function: modifyLiquidity
///
/// # Arguments
/// * `tick_lower` - Lower tick of the position
/// * `tick_upper` - Upper tick of the position
/// * `liquidity_delta` - Amount of liquidity to add/remove
/// * `sqrt_price_x96` - Current square root price
///
/// # Returns
/// * `Result<(U256, U256), Box<dyn Error>>` - Token amounts (amount0, amount1)
pub fn calculate_token_amounts(
    current_tick: i64,
    tick_lower: i32,
    tick_upper: i32,
    liquidity_delta: i128,
) -> Result<(BigInt, BigInt), Box<dyn Error>> {
    // Convert ticks to square root prices
    let tick_lower = I24::try_from(tick_lower)?;
    let tick_upper = I24::try_from(tick_upper)?;
    let current_tick = I24::try_from(current_tick)?;

    let sqrt_price_x96: U160 = TickMath::get_sqrt_ratio_at_tick(current_tick)?;
    let sqrt_price_lower_x96: U160 = TickMath::get_sqrt_ratio_at_tick(tick_lower)?;
    let sqrt_price_upper_x96: U160 = TickMath::get_sqrt_ratio_at_tick(tick_upper)?;

    // Calculate amounts based on current price relative to the range
    let (amount0, amount1) = if sqrt_price_x96 <= sqrt_price_lower_x96 {
        // Current price is below the range: position in token0
        let amount0 = SqrtPriceMath::get_amount_0_delta_signed(
            sqrt_price_lower_x96,
            sqrt_price_upper_x96,
            liquidity_delta,
        )?;
        (amount0, I256::from_big_int(BigInt::from(0)))
    } else if sqrt_price_x96 < sqrt_price_upper_x96 {
        // Current price is within the range: position in both tokens
        let amount0 = SqrtPriceMath::get_amount_0_delta_signed(
            sqrt_price_x96,
            sqrt_price_upper_x96,
            liquidity_delta,
        )?;

        let amount1 = SqrtPriceMath::get_amount_1_delta_signed(
            sqrt_price_lower_x96,
            sqrt_price_x96,
            liquidity_delta,
        )?;

        (amount0, amount1)
    } else {
        // Current price is above the range: position in token1
        let amount1 = SqrtPriceMath::get_amount_1_delta_signed(
            sqrt_price_lower_x96,
            sqrt_price_upper_x96,
            liquidity_delta,
        )?;

        (I256::from_big_int(BigInt::from(0)), amount1)
    };

    Ok((amount0.to_big_int(), amount1.to_big_int()))
}
