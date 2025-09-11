use std::collections::HashMap;

use alloy::{
    contract::{ContractInstance, Interface},
    dyn_abi::DynSolValue,
    eips::eip1898::BlockId,
    primitives::{address, keccak256, map::AddressHashMap, Address, FixedBytes, TxKind, U256},
    providers::{Provider, ProviderBuilder},
    rpc::types::{
        simulate::{SimBlock, SimulatePayload},
        state::AccountOverride,
        Block, BlockOverrides, TransactionRequest,
    },
    transports::http::reqwest::Url,
};
use miette::{IntoDiagnostic, WrapErr};
use serde_json::{json, Value};
use tracing::info;

const NATIVE_ALIASES: &[Address] = &[
    address!("0x0000000000000000000000000000000000000000"),
    address!("0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"),
];

const ERC_20_ABI: &str = r#"[{"inputs":[{"name":"_owner","type":"address"}],"name":"balanceOf","outputs":[{"name":"balance","type":"uint256"}],"stateMutability":"view","type":"function"}]"#;

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

pub struct RPCProvider {
    url: Url,
}

impl RPCProvider {
    pub fn new(url: String) -> Self {
        let url = url.as_str().parse().unwrap();
        RPCProvider { url }
    }

    pub async fn get_token_balance(
        &self,
        token_address: Address,
        wallet_address: Address,
        block_number: u64,
    ) -> miette::Result<U256> {
        let provider = ProviderBuilder::new().connect_http(self.url.clone());
        let block_id: BlockId = BlockId::from(block_number);

        match NATIVE_ALIASES.contains(&token_address) {
            true => provider
                .get_balance(wallet_address)
                .block_id(block_id)
                .await
                .into_diagnostic()
                .wrap_err("Failed to fetch token balance"),
            false => {
                let abi = serde_json::from_str(ERC_20_ABI)
                    .into_diagnostic()
                    .wrap_err("invalid ABI")?;

                let contract = ContractInstance::new(token_address, provider, Interface::new(abi));

                let wallet_addr = DynSolValue::from(wallet_address);

                let result_value = contract
                    .function("balanceOf", &[wallet_addr])
                    .expect("Failed to build function call")
                    .block(block_id)
                    .call()
                    .await
                    .into_diagnostic()
                    .wrap_err("Failed to fetch ERC-20 Balance")?;
                let result: U256 = result_value
                    .first()
                    .ok_or_else(|| miette::miette!("No value returned from contract call"))?
                    .as_uint()
                    .ok_or_else(|| miette::miette!("Returned value is not a uint"))?
                    .0;
                Ok(result)
            }
        }
    }

    pub async fn get_block_header(&self, block_number: u64) -> miette::Result<Block> {
        let provider = ProviderBuilder::new().connect_http(self.url.clone());
        let block_id: BlockId = BlockId::from(block_number);

        provider
            .get_block(block_id)
            .await
            .into_diagnostic()
            .wrap_err("Failed to fetch block header")
            .and_then(|block_opt| block_opt.ok_or_else(|| miette::miette!("Block not found")))
    }

