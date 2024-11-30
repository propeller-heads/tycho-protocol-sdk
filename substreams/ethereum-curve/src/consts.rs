use substreams::hex;

// Registries
pub const META_REGISTRY: [u8; 20] = hex!("F98B45FA17DE75FB1aD0e7aFD971b0ca00e379fC");

// Factories
pub const CRYPTO_POOL_FACTORY: [u8; 20] = hex!("F18056Bbd320E96A48e3Fbf8bC061322531aac99");
pub const META_POOL_FACTORY: [u8; 20] = hex!("B9fC157394Af804a3578134A6585C0dc9cc990d4");
// pub const META_POOL_FACTORY_OLD: [u8; 20] = hex!("0959158b6040D32d04c301A72CBFD6b39E21c9AE");
pub const CRYPTO_SWAP_NG_FACTORY: [u8; 20] = hex!("6A8cbed756804B16E05E741eDaBd5cB544AE21bf");
pub const TRICRYPTO_FACTORY: [u8; 20] = hex!("0c0e5f2fF0ff18a3be9b835635039256dC4B4963");
pub const TWOCRYPTO_FACTORY: [u8; 20] = hex!("98ee851a00abee0d95d08cf4ca2bdce32aeaaf7f");
pub const STABLESWAP_FACTORY: [u8; 20] = hex!("4F8846Ae9380B90d2E71D5e3D042dff3E7ebb40d");

// Important addresses
pub const WETH_ADDRESS: [u8; 20] = hex!("C02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2");
pub const ETH_ADDRESS: [u8; 20] = hex!("EeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE");
pub const OLD_SUSD: [u8; 20] = hex!("57Ab1E02fEE23774580C119740129eAC7081e9D3");
pub const NEW_SUSD: [u8; 20] = hex!("57ab1ec28d129707052df4df418d58a2d46d5f51");
pub const TRICRYPTO_2_LP: [u8; 20] = hex!("c4ad29ba4b3c580e6d59105fff484999997675ff");
pub const TRICRYPTO_2_MATH_CONTRACT: [u8; 20] = hex!("40745803c2faa8e8402e2ae935933d07ca8f355c");

pub const CONTRACTS_TO_INDEX: [[u8; 20]; 5] = [
    CRYPTO_SWAP_NG_FACTORY,
    TRICRYPTO_FACTORY,
    TRICRYPTO_2_LP,
    TRICRYPTO_2_MATH_CONTRACT,
    TWOCRYPTO_FACTORY,
];
