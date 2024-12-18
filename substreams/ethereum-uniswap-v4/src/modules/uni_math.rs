use num_bigint::BigInt;
use std::{error::Error, fmt, ops::Shr};

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
    current_tick: i32,
    tick_lower: i32,
    tick_upper: i32,
    liquidity_delta: i128,
) -> Result<(BigInt, BigInt), Box<dyn Error>> {
    let sqrt_price_x96: BigInt = get_sqrt_ratio_at_tick(current_tick)?;
    let sqrt_price_lower_x96: BigInt = get_sqrt_ratio_at_tick(tick_lower)?;
    let sqrt_price_upper_x96: BigInt = get_sqrt_ratio_at_tick(tick_upper)?;

    // Calculate amounts based on current price relative to the range
    let (amount0, amount1) = if sqrt_price_x96 <= sqrt_price_lower_x96 {
        // Current price is below the range: position in token0
        let amount0 =
            get_amount_0_delta_signed(sqrt_price_lower_x96, sqrt_price_upper_x96, liquidity_delta)?;
        (amount0, BigInt::from(0))
    } else if sqrt_price_x96 < sqrt_price_upper_x96 {
        // Current price is within the range: position in both tokens
        let amount0 = get_amount_0_delta_signed(
            sqrt_price_x96.clone(),
            sqrt_price_upper_x96,
            liquidity_delta,
        )?;

        let amount1 =
            get_amount_1_delta_signed(sqrt_price_lower_x96, sqrt_price_x96, liquidity_delta)?;

        (amount0, amount1)
    } else {
        // Current price is above the range: position in token1
        let amount1 =
            get_amount_1_delta_signed(sqrt_price_lower_x96, sqrt_price_upper_x96, liquidity_delta)?;

        (BigInt::from(0), amount1)
    };

    Ok((amount0, amount1))
}

const MAX_TICK: i32 = 887272;

#[derive(Debug)]
pub enum MathError {
    InvalidTick(i32),
    InvalidPrice,
}

// Implement Display trait
impl fmt::Display for MathError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MathError::InvalidTick(tick) => write!(f, "Invalid tick value: {}", tick),
            MathError::InvalidPrice => write!(f, "Invalid price"),
        }
    }
}

// Implement Error trait
impl Error for MathError {}

