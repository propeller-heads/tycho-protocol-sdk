use std::str::FromStr;

use substreams::scalar::BigInt;

// --- Helper Functions ---
/// Returns the two addresses in a canonical order.
pub fn canonicalize_addresses(addr1: Vec<u8>, addr2: Vec<u8>) -> (Vec<u8>, Vec<u8>) {
    // Using the default lexicographical order
    if addr1 < addr2 {
        (addr1, addr2)
    } else {
        (addr2, addr1)
    }
}

// requires canonicalized addresses
pub fn key_from_tokens(addr1: &Vec<u8>, addr2: &Vec<u8>) -> String {
    format!("pool:0x{:}:0x{:}", hex::encode(addr1), hex::encode(addr2))
}

pub fn format_address(addr: &Vec<u8>) -> String {
    format!("0x{:}", hex::encode(addr))
}

pub fn compute_exchange_price(token0: Vec<u8>, sell_token: Vec<u8>, price: String) -> BigInt {
    if token0 == sell_token {
        BigInt::from_str(&price).unwrap()
    } else {
        BigInt::from(1) / BigInt::from_str(&price).unwrap()
    }
}
