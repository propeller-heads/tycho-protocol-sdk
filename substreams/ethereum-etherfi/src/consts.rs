use substreams::hex;

pub const ADDRESS_ZERO: [u8; 20] = hex!("0000000000000000000000000000000000000000");

// WEETH Creation Hasb
pub const WEETH_CREATION_HASH: [u8; 32] = hex!("a034bdf7ec3b407125fcfbb786d908b0bcfd9976f2fbaf489776ba58b9db61ac");
pub const EETH_CREATION_HASH: [u8; 32] = hex!("f6763c707b90b260bba114fce9a141aa4a923327539ded9d4d4ae4395b2200ff");
pub const LIQUIDITY_POOL_CREATION_HASH: [u8; 32] = hex!("491b823bc15ced4c54f0ed5a235d39e478f8aae3ad02eb553924b40ad9859e10");

// Liquidity Pools
pub const LIQUIDITY_POOL_ADDRESS: [u8; 20] = hex!("308861A430be4cce5502d0A12724771Fc6DaF216");

// Tokens
pub const EETH_ADDRESS: [u8; 20] = hex!("35fA164735182de50811E8e2E824cFb9B6118ac2");
pub const WEETH_ADDRESS: [u8; 20] = hex!("Cd5fE23C85820F7B72D0926FC9b05b43E359b7ee");
pub const ETH_ADDRESS: [u8; 20] = hex!("EeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE");
