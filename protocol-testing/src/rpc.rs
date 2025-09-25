use std::str::FromStr;

use alloy::{
    contract::{ContractInstance, Interface},
    dyn_abi::DynSolValue,
    eips::eip1898::BlockId,
    primitives::{address, map::AddressHashMap, Address, FixedBytes, U256},
    providers::{Provider, ProviderBuilder},
    rpc::types::{
        state::AccountOverride,
        trace::geth::{
            GethDebugBuiltInTracerType, GethDebugTracerType, GethDebugTracingCallOptions,
            GethDebugTracingOptions,
        },
        Block, TransactionRequest,
    },
    sol_types::SolValue,
    transports::http::reqwest::Url,
};
use miette::{IntoDiagnostic, WrapErr};
use serde_json::Value;
use tracing::info;
use tycho_simulation::tycho_common::Bytes;

use crate::{
    execution::{
        calculate_executor_storage_slot, load_executor_bytecode, EXECUTOR_ADDRESS,
        ROUTER_BYTECODE_JSON,
    },
    traces::print_call_trace,
};

const NATIVE_ALIASES: &[Address] = &[
    address!("0x0000000000000000000000000000000000000000"),
    address!("0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"),
];

const ERC_20_ABI: &str = r#"[{"inputs":[{"name":"_owner","type":"address"}],"name":"balanceOf","outputs":[{"name":"balance","type":"uint256"}],"stateMutability":"view","type":"function"}]"#;

pub struct RPCProvider {
    pub url: Url,
    trace: bool,
}

