use substreams::hex;

// Transmuters
pub const TRANSMUTERS_EUR: Vec<[u8; 20]> = [
    hex!("00253582b2a3FE112feEC532221d9708c64cEFAb"), // Ethereum
    hex!("00253582b2a3FE112feEC532221d9708c64cEFAb"), // Arbitrum
];
pub const TRANSMUTERS_USD: Vec<[u8; 20]> = [
    hex!("222222fD79264BBE280b4986F6FEfBC3524d0137"),
    hex!("222222fD79264BBE280b4986F6FEfBC3524d0137"),
];

// agTokens
pub const AGTOKENS_EUR: Vec<[u8; 20]> = [
    hex!("1a7e4e63778B4f12a199C062f3eFdD288afCBce8"),
    hex!("1a7e4e63778B4f12a199C062f3eFdD288afCBce8"),
];
pub const AGTOKENS_USD: Vec<[u8; 20]> = [
    hex!("0000206329b97DB379d5E1Bf586BbDB969C63274"),
    hex!("0000206329b97DB379d5E1Bf586BbDB969C63274"),
];
