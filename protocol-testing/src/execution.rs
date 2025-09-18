//! Swap execution simulation module.
//!
//! This module provides functionality to simulate executing swaps through RPC requests
//! using state overwrites and historical blockchain data. It allows testing swap execution
//! against specific block states without actually performing on-chain transactions.

use std::{collections::HashMap, path::PathBuf, str::FromStr, sync::LazyLock};

use alloy::{
    primitives::{Address, U256},
    rpc::types::{Block, TransactionRequest},
};
use miette::{miette, IntoDiagnostic, WrapErr};
use num_bigint::BigUint;
use serde_json::Value;
use tracing::info;
use tycho_simulation::{
    evm::protocol::u256_num::u256_to_biguint, tycho_execution::encoding::models::Solution,
};

use crate::rpc::RPCProvider;

/// Mapping from protocol component patterns to executor bytecode files
static EXECUTOR_MAPPING: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert("uniswap_v2", "UniswapV2.runtime.json");
    map.insert("sushiswap", "UniswapV2.runtime.json");
    map.insert("pancakeswap_v2", "UniswapV2.runtime.json");
    map.insert("uniswap_v3", "UniswapV3.runtime.json");
    map.insert("pancakeswap_v3", "UniswapV3.runtime.json");
    map.insert("uniswap_v4", "UniswapV4.runtime.json");
    map.insert("balancer_v2", "BalancerV2.runtime.json");
    map.insert("balancer_v3", "BalancerV3.runtime.json");
    map.insert("curve", "Curve.runtime.json");
    map.insert("maverick_v2", "MaverickV2.runtime.json");
    map
});

/// Executor addresses loaded from test_executor_addresses.json at startup
pub static EXECUTOR_ADDRESSES: LazyLock<HashMap<String, Address>> = LazyLock::new(|| {
    let executor_addresses_path = PathBuf::from("test_executor_addresses.json");

    let json_content = std::fs::read_to_string(&executor_addresses_path)
        .expect("Failed to read test_executor_addresses.json");

    let json_value: Value =
        serde_json::from_str(&json_content).expect("Failed to parse test_executor_addresses.json");

    let ethereum_addresses = json_value["ethereum"]
        .as_object()
        .expect("Missing 'ethereum' key in test_executor_addresses.json");

    let mut addresses = HashMap::new();
    for (protocol_name, address_value) in ethereum_addresses {
        let address_str = address_value
            .as_str()
            .unwrap_or_else(|| panic!("Invalid address format for protocol '{protocol_name}'"));

        let address = Address::from_str(address_str).unwrap_or_else(|_| {
            panic!("Invalid address '{address_str}' for protocol '{protocol_name}'")
        });

        addresses.insert(protocol_name.clone(), address);
    }
    addresses
});

#[derive(Debug, Clone)]
pub struct StateOverride {
    pub code: Option<Vec<u8>>,
    pub balance: Option<U256>,
    pub state_diff: HashMap<alloy::primitives::Bytes, alloy::primitives::Bytes>,
}

impl StateOverride {
    pub fn new() -> Self {
        Self { code: None, balance: None, state_diff: HashMap::new() }
    }

    pub fn with_code(mut self, code: Vec<u8>) -> Self {
        self.code = Some(code);
        self
    }

    pub fn with_balance(mut self, balance: U256) -> Self {
        self.balance = Some(balance);
        self
    }

    pub fn with_state_diff(
        mut self,
        slot: alloy::primitives::Bytes,
        value: alloy::primitives::Bytes,
    ) -> Self {
        self.state_diff.insert(slot, value);
        self
    }
}

/// Determine the executor file based on component ID
fn get_executor_file(component_id: &str) -> miette::Result<&'static str> {
    for (pattern, executor_file) in EXECUTOR_MAPPING.iter() {
        if component_id.contains(pattern) {
            return Ok(executor_file);
        }
    }
    Err(miette!("Unknown component type '{}' - no matching executor found", component_id))
}

/// Get executor address for a given component ID
fn get_executor_address(component_id: &str) -> miette::Result<Address> {
    if let Some(&address) = EXECUTOR_ADDRESSES.get(component_id) {
        return Ok(address);
    }
    Err(miette!("No executor address found for component type '{}'", component_id))
}

/// Load executor bytecode from the appropriate file based on solution component
fn load_executor_bytecode(solution: &Solution) -> miette::Result<Vec<u8>> {
    let first_swap = solution.swaps.first().unwrap();
    let component_id = &first_swap.component;

    let executor_file = get_executor_file(&component_id.protocol_system)?;
    let executor_path = PathBuf::from("../evm/test/executors").join(executor_file);

    // Read the JSON file and extract the bytecode
    let executor_json = std::fs::read_to_string(&executor_path)
        .into_diagnostic()
        .wrap_err(format!("Failed to read executor file: {}", executor_path.display()))?;

    let json_value: serde_json::Value = serde_json::from_str(&executor_json)
        .into_diagnostic()
        .wrap_err("Failed to parse executor JSON")?;

    let bytecode_str = json_value["runtimeBytecode"]
        .as_str()
        .ok_or_else(|| miette!("No bytecode field found in executor JSON"))?;

    // Remove 0x prefix if present
    let bytecode_hex =
        if let Some(stripped) = bytecode_str.strip_prefix("0x") { stripped } else { bytecode_str };

    hex::decode(bytecode_hex)
        .into_diagnostic()
        .wrap_err("Failed to decode executor bytecode from hex")
}