impl RPCProvider {
    pub fn new(url: String, trace: bool) -> Self {
        let url = url.as_str().parse().unwrap();
        RPCProvider { url, trace }
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

    /// Helper function to get the contract's storage at the given slot at the latest block.
    pub async fn get_storage_at(
        &self,
        contract_address: Address,
        slot: FixedBytes<32>,
    ) -> miette::Result<FixedBytes<32>> {
        let provider = ProviderBuilder::new().connect_http(self.url.clone());
        let storage_value = provider
            .get_storage_at(contract_address, slot.into())
            .await
            .into_diagnostic()
            .wrap_err("Failed to fetch storage slot")?;

        Ok(storage_value.into())
    }

    /// Sets up state overwrites for the Tycho router and its associated executor.
    ///
    /// This method prepares the router for simulation by:
    /// 1. Overriding the router's bytecode with the embedded runtime bytecode
    /// 2. Copying executor approval storage from the current block to maintain permissions
    /// 3. Overriding the executor's bytecode based on the protocol system
    ///
    /// # Arguments
    /// * `router_address` - The address of the Tycho router contract
    /// * `protocol_system` - The protocol system identifier (e.g., "uniswap_v2", "vm:balancer_v2")
    ///
    /// # Returns
    /// A HashMap containing account overwrites for both the router and executor addresses.
    /// The router override includes bytecode and executor approval storage.
    /// The executor override includes the appropriate bytecode for the protocol.
    ///
    /// # Errors
    /// Returns an error if:
    /// - Router bytecode JSON parsing fails
    /// - Executor address parsing fails
    /// - Storage slot fetching fails
    /// - Executor bytecode loading fails
    pub async fn setup_router_overwrites(
        &self,
        router_address: Address,
        protocol_system: &str,
    ) -> miette::Result<AddressHashMap<AccountOverride>> {
        let json_value: serde_json::Value = serde_json::from_str(ROUTER_BYTECODE_JSON)
            .into_diagnostic()
            .wrap_err("Failed to parse router JSON")?;

        let bytecode_str = json_value["runtimeBytecode"]
            .as_str()
            .ok_or_else(|| miette::miette!("No runtimeBytecode field found in router JSON"))?;

        // Remove 0x prefix if present
        let bytecode_hex = if let Some(stripped) = bytecode_str.strip_prefix("0x") {
            stripped
        } else {
            bytecode_str
        };

        let router_bytecode = hex::decode(bytecode_hex)
            .into_diagnostic()
            .wrap_err("Failed to decode router bytecode from hex")?;

        // Start with the router bytecode override
        let mut state_overwrites = AddressHashMap::default();
        let mut tycho_router_override = AccountOverride::default().with_code(router_bytecode);

        // Find executor address approval storage slot
        let executor_address = Address::from_str(EXECUTOR_ADDRESS).into_diagnostic()?;
        let storage_slot = calculate_executor_storage_slot(executor_address);

        let storage_value = self
            .get_storage_at(router_address, storage_slot)
            .await?;

        tycho_router_override =
            tycho_router_override.with_state_diff(vec![(storage_slot, storage_value)]);

        state_overwrites.insert(router_address, tycho_router_override);

        // Add bytecode overwrite for the executor
        let executor_bytecode = load_executor_bytecode(protocol_system)?;
        state_overwrites.insert(
            executor_address,
            AccountOverride::default().with_code(executor_bytecode.to_vec()),
        );
        Ok(state_overwrites)
    }

    pub async fn simulate_transactions_with_tracing(
        &self,
        transaction: TransactionRequest,
        block_number: u64,
        state_overwrites: AddressHashMap<AccountOverride>,
    ) -> miette::Result<U256> {
        let provider = ProviderBuilder::new().connect_http(self.url.clone());

        // Configure tracing options - use callTracer for better formatted results
        let tracing_options = GethDebugTracingOptions {
            tracer: Some(GethDebugTracerType::BuiltInTracer(
                GethDebugBuiltInTracerType::CallTracer,
            )),
            config: Default::default(),
            tracer_config: Default::default(),
            timeout: None,
        };

        let trace_options = GethDebugTracingCallOptions {
            tracing_options,
            state_overrides: if state_overwrites.is_empty() {
                None
            } else {
                Some(state_overwrites)
            },
            block_overrides: None,
        };

        let result: Value = provider
            .client()
            .request("debug_traceCall", (transaction, BlockId::from(block_number), trace_options))
            .await
            .map_err(|e| {
                tracing::error!("debug_traceCall RPC error: {:#}", e);
                e
            })
            .into_diagnostic()
            .wrap_err("Failed to debug trace call.")?;

        if self.trace {
            print_call_trace(&result, 0).await;
        }
        let has_error = result
            .as_object()
            .and_then(|obj| obj.get("error"))
            .is_some();

        let has_failed = result
            .as_object()
            .and_then(|obj| obj.get("failed"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if has_error || has_failed {
            if let Some(result_obj) = result.as_object() {
                if let Some(error) = result_obj.get("error") {
                    return Err(miette::miette!("Transaction execution failed: {}", error));
                }
                if let Some(revert_reason) = result_obj.get("revertReason") {
                    return Err(miette::miette!("Transaction reverted: {}", revert_reason));
                }
            }
            return Err(miette::miette!("Transaction failed"));
        } else {
            info!("Transaction successfully simulated.");
        }

        let mut executed_amount_out = U256::ZERO;
        if let Some(result_obj) = result.as_object() {
            if let Some(gas_used) = result_obj
                .get("gasUsed")
                .and_then(|v| v.as_str())
            {
                let gas_used_decoded = U256::from_str_radix(gas_used.trim_start_matches("0x"), 16)
                    .into_diagnostic()?;
                info!("Gas used: {}", gas_used_decoded);
            }
            if let Some(output) = result_obj
                .get("output")
                .and_then(|v| v.as_str())
            {
                executed_amount_out = U256::abi_decode(&Bytes::from_str(output).into_diagnostic()?)
                    .into_diagnostic()?;
            }
        }
        Ok(executed_amount_out)
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

        let rpc_provider = RPCProvider::new(eth_rpc_url, false);
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

        let rpc_provider = RPCProvider::new(eth_rpc_url, false);
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

        let rpc_provider = RPCProvider::new(eth_rpc_url, false);
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