    pub async fn eth_call_with_overwrites(
        &self,
        transaction: TransactionRequest,
        block_number: u64,
        state_overwrites: HashMap<Address, StateOverride>,
    ) -> miette::Result<alloy::primitives::Bytes> {
        let client = reqwest::Client::new();
        let mut overwrites_json = serde_json::Map::new();
        for (address, override_data) in state_overwrites {
            let mut override_obj = serde_json::Map::new();
            if let Some(code) = override_data.code {
                override_obj.insert("code".to_string(), json!(format!("0x{}", hex::encode(code))));
            }
            if let Some(balance) = override_data.balance {
                override_obj.insert("balance".to_string(), json!(format!("0x{:x}", balance)));
            }
            if !override_data.state_diff.is_empty() {
                let mut state_diff_json = serde_json::Map::new();
                for (slot, value) in override_data.state_diff {
                    state_diff_json.insert(
                        format!("0x{}", hex::encode(&**slot)),
                        json!(format!("0x{}", hex::encode(&**value))),
                    );
                }
                // "stateDiff" contains the fake key-value mapping to override individual slots in
                // the account storage. Used to override attrs that are not code or balance.
                override_obj.insert("stateDiff".to_string(), json!(state_diff_json));
            }

            overwrites_json.insert(format!("0x{}", hex::encode(address)), json!(override_obj));
        }
        info!("Transaction info: {transaction:?}");
        let block_string = format!("0x{:x}", block_number);
        // let block_string: String = "latest".into();

        let request_body = json!({
            "jsonrpc": "2.0",
            "method": "eth_call",
            "params": [
                {
                    "to": transaction.to.map(|addr| match addr {
                        TxKind::Call(address) => format!("0x{}", hex::encode(address)),
                        TxKind::Create => String::new(),
                    }),
                    "data": transaction.input.data.as_ref().map(|data| format!("0x{}", hex::encode(data.clone()))),
                    "value": transaction.value.map(|val| format!("0x{val:x}")),
                },
                block_string,
                overwrites_json
            ],
            "id": 1
        });
        // info!("About to post request body: {request_body:?}");

        let response = client
            .post(self.url.as_str())
            .json(&request_body)
            .send()
            .await
            .into_diagnostic()
            .wrap_err("Failed to send eth_call request")?;

        info!("Posted. Got response: {response:?}");

        let response_json: Value = response
            .json()
            .await
            .into_diagnostic()
            .wrap_err("Failed to parse JSON response")?;

        info!("Got the following in the response json: {response_json:?}");

        if let Some(error) = response_json.get("error") {
            let error_code = error
                .get("code")
                .and_then(|c| c.as_i64())
                .unwrap_or(0);
            let error_message = error
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown error");

            // Try to extract more details from the error
            let detailed_error = if let Some(data) = error.get("data") {
                format!(
                    "RPC error (code: {}, message: {}, data: {})",
                    error_code, error_message, data
                )
            } else {
                format!("RPC error (code: {}, message: {})", error_code, error_message)
            };

            info!("❌ Detailed simulation error: {}", detailed_error);
            return Err(miette::miette!("{}", detailed_error));
        }

        let result = response_json
            .get("result")
            .ok_or_else(|| miette::miette!("No result in response"))?
            .as_str()
            .ok_or_else(|| miette::miette!("Result is not a string"))?;

        let bytes_result =
            if result.starts_with("0x") { hex::decode(&result[2..]) } else { hex::decode(result) }
                .into_diagnostic()
                .wrap_err("Failed to decode hex result")?;

        Ok(alloy::primitives::Bytes::from(bytes_result))
    }

    /// Helper function to get the contract's storage at the given slot at the latest block.
    pub async fn get_storage_at(
        &self,
        contract_address: Address,
        slot: alloy::primitives::FixedBytes<32>,
    ) -> miette::Result<alloy::primitives::FixedBytes<32>> {
        let provider = ProviderBuilder::new().connect_http(self.url.clone());
        let storage_value = provider
            .get_storage_at(contract_address, slot.into())
            .await
            .into_diagnostic()
            .wrap_err("Failed to fetch storage slot")?;

        Ok(storage_value.into())
    }