/// Calculate gas fees based on block base fee
fn calculate_gas_fees(block_header: &Block) -> miette::Result<(U256, U256)> {
    let base_fee = block_header
        .header
        .base_fee_per_gas
        .ok_or_else(|| miette::miette!("Block does not have base fee (pre-EIP-1559)"))?;
    // Set max_priority_fee_per_gas to a reasonable value (2 Gwei)
    let max_priority_fee_per_gas = U256::from(2_000_000_000u64); // 2 Gwei
                                                                 // Set max_fee_per_gas to base_fee * 2 + max_priority_fee_per_gas to handle fee fluctuations
    let max_fee_per_gas = U256::from(base_fee) * U256::from(2u64) + max_priority_fee_per_gas;

    info!(
        "Gas pricing: base_fee={}, max_priority_fee_per_gas={}, max_fee_per_gas={}",
        base_fee, max_priority_fee_per_gas, max_fee_per_gas
    );

    Ok((max_fee_per_gas, max_priority_fee_per_gas))
}

/// Set up all state overrides needed for simulation
fn setup_state_overrides(
    solution: &Solution,
    transaction: &tycho_simulation::tycho_execution::encoding::models::Transaction,
    user_address: Address,
    executor_bytecode: &[u8],
    include_executor_override: bool,
) -> miette::Result<HashMap<Address, StateOverride>> {
    let mut state_overwrites = HashMap::new();
    let token_address = Address::from_slice(&solution.given_token[..20]);

    // Extract executor address from the encoded solution's swaps data.
    // The solution should only have one swap for the test, so this should be safe.
    let executor_address = if let Some(first_swap) = solution.swaps.first() {
        get_executor_address(&first_swap.component.protocol_system)?
    } else {
        return Err(miette!("No swaps in solution - cannot determine executor address"));
    };

    // Add bytecode overwrite for the executor (conditionally)
    if include_executor_override {
        state_overwrites
            .insert(executor_address, StateOverride::new().with_code(executor_bytecode.to_vec()));
    }

    let tycho_router_address = Address::from_slice(&transaction.to[..20]);

    // Add ETH balance override for the user to ensure they have enough gas funds
    state_overwrites.insert(
        user_address,
        StateOverride::new().with_balance(U256::from_str("100000000000000000000").unwrap()), // 100 ETH
    );

    // TODO: temporary
    state_overwrites.insert(
        token_address,
        StateOverride::new()
            .with_state_diff(
                alloy::primitives::Bytes::from_str(
                    "0x4e4b5f80f87725e40fd825bd7b26188e05acd6dbf57e82d1bd0f2bd067293504",
                )
                .unwrap(),
                alloy::primitives::Bytes::from(U256::MAX.to_be_bytes::<32>()),
            )
            .with_state_diff(
                alloy::primitives::Bytes::from_str(
                    "0x4d1be9a589dbf86739d4cb42cefbbfa52cf9e4446f5af2b08fabe809bc11104a",
                )
                .unwrap(),
                alloy::primitives::Bytes::from(U256::MAX.to_be_bytes::<32>()),
            ),
    );

    Ok(state_overwrites)
}

/// Simulate a trade using eth_call for historical blocks
pub async fn simulate_trade_with_eth_call(
    rpc_provider: &RPCProvider,
    transaction: &tycho_simulation::tycho_execution::encoding::models::Transaction,
    solution: &Solution,
    block_number: u64,
    _adapter_contract_path: &str,
    block_header: &Block,
) -> miette::Result<BigUint> {
    let executor_bytecode = load_executor_bytecode(solution)?;
    let user_address = Address::from_slice(&solution.sender[..20]);
    let (max_fee_per_gas, max_priority_fee_per_gas) = calculate_gas_fees(block_header)?;

    // Convert main transaction to alloy TransactionRequest
    let execution_tx = TransactionRequest::default()
        .to(Address::from_slice(&transaction.to[..20]))
        .input(transaction.data.clone().into())
        .value(U256::from_str(&transaction.value.to_string()).unwrap_or_default())
        .from(user_address)
        .max_fee_per_gas(
            max_fee_per_gas
                .try_into()
                .unwrap_or(u128::MAX),
        )
        .max_priority_fee_per_gas(
            max_priority_fee_per_gas
                .try_into()
                .unwrap_or(u128::MAX),
        );
    let tycho_router_address = Address::from_slice(&transaction.to[..20]);
    let _token_address = Address::from_slice(&solution.given_token[..20]);

    // Copy router storage and code from current block to historical block
    // TODO get this at compile time.
    let router_bytecode_path = "../evm/test/router/TychoRouter.runtime.json";

    let router_override = rpc_provider
        .copy_contract_storage_and_code(tycho_router_address, router_bytecode_path)
        .await
        .wrap_err("Failed to copy router contract storage and code")?;

    // Set up state overrides including router override
    let mut state_overwrites =
        setup_state_overrides(solution, transaction, user_address, &executor_bytecode, true)?; // Include executor override for historical blocks

    // Add the router override
    state_overwrites.insert(tycho_router_address, router_override);

    let execution_amount_out = rpc_provider
        .simulate_transactions_with_tracing(execution_tx, block_number, state_overwrites)
        .await
        .map_err(|e| {
            info!("Execution transaction failed with error: {}", e);
            e
        })
        .wrap_err("Execution simulation failed")?;

    Ok(u256_to_biguint(execution_amount_out))
}
