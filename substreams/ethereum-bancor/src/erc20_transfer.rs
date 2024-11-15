use crate::abi::erc20::events::Transfer;
use substreams::scalar::BigInt as ScalarBigInt;
use substreams_ethereum::pb::eth;
use hex_literal::hex;

pub const ERC20_TRANSFER_SIG: [u8; 32] = hex!("ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef");

pub fn is_erc20_transfer(log: &eth::v2::Log) -> bool {
    log
        .topics
        .get(0)
        .map_or(false, |topic| *topic == ERC20_TRANSFER_SIG)
}

pub fn decode_erc20_transfer(log: &eth::v2::Log) -> Option<Transfer> {
    if !is_erc20_transfer(log) {
        return None;
    }

    let log_amount = log
        .data
        .get(0..32)
        .map(|slice| ScalarBigInt::from_signed_bytes_be(slice))
        .unwrap_or_else(|| ScalarBigInt::zero());

    Some(Transfer {
        from: log.topics.get(1).cloned().unwrap_or_default(),
        to: log.topics.get(2).cloned().unwrap_or_default(),
        value: log_amount,
    })
}