    pub async fn copy_contract_storage_and_code(
        &self,
        contract_address: Address,
        router_bytecode_path: &str,
    ) -> miette::Result<StateOverride> {
        let router_json = std::fs::read_to_string(router_bytecode_path)
            .into_diagnostic()
            .wrap_err("Failed to read router bytecode file")?;

        let json_value: serde_json::Value = serde_json::from_str(&router_json)
            .into_diagnostic()
            .wrap_err("Failed to parse router JSON")?;

        let bytecode_str = json_value["runtimeBytecode"]
            .as_str()
            .ok_or_else(|| miette::miette!("No runtimeBytecode field found in router JSON"))?;

        // Remove 0x prefix if present
        let bytecode_hex =
            if bytecode_str.starts_with("0x") { &bytecode_str[2..] } else { bytecode_str };

        let router_bytecode = hex::decode(bytecode_hex)
            .into_diagnostic()
            .wrap_err("Failed to decode router bytecode from hex")?;

        info!("Loaded router bytecode: {} bytes", router_bytecode.len());

        // Start with the router bytecode override
        let mut state_override = StateOverride::new().with_code(router_bytecode);

        // The executors mapping is: mapping(address => bool) public executors;
        info!(
            "Copying executor mapping data from {} known executors...",
            crate::test_runner::EXECUTOR_ADDRESSES.len()
        );

        for (protocol_name, &executor_address) in crate::test_runner::EXECUTOR_ADDRESSES.iter() {
            let storage_slot = self.calculate_executor_storage_slot(executor_address);

            // Double check that this executor is actually approved.
            // TODO this can be simplified to just set the value to 1 every time.
            //  This explicit check was just for debug purposes to verify our storage slot calculation.
            match self
                .get_storage_at(contract_address, storage_slot)
                .await
            {
                Ok(value) => {
                    if !value.is_zero() {
                        info!(
                            "Found executor approval for {} ({:?}): 0x{}",
                            protocol_name,
                            executor_address,
                            hex::encode(value)
                        );
                        state_override = state_override.with_state_diff(
                            alloy::primitives::Bytes::from(storage_slot.to_vec()),
                            alloy::primitives::Bytes::from(value.to_vec()),
                        );
                    } else {
                        info!(
                            "Executor {} ({:?}) is not approved (value is zero)",
                            protocol_name, executor_address
                        );
                    }
                }
                Err(e) => {
                    info!(
                        "Failed to fetch executor approval for {} ({:?}): {}",
                        protocol_name, executor_address, e
                    );
                }
            }
        }

        info!("Completed storage fetch for contract {:?}", contract_address);
        Ok(state_override)
    }

    /// Calculate storage slot for Solidity mapping.
    ///
    /// The solidity code:
    /// keccak256(abi.encodePacked(bytes32(key), bytes32(slot)))
    pub fn calculate_executor_storage_slot(&self, key: Address) -> FixedBytes<32> {
        // Convert key (20 bytes) to 32-byte left-padded array (uint256)
        let mut key_bytes = [0u8; 32];
        key_bytes[12..].copy_from_slice(key.as_slice());

        // The base of the executor storage slot is 1, since there is only one
        // variable that is initialized before it (which is _roles in AccessControl.sol).
        // In this case, _roles gets slot 0.
        // The slots are given in order to the parent contracts' variables first and foremost.
        let slot = U256::from(1);

        // Convert U256 slot to 32-byte big-endian array
        let slot_bytes = slot.to_be_bytes::<32>();

        // Concatenate key_bytes + slot_bytes, then keccak hash
        let mut buf = [0u8; 64];
        buf[..32].copy_from_slice(&key_bytes);
        buf[32..].copy_from_slice(&slot_bytes);
        keccak256(&buf)
    }