/// Returns the sqrt ratio as a Q64.96 for the given tick. The sqrt ratio is computed as
/// sqrt(1.0001)^tick
/// Adapted from: https://github.com/shuhuiluo/uniswap-v3-sdk-rs/blob/v2.9.1/src/utils/tick_math.rs#L57
fn get_sqrt_ratio_at_tick(tick: i32) -> Result<BigInt, MathError> {
    let abs_tick = tick.abs();

    if abs_tick > MAX_TICK {
        return Err(MathError::InvalidTick(tick));
    }

    // Initialize ratio with either 2^128 / sqrt(1.0001) or 2^128
    let mut ratio = if abs_tick & 0x1 != 0 {
        BigInt::parse_bytes(b"0xfffcb933bd6fad37aa2d162d1a594001", 16).unwrap()
    } else {
        BigInt::from(1) << 128
    };

    if abs_tick & 0x2 != 0 {
        ratio = (&ratio * BigInt::parse_bytes(b"0xfff97272373d413259a46990580e213a", 16).unwrap())
            .shr(128);
    }
    if abs_tick & 0x4 != 0 {
        ratio = (&ratio * BigInt::parse_bytes(b"0xfff2e50f5f656932ef12357cf3c7fdcc", 16).unwrap())
            .shr(128);
    }
    if abs_tick & 0x8 != 0 {
        ratio = (&ratio * BigInt::parse_bytes(b"0xffe5caca7e10e4e61c3624eaa0941cd0", 16).unwrap())
            .shr(128);
    }
    if abs_tick & 0x10 != 0 {
        ratio = (&ratio * BigInt::parse_bytes(b"0xffcb9843d60f6159c9db58835c926644", 16).unwrap())
            .shr(128);
    }
    if abs_tick & 0x20 != 0 {
        ratio = (&ratio * BigInt::parse_bytes(b"0xff973b41fa98c081472e6896dfb254c0", 16).unwrap())
            .shr(128);
    }
    if abs_tick & 0x40 != 0 {
        ratio = (&ratio * BigInt::parse_bytes(b"0xff2ea16466c96a3843ec78b326b52861", 16).unwrap())
            .shr(128);
    }
    if abs_tick & 0x80 != 0 {
        ratio = (&ratio * BigInt::parse_bytes(b"0xfe5dee046a99a2a811c461f1969c3053", 16).unwrap())
            .shr(128);
    }
    if abs_tick & 0x100 != 0 {
        ratio = (&ratio * BigInt::parse_bytes(b"0xfcbe86c7900a88aedcffc83b479aa3a4", 16).unwrap())
            .shr(128);
    }
    if abs_tick & 0x200 != 0 {
        ratio = (&ratio * BigInt::parse_bytes(b"0xf987a7253ac413176f2b074cf7815e54", 16).unwrap())
            .shr(128);
    }
    if abs_tick & 0x400 != 0 {
        ratio = (&ratio * BigInt::parse_bytes(b"0xf3392b0822b70005940c7a398e4b70f3", 16).unwrap())
            .shr(128);
    }
    if abs_tick & 0x800 != 0 {
        ratio = (&ratio * BigInt::parse_bytes(b"0xe7159475a2c29b7443b29c7fa6e889d9", 16).unwrap())
            .shr(128);
    }
    if abs_tick & 0x1000 != 0 {
        ratio = (&ratio * BigInt::parse_bytes(b"0xd097f3bdfd2022b8845ad8f792aa5825", 16).unwrap())
            .shr(128);
    }
    if abs_tick & 0x2000 != 0 {
        ratio = (&ratio * BigInt::parse_bytes(b"0xa9f746462d870fdf8a65dc1f90e061e5", 16).unwrap())
            .shr(128);
    }
    if abs_tick & 0x4000 != 0 {
        ratio = (&ratio * BigInt::parse_bytes(b"0x70d869a156d2a1b890bb3df62baf32f7", 16).unwrap())
            .shr(128);
    }
    if abs_tick & 0x8000 != 0 {
        ratio = (&ratio * BigInt::parse_bytes(b"0x31be135f97d08fd981231505542fcfa6", 16).unwrap())
            .shr(128);
    }
    if abs_tick & 0x10000 != 0 {
        ratio = (&ratio * BigInt::parse_bytes(b"0x9aa508b5b7a84e1c677de54f3e99bc9", 16).unwrap())
            .shr(128);
    }
    if abs_tick & 0x20000 != 0 {
        ratio = (&ratio * BigInt::parse_bytes(b"0x5d6af8dedb81196699c329225ee604", 16).unwrap())
            .shr(128);
    }
    if abs_tick & 0x40000 != 0 {
        ratio =
            (&ratio * BigInt::parse_bytes(b"0x2216e584f5fa1ea926041bedfe98", 16).unwrap()).shr(128);
    }
    if abs_tick & 0x80000 != 0 {
        ratio = (&ratio * BigInt::parse_bytes(b"0x48a170391f7dc42444e8fa2", 16).unwrap()).shr(128);
    }

    if tick > 0 {
        let max = (BigInt::from(1) << 256) - 1;
        ratio = max / ratio;
    }
    // Add 2^32 - 1 and shift right by 32
    ratio = (ratio + ((BigInt::from(1) << 32) - 1)) >> 32;

    Ok(ratio)
}

/// Helper that gets signed token0 delta
/// Source: https://github.com/shuhuiluo/uniswap-v3-sdk-rs/blob/v2.9.1/src/utils/sqrt_price_math.rs#L422
///
/// ## Arguments
///
/// * `sqrt_ratio_a_x96`: A sqrt price
/// * `sqrt_ratio_b_x96`: Another sqrt price
/// * `liquidity`: The change in liquidity for which to compute the amount0 delta
///
/// ## Returns
///
/// Amount of token0 corresponding to the passed liquidityDelta between the two prices
fn get_amount_0_delta_signed(
    sqrt_ratio_a_x96: BigInt,
    sqrt_ratio_b_x96: BigInt,
    liquidity: i128,
) -> Result<BigInt, MathError> {
    let sign = !liquidity.is_negative();
    // Create mask for negative numbers
    let mask = if sign { 0u128 } else { u128::MAX };

    // Get absolute value of liquidity using XOR and addition
    let liquidity = (mask ^ mask.wrapping_add_signed(liquidity)) as u128;

    // Convert mask to BigInt (all 1s or all 0s)
    let mask = if sign { BigInt::from(0) } else { -BigInt::from(1) };

    let amount_0 = get_amount_0_delta(sqrt_ratio_a_x96, sqrt_ratio_b_x96, liquidity, sign)?;

    // Apply the mask using XOR and subtraction to restore the sign
    Ok((amount_0 ^ &mask) - mask)
}

