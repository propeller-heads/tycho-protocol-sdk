use std::collections::HashMap;
use ethabi::ethereum_types::H256;
use substreams::hex;

// This is wrong, this is the address of Bancor Network V3
// Factories
pub const FACTORIES: [[u8; 20]; 1] = [
    hex!("eEF417e1D5CC832e619ae18D2F140De2999dD4fB"), // Ethereum
];

// 1. Trades are initited by calling "tradeBySourceAmount" or "tradeByTragetAmount" on the Bancor Network V3 Contract 
// Bancor Network V3 Contract address: 0xeEF417e1D5CC832e619ae18D2F140De2999dD4fB
// Bancor Network V3 Creation Tx: 0x6b1dfd6608d4ababbb50e25adb8498f318551cd305981fc0e950e51e0d3c6e7f
// Bancor Network V3 Creation Block: 14609379;
// 2. sourceAmount is sent to Master Vault
// Master Vault Contract address: 0x649765821D9f64198c905eC0B2B037a4a52Bc373
// Master Vault Creation Tx: 0x771fabdc9c778ea942b465cce5e64dea77dfd12f83c110431dab4c77563e6407
// Master Vault Creation Block: 14609331;
// 3. Master Vault send to trader the targetToken.


pub const NETWORK: [u8; 20] = hex!("eEF417e1D5CC832e619ae18D2F140De2999dD4fB");
pub const NETWORK_TX_CREATION: [u8; 32] = hex!("6b1dfd6608d4ababbb50e25adb8498f318551cd305981fc0e950e51e0d3c6e7f");
pub const NETWORK_BLOCK_CREATION: i32 = 14609379;

pub const MASTER_VAULT: [u8; 20] = hex!("649765821D9f64198c905eC0B2B037a4a52Bc373");
pub const MASTER_VAULT_TX_CREATION: [u8; 32] = hex!("771fabdc9c778ea942b465cce5e64dea77dfd12f83c110431dab4c77563e6407");
pub const MASTER_VAULT_BLOCK_CREATION: i32 = 14609331;

// Each Pool is paired with BNT, when swapping 2 non BNT tokens, it effectuate a multi-hop swap
// Example if swapping e.g. LINK for WBTC:
// 1. Swap LINK for BNT
// 2. Swap BNT for WBTC
pub const BNT: [u8; 20] = hex!("1F573D6Fb3F13d689FF844B4cE37794d79a7FF1C");
pub const BNT_TX_CREATION: [u8; 32] = hex!("750f6632f7884defd40b79be8174bbb44de9ef21ca4172a249d7a4fe10b45e8e");
pub const BNT_BLOCK_CREATION: i32 = 3851136; 

// Each token corresponds to a pool
pub const ETH: [u8; 20] = hex!("EeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE");
pub const LINK: [u8; 20] = hex!("514910771AF9Ca656af840dff83E8264EcF986CA");
pub const BAT: [u8; 20] = hex!("0D8775F648430679A709E98d2b0Cb6250d2887EF");
pub const ENJ: [u8; 20] = hex!("F629cBd94d3791C9250152BD8dfBDF380E2a3B9c");
pub const MATIC_OLD: [u8; 20] = hex!("7D1AfA7B718fb893dB30A3aBc0Cfc608AaCfeBB0");
pub const MKR: [u8; 20] = hex!("9f8F72aA9304c8B593d555F12eF6589cC3A579A2");
pub const QNT: [u8; 20] = hex!("4a220E6096B25EADb88358cb44068A3248254675");
pub const WBTC: [u8; 20] = hex!("2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599");
pub const W_NXM: [u8; 20] = hex!("0d438F3b5175Bebc262bF23753C1E53d03432bDE");
pub const DAI: [u8; 20] = hex!("85B9E00f820849BE1308A49eaDCFd44C74E3F001");

// Global and agnostic ERC20 signatures used to filter events (Transfer and Approval)
pub const ERC20_TRANSFER_SIG: [u8; 32] = hex!("ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef");