    // TODO remove this. This was useful for initial debugging, but is ultimately useless for
    //  simulations on past blocks.
    pub async fn simulate_transactions_with_tracing(
        &self,
        transactions: Vec<TransactionRequest>,
        block_number: u64,
        state_overwrites: HashMap<Address, StateOverride>,
    ) -> miette::Result<Vec<alloy::rpc::types::simulate::SimulatedBlock>> {
        let provider = ProviderBuilder::new().connect_http(self.url.clone());

        // Convert our StateOverride to alloy's state override format
        let mut alloy_state_overrides = AddressHashMap::default();
        for (address, override_data) in state_overwrites {
            let mut account_override = AccountOverride::default();

            if let Some(code) = override_data.code {
                account_override.code = Some(alloy::primitives::Bytes::from(code));
            }

            if let Some(balance) = override_data.balance {
                account_override.balance = Some(balance);
            }

            if !override_data.state_diff.is_empty() {
                // Convert Bytes to FixedBytes<32> for storage slots
                let mut state_diff = HashMap::default();
                for (slot, value) in override_data.state_diff {
                    // Convert dynamic bytes to 32-byte fixed bytes
                    let slot_bytes: [u8; 32] = if slot.len() >= 32 {
                        slot[..32]
                            .try_into()
                            .unwrap_or([0u8; 32])
                    } else {
                        let mut arr = [0u8; 32];
                        arr[32 - slot.len()..].copy_from_slice(&slot);
                        arr
                    };
                    let value_bytes: [u8; 32] = if value.len() >= 32 {
                        value[..32]
                            .try_into()
                            .unwrap_or([0u8; 32])
                    } else {
                        let mut arr = [0u8; 32];
                        arr[32 - value.len()..].copy_from_slice(&value);
                        arr
                    };
                    state_diff.insert(FixedBytes(slot_bytes), FixedBytes(value_bytes));
                }
                account_override.state_diff = Some(state_diff);
            }

            alloy_state_overrides.insert(address, account_override);
        }

        let payload = SimulatePayload {
            block_state_calls: vec![SimBlock {
                block_overrides: Some(BlockOverrides {
                    number: Some(U256::from(block_number)),
                    ..Default::default()
                }),
                state_overrides: if alloy_state_overrides.is_empty() {
                    None
                } else {
                    Some(alloy_state_overrides)
                },
                calls: transactions,
            }],
            trace_transfers: true,
            validation: true,
            return_full_transactions: true,
        };
        let result = provider.simulate(&payload).await;
        let result_diagnostic = result
            .into_diagnostic()
            .wrap_err("Failed to simulate transactions with full tracing")?;

        Ok(result_diagnostic)
    }
}

#[cfg(test)]
mod tests {
    use std::{env, str::FromStr};

    use alloy::primitives::address;

    use super::*;

    #[tokio::test]
    async fn get_token_balance_native_token() {
        let eth_rpc_url = env::var("RPC_URL").expect("Missing RPC_URL in environment");

        let rpc_provider = RPCProvider::new(eth_rpc_url);
        let token_address = address!("0x0000000000000000000000000000000000000000");
        let wallet_address = address!("0x787B8840100d9BaAdD7463f4a73b5BA73B00C6cA");
        let block_number = 21998530;

        let balance = rpc_provider
            .get_token_balance(token_address, wallet_address, block_number)
            .await
            .unwrap();

        assert_eq!(
            balance,
            U256::from_str("1070041574684539264153").expect("Failed to convert ETH value")
        );
    }

    #[tokio::test]
    async fn get_token_balance_erc20_token() {
        let eth_rpc_url = env::var("RPC_URL").expect("Missing RPC_URL in environment");

        let rpc_provider = RPCProvider::new(eth_rpc_url);
        let token_address = address!("0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48");
        let wallet_address = address!("0x787B8840100d9BaAdD7463f4a73b5BA73B00C6cA");
        let block_number = 21998530;

        let balance = rpc_provider
            .get_token_balance(token_address, wallet_address, block_number)
            .await
            .unwrap();

        assert_eq!(balance, U256::from(717250938432_u64));
    }

    #[tokio::test]
    async fn get_block_header() {
        let eth_rpc_url = env::var("RPC_URL").expect("Missing RPC_URL in environment");

        let rpc_provider = RPCProvider::new(eth_rpc_url);
        let block_number = 21998530;

        let block_header = rpc_provider
            .get_block_header(block_number)
            .await
            .unwrap();

        // Verify that we got a block with the correct number
        assert_eq!(block_number, block_header.header.number);
        assert_eq!(block_header.header.timestamp, 1741393115);
    }
}