/// Gets the amount0 delta between two prices
///
/// Calculates liquidity / sqrt(lower) - liquidity / sqrt(upper),
/// i.e. liquidity * (sqrt(upper) - sqrt(lower)) / (sqrt(upper) * sqrt(lower))
///
/// ## Arguments
///
/// * `sqrt_ratio_a_x96`: A sqrt price assumed to be lower otherwise swapped
/// * `sqrt_ratio_b_x96`: Another sqrt price
/// * `liquidity`: The amount of usable liquidity
/// * `round_up`: Whether to round the amount up or down
///
/// ## Returns
///
/// Amount of token0 required to cover a position of size liquidity between the two passed prices
fn get_amount_0_delta(
    sqrt_ratio_a_x96: BigInt,
    sqrt_ratio_b_x96: BigInt,
    liquidity: u128,
    round_up: bool,
) -> Result<BigInt, MathError> {
    let (sqrt_ratio_a_x96, sqrt_ratio_b_x96) = if sqrt_ratio_a_x96 < sqrt_ratio_b_x96 {
        (sqrt_ratio_a_x96, sqrt_ratio_b_x96)
    } else {
        (sqrt_ratio_b_x96, sqrt_ratio_a_x96)
    };

    if sqrt_ratio_a_x96 == BigInt::from(0) {
        return Err(MathError::InvalidPrice);
    }

    let numerator_1 = BigInt::from(liquidity) << 96;
    let numerator_2 = &sqrt_ratio_b_x96 - &sqrt_ratio_a_x96;

    if round_up {
        // For rounding up: ceil(ceil(numerator_1 * numerator_2 / sqrt_ratio_b_x96) /
        // sqrt_ratio_a_x96)
        let temp =
            (&numerator_1 * &numerator_2 + &sqrt_ratio_b_x96 - BigInt::from(1)) / &sqrt_ratio_b_x96;
        Ok((&temp + &sqrt_ratio_a_x96 - BigInt::from(1)) / sqrt_ratio_a_x96)
    } else {
        // For rounding down: floor(floor(numerator_1 * numerator_2 / sqrt_ratio_b_x96) /
        // sqrt_ratio_a_x96)
        Ok((&numerator_1 * &numerator_2) / &sqrt_ratio_b_x96 / sqrt_ratio_a_x96)
    }
}

const Q96: u128 = 1 << 96;

/// Helper that gets signed token1 delta
///
/// ## Arguments
///
/// * `sqrt_ratio_a_x96`: A sqrt price
/// * `sqrt_ratio_b_x96`: Another sqrt price
/// * `liquidity`: The change in liquidity for which to compute the amount1 delta
///
/// ## Returns
///
/// Amount of token1 corresponding to the passed liquidityDelta between the two prices
fn get_amount_1_delta_signed(
    sqrt_ratio_a_x96: BigInt,
    sqrt_ratio_b_x96: BigInt,
    liquidity: i128,
) -> Result<BigInt, MathError> {
    let sign = !liquidity.is_negative();

    // Create mask for negative numbers
    let mask = if sign { 0u128 } else { u128::MAX };

    // Get absolute value of liquidity using XOR and addition
    let liquidity = (mask ^ mask.wrapping_add_signed(liquidity)) as u128;

    // Convert mask to BigInt (all 1s or all 0s)
    let mask = if sign { BigInt::from(0) } else { -BigInt::from(1) };

    let amount_1 = get_amount_1_delta(sqrt_ratio_a_x96, sqrt_ratio_b_x96, liquidity, sign)?;

    // Apply the mask using XOR and subtraction to restore the sign
    Ok((amount_1 ^ &mask) - mask)
}

/// Gets the amount1 delta between two prices
///
/// Calculates liquidity * (sqrt(upper) - sqrt(lower))
///
/// ## Arguments
///
/// * `sqrt_ratio_a_x96`: A sqrt price assumed to be lower otherwise swapped
/// * `sqrt_ratio_b_x96`: Another sqrt price
/// * `liquidity`: The amount of usable liquidity
/// * `round_up`: Whether to round the amount up, or down
///
/// ## Returns
///
/// Amount of token1 required to cover a position of size liquidity between the two passed prices
fn get_amount_1_delta(
    sqrt_ratio_a_x96: BigInt,
    sqrt_ratio_b_x96: BigInt,
    liquidity: u128,
    round_up: bool,
) -> Result<BigInt, MathError> {
    let (sqrt_ratio_a_x96, sqrt_ratio_b_x96) = if sqrt_ratio_a_x96 < sqrt_ratio_b_x96 {
        (sqrt_ratio_a_x96, sqrt_ratio_b_x96)
    } else {
        (sqrt_ratio_b_x96, sqrt_ratio_a_x96)
    };

    let numerator = &sqrt_ratio_b_x96 - &sqrt_ratio_a_x96;
    let denominator = BigInt::from(Q96);

    let liquidity = BigInt::from(liquidity);
    let amount_1 = &liquidity * &numerator / &denominator;

    // Calculate if there's a remainder
    let remainder = (&liquidity * &numerator) % &denominator;
    let carry = remainder > BigInt::from(0) && round_up;

    Ok(if carry { amount_1 + 1 } else { amount_1 })
}
