use std::{collections::HashMap, env, path::PathBuf, str::FromStr, sync::LazyLock};

use alloy::{
    primitives::{Address, Keccak256, U256},
    rpc::types::{Block, TransactionRequest},
};
use figment::{
    providers::{Format, Yaml},
    Figment,
};
use itertools::Itertools;
use miette::{miette, IntoDiagnostic, WrapErr};
use num_bigint::BigUint;
use num_traits::Zero;
use postgres::{Client, Error, NoTls};
use serde_json::{self, Value};
use tokio::runtime::Runtime;
use tracing::{debug, error, info};
use tycho_client::feed::BlockHeader;
use tycho_common::{
    dto::{Chain, ProtocolComponent, ResponseAccount, ResponseProtocolState},
    models::token::Token,
    Bytes,
};
use tycho_simulation::{
    evm::{
        decoder::TychoStreamDecoder,
        engine_db::tycho_db::PreCachedDB,
        protocol::{u256_num::bytes_to_u256, vm::state::EVMPoolState},
    },
    protocol::models::DecoderContext,
    tycho_client::feed::{
        synchronizer::{ComponentWithState, Snapshot, StateSyncMessage},
        FeedMessage,
    },
    tycho_execution::encoding::models::Solution,
};

use crate::{
    adapter_builder::AdapterContractBuilder,
    config::{IntegrationTest, IntegrationTestsConfig, ProtocolComponentWithTestConfig},
    encoding::encode_swap,
    rpc::{RPCProvider, StateOverride},
    tycho_rpc::TychoClient,
    tycho_runner::TychoRunner,
    utils::build_spkg,
};

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
            .expect(&format!("Invalid address format for protocol '{}'", protocol_name));

        let address = Address::from_str(address_str)
            .expect(&format!("Invalid address '{}' for protocol '{}'", address_str, protocol_name));

        addresses.insert(protocol_name.clone(), address);
    }

    info!("Loaded {} executor addresses from test_executor_addresses.json", addresses.len());
    addresses
});

/// Determine the executor file based on component ID
fn get_executor_file(component_id: &str) -> miette::Result<&'static str> {
    for (pattern, executor_file) in EXECUTOR_MAPPING.iter() {
        if component_id.contains(pattern) {
            info!("Matched component '{}' to executor: {}", component_id, executor_file);
            return Ok(executor_file);
        }
    }
    Err(miette!("Unknown component type '{}' - no matching executor found", component_id))
}

/// Get executor address for a given component ID
fn get_executor_address(component_id: &str) -> miette::Result<Address> {
    if let Some(&address) = EXECUTOR_ADDRESSES.get(component_id) {
        info!("Matched component '{}' to executor address: {:?}", component_id, address);
        return Ok(address);
    }
    Err(miette!("No executor address found for component type '{}'", component_id))
}

pub struct TestRunner {
    tycho_logs: bool,
    db_url: String,
    vm_traces: bool,
    substreams_path: PathBuf,
    adapter_contract_builder: AdapterContractBuilder,
    match_test: Option<String>,
}

impl TestRunner {
    pub fn new(
        substreams_path: PathBuf,
        evm_path: PathBuf,
        match_test: Option<String>,
        tycho_logs: bool,
        db_url: String,
        vm_traces: bool,
    ) -> Self {
        let adapter_contract_builder =
            AdapterContractBuilder::new(evm_path.to_string_lossy().to_string());
        Self {
            tycho_logs,
            db_url,
            vm_traces,
            substreams_path,
            adapter_contract_builder,
            match_test,
        }
    }

    pub fn run_tests(&self) -> miette::Result<()> {
        let terminal_width = termsize::get()
            .map(|size| size.cols as usize - 35) // Remove length of log prefix (35)
            .unwrap_or(80);
        info!("{}\n", "-".repeat(terminal_width));

        let config_yaml_path = self
            .substreams_path
            .join("integration_test.tycho.yaml");

        let config = Self::parse_config(&config_yaml_path)?;

        let tests = match &self.match_test {
            Some(filter) => config
                .tests
                .iter()
                .filter(|test| test.name.contains(filter))
                .collect::<Vec<&IntegrationTest>>(),
            None => config
                .tests
                .iter()
                .collect::<Vec<&IntegrationTest>>(),
        };
        let tests_count = tests.len();

        info!("Running {} tests ...\n", tests_count);

        let mut failed_tests: Vec<String> = Vec::new();
        let mut count = 1;

        for test in &tests {
            info!("TEST {}: {}", count, test.name);

            match self.run_test(test, &config, config.skip_balance_check) {
                Ok(_) => {
                    info!("✅ {} passed\n", test.name);
                }
                Err(e) => {
                    failed_tests.push(test.name.clone());
                    error!("❗️{} failed: {:?}\n", test.name, e);
                }
            }

            info!("{}\n", "-".repeat(terminal_width));
            count += 1;
        }

        info!("Tests finished!");
        info!("Passed {}/{}\n", tests_count - failed_tests.len(), tests_count);
        if !failed_tests.is_empty() {
            Err(miette!("Failed tests: {}", failed_tests.join(", ")))
        } else {
            Ok(())
        }
    }

    fn parse_config(config_yaml_path: &PathBuf) -> miette::Result<IntegrationTestsConfig> {
        info!("Config YAML: {}", config_yaml_path.display());
        let yaml = Yaml::file(config_yaml_path);
        let figment = Figment::new().merge(yaml);
        let config = figment
            .extract::<IntegrationTestsConfig>()
            .into_diagnostic()
            .wrap_err("Failed to load test configuration:")?;
        Ok(config)
    }

    fn run_test(
        &self,
        test: &IntegrationTest,
        config: &IntegrationTestsConfig,
        skip_balance_check: bool,
    ) -> miette::Result<()> {
        self.empty_database()
            .into_diagnostic()
            .wrap_err("Failed to empty the database")?;

        let substreams_yaml_path = self
            .substreams_path
            .join(&config.substreams_yaml_path);

        let mut initialized_accounts = config
            .initialized_accounts
            .clone()
            .unwrap_or_default();
        initialized_accounts.extend(
            test.initialized_accounts
                .clone()
                .unwrap_or_default(),
        );

        let spkg_path =
            build_spkg(&substreams_yaml_path, test.start_block).wrap_err("Failed to build spkg")?;

        let tycho_runner =
            TychoRunner::new(self.db_url.clone(), initialized_accounts, self.tycho_logs);

        tycho_runner
            .run_tycho(
                spkg_path.as_str(),
                test.start_block,
                test.stop_block,
                &config.protocol_type_names,
                &config.protocol_system,
            )
            .wrap_err("Failed to run Tycho")?;

        tycho_runner.run_with_rpc_server(
            |expected_components, start_block, stop_block, skip_balance_check| {
                validate_state(
                    expected_components,
                    start_block,
                    stop_block,
                    skip_balance_check,
                    config,
                    &self.adapter_contract_builder,
                )
            },
            &test.expected_components,
            test.start_block,
            test.stop_block,
            skip_balance_check,
        )?
    }

    fn empty_database(&self) -> Result<(), Error> {
        // Remove db name from URL. This is required because we cannot drop a database that we are
        // currently connected to.
        let base_url = match self.db_url.rfind('/') {
            Some(pos) => &self.db_url[..pos],
            None => self.db_url.as_str(),
        };
        let mut client = Client::connect(base_url, NoTls)?;

        client.execute("DROP DATABASE IF EXISTS \"tycho_indexer_0\" WITH (FORCE)", &[])?;
        client.execute("CREATE DATABASE \"tycho_indexer_0\"", &[])?;

        Ok(())
    }
}

fn validate_state(
    expected_components: &Vec<ProtocolComponentWithTestConfig>,
    start_block: u64,
    stop_block: u64,
    skip_balance_check: bool,
    config: &IntegrationTestsConfig,
    adapter_contract_builder: &AdapterContractBuilder,
) -> miette::Result<()> {
    let rt = Runtime::new().unwrap();

    // Create Tycho client for the RPC server
    let tycho_client = TychoClient::new("http://localhost:4242")
        .into_diagnostic()
        .wrap_err("Failed to create Tycho client")?;

    let chain = Chain::Ethereum;
    let protocol_system = &config.protocol_system;

    // Fetch data from Tycho RPC. We use block_on to avoid using async functions on the testing
    // module, in order to simplify debugging
    let protocol_components = rt
        .block_on(tycho_client.get_protocol_components(protocol_system, chain))
        .into_diagnostic()
        .wrap_err("Failed to get protocol components")?;

    let expected_ids = expected_components
        .iter()
        .map(|c| c.base.id.clone())
        .collect::<Vec<String>>();

    let protocol_states = rt
        .block_on(tycho_client.get_protocol_state(protocol_system, expected_ids, chain))
        .into_diagnostic()
        .wrap_err("Failed to get protocol state")?;

    let vm_storages = rt
        .block_on(tycho_client.get_contract_state(protocol_system, chain))
        .into_diagnostic()
        .wrap_err("Failed to get contract state")?;

    // Create a map of component IDs to components for easy lookup
    let components_by_id: HashMap<String, ProtocolComponent> = protocol_components
        .clone()
        .into_iter()
        .map(|c| (c.id.clone(), c))
        .collect();

    let protocol_states_by_id: HashMap<String, ResponseProtocolState> = protocol_states
        .into_iter()
        .map(|s| (s.component_id.clone(), s))
        .collect();

    info!("Found {} protocol components", components_by_id.len());
    info!("Found {} protocol states", protocol_states_by_id.len());

    info!("Validating state...");

    // Step 1: Validate that all expected components are present on Tycho after indexing
    debug!("Validating {:?} expected components", expected_components.len());
    for expected_component in expected_components {
        let component_id = expected_component.base.id.clone();

        let component = components_by_id
            .get(&component_id)
            .ok_or_else(|| miette!("Component {:?} was not found on Tycho", component_id))?;

        let diff = expected_component
            .base
            .compare(component, true);
        match diff {
            Some(diff) => {
                return Err(miette!(
                    "Component {} does not match the expected state:\n{}",
                    component_id,
                    diff
                ));
            }
            None => {
                info!("Component {} matches the expected state", component_id);
            }
        }
    }
    info!("All expected components were successfully found on Tycho and match the expected state");

    // Step 2: Validate Token Balances
    match skip_balance_check {
        true => info!("Skipping balance check"),
        false => {
            validate_token_balances(&components_by_id, &protocol_states_by_id, start_block, &rt)?;
            info!("All token balances match the values found onchain")
        }
    }

    // Step 3: Run Tycho Simulation

    // Build/find the adapter contract
    let adapter_contract_path =
        match adapter_contract_builder.find_contract(&config.adapter_contract) {
            Ok(path) => {
                info!("Found adapter contract at: {}", path.display());
                path
            }
            Err(_) => {
                info!("Adapter contract not found, building it...");
                adapter_contract_builder
                    .build_target(
                        &config.adapter_contract,
                        config
                            .adapter_build_signature
                            .as_deref(),
                        config.adapter_build_args.as_deref(),
                    )
                    .wrap_err("Failed to build adapter contract")?
            }
        };

    info!("Using adapter contract: {}", adapter_contract_path.display());
    let adapter_contract_path_str: &str = adapter_contract_path.to_str().unwrap();

    // Clear the shared database state to ensure test isolation
    // This prevents state from previous tests from affecting the current test
    tycho_simulation::evm::engine_db::SHARED_TYCHO_DB.clear();

    let mut decoder = TychoStreamDecoder::new();
    let decoder_context = DecoderContext::new().vm_adapter_path(adapter_contract_path_str);
    decoder.register_decoder_with_context::<EVMPoolState<PreCachedDB>>(
        protocol_system,
        decoder_context,
    );

    // Filter out components that have skip_simulation = true (match Python behavior)
    let simulation_component_ids: std::collections::HashSet<String> = expected_components
        .iter()
        .filter(|c| !c.skip_simulation)
        .map(|c| c.base.id.clone())
        .collect();

    info!("Components to simulate: {}", simulation_component_ids.len());
    for id in &simulation_component_ids {
        info!("  Simulating component: {}", id);
    }

    if simulation_component_ids.is_empty() {
        info!("No components to simulate, skipping simulation validation");
        return Ok(());
    }

    // Mock a stream message, with only a Snapshot and no deltas
    let mut states: HashMap<String, ComponentWithState> = HashMap::new();
    for (id, component) in &components_by_id {
        let component_id = id;

        // Only include components that should be simulated
        if !simulation_component_ids.contains(component_id) {
            continue;
        }

        let state = protocol_states_by_id
            .get(component_id)
            .wrap_err(format!(
                "Component {id} does not exist in protocol_states_by_id {protocol_states_by_id:?}"
            ))?
            .clone();

        let component_with_state = ComponentWithState {
            state,
            component: component.clone(),
            component_tvl: None,
            entrypoints: vec![],
        }; // TODO
        states.insert(component_id.clone(), component_with_state);
    }
    // Convert vm_storages to a HashMap - match Python behavior exactly
    let vm_storage: HashMap<Bytes, ResponseAccount> = vm_storages
        .into_iter()
        .map(|x| (x.address.clone(), x))
        .collect();

    let snapshot = Snapshot { states, vm_storage };

    let bytes = [0u8; 32];

    // Get block header to extract the timestamp
    let rpc_url = env::var("RPC_URL")
        .into_diagnostic()
        .wrap_err("Missing RPC_URL in environment")?;
    let rpc_provider = RPCProvider::new(rpc_url);
    let block_header = rt
        .block_on(rpc_provider.get_block_header(stop_block))
        .wrap_err("Failed to get block header")?;

    let state_msgs: HashMap<String, StateSyncMessage<BlockHeader>> = HashMap::from([(
        String::from(protocol_system),
        StateSyncMessage {
            header: BlockHeader {
                hash: Bytes::from(bytes),
                number: stop_block,
                parent_hash: Bytes::from(bytes),
                revert: false,
                timestamp: block_header.header.timestamp,
            },
            snapshots: snapshot,
            deltas: None,
            removed_components: HashMap::new(),
        },
    )]);

    let all_tokens = rt
        .block_on(tycho_client.get_tokens(Chain::Ethereum, None, None))
        .into_diagnostic()
        .wrap_err("Failed to get tokens")?;
    info!("Loaded {} tokens", all_tokens.len());

    rt.block_on(decoder.set_tokens(all_tokens));

    let mut pairs: HashMap<String, Vec<Token>> = HashMap::new();

    let message: FeedMessage = FeedMessage { state_msgs, sync_states: Default::default() };

    let block_msg = rt
        .block_on(decoder.decode(message))
        .into_diagnostic()
        .wrap_err("Failed to decode message")?;

    for (id, comp) in block_msg.new_pairs.iter() {
        pairs
            .entry(id.clone())
            .or_insert_with(|| comp.tokens.clone());
    }

    for (id, state) in block_msg.states.iter() {
        if let Some(tokens) = pairs.get(id) {
            let formatted_token_str = format!("{:}/{:}", &tokens[0].symbol, &tokens[1].symbol);
            info!("Amount out for {}: calculating for tokens {:?}", id, formatted_token_str);
            state
                .spot_price(&tokens[0], &tokens[1])
                .map(|price| info!("Spot price {:?}: {:?}", formatted_token_str, price))
                .into_diagnostic()
                .wrap_err(format!("Error calculating spot price for Pool {id:?}."))?;

            // Test get_amount_out with different percentages of limits. The reserves or limits are
            // relevant because we need to know how much to test with. We don't know if a pool is
            // going to revert with 10 or 10 million USDC, for example, so by using the limits we
            // can use "safe values" where the sim shouldn't break.
            // We then retrieve the amount out for 0.1%, 1% and 10%.
            let percentages = [0.001, 0.01, 0.1];

            // Test all permutations of swap directions
            let swap_directions: Vec<_> = tokens
                .iter()
                .permutations(2)
                .map(|perm| (perm[0], perm[1]))
                .collect();

            for (token_in, token_out) in &swap_directions {
                let (max_input, max_output) = state
                    .get_limits(token_in.address.clone(), token_out.address.clone())
                    .into_diagnostic()
                    .wrap_err(format!(
                        "Error getting limits for Pool {id:?} for in token: {}, and out token: {}",
                        token_in.address, token_out.address
                    ))?;

                info!(
                    "Retrieved limits. | Max input: {max_input} {} | Max output: {max_output} {}",
                    token_in.symbol, token_out.symbol
                );

                for percentage in &percentages {
                    // For precision, multiply by 1000 then divide by 1000
                    let percentage_biguint = BigUint::from((percentage * 1000.0) as u32);
                    let thousand = BigUint::from(1000u32);
                    let amount_in = (&max_input * &percentage_biguint) / &thousand;

                    // Skip if amount is zero
                    if amount_in.is_zero() {
                        info!("Amount in multiplied by percentage {percentage} is zero. Skipping pool {id}.");
                        continue;
                    }

                    let amount_out_result = state
                        .get_amount_out(amount_in.clone(), token_in, token_out)
                        .into_diagnostic()
                        .wrap_err(format!(
                            "Error calculating amount out for Pool {id:?} at {:.1}% with input of {amount_in} {}.",
                            percentage * 100.0,
                            token_in.symbol,
                        ))?;

                    info!(
                        "Amount out for trading {:.1}% of max: ({} {} -> {} {}) (gas: {})",
                        percentage * 100.0,
                        amount_in,
                        token_in.symbol,
                        amount_out_result.amount,
                        token_out.symbol,
                        amount_out_result.gas
                    );

                    let protocol_component = block_msg.new_pairs.get(id).unwrap();

                    let (calldata, solution) = encode_swap(
                        protocol_component.clone(),
                        token_in.address.clone(),
                        token_out.address.clone(),
                        amount_in,
                        amount_out_result.amount,
                    )
                    .into_diagnostic()
                    .wrap_err("Failed to encode swap")?;
                    info!("Encoded swap successfully");

                    // Simulate the trade using eth_call with overwrites
                    let simulation_result = rt.block_on(simulate_trade_with_eth_call(
                        &rpc_provider,
                        &calldata,
                        &solution,
                        stop_block,
                        adapter_contract_path_str,
                        &block_header,
                    ));

                    match simulation_result {
                        Ok(_) => {
                            info!(
                                "Trade simulation successful for {} -> {}",
                                token_in.symbol, token_out.symbol
                            );
                        }
                        Err(e) => {
                            info!(
                                "Trade simulation failed for {} -> {}: {}",
                                token_in.symbol, token_out.symbol, e
                            );
                        }
                    }
                }
            }
        }
    }

    info!("✅ Simulation validation passed");
    Ok(())
}

/// Validate that the token balances of the components match the values
/// on-chain, extracted by querying the token balances using a node.
fn validate_token_balances(
    components_by_id: &HashMap<String, ProtocolComponent>,
    protocol_states_by_id: &HashMap<String, ResponseProtocolState>,
    start_block: u64,
    rt: &Runtime,
) -> miette::Result<()> {
    let rpc_url = env::var("RPC_URL")
        .into_diagnostic()
        .wrap_err("Missing RPC_URL in environment")?;
    let rpc_provider = RPCProvider::new(rpc_url.to_string());

    for (id, component) in components_by_id.iter() {
        let component_state = protocol_states_by_id.get(id);

        for token in &component.tokens {
            let mut balance: U256 = U256::from(0);

            if let Some(state) = component_state {
                let bal = state.balances.get(token);
                if let Some(bal) = bal {
                    let bal = bal.clone().into();
                    balance = bytes_to_u256(bal);
                }
            }

            info!("Validating token balance for component {} and token {}", component.id, token);
            let token_address = alloy::primitives::Address::from_slice(&token[..20]);
            let component_address = alloy::primitives::Address::from_str(component.id.as_str())
                .expect("Failed to parse component address");
            let node_balance = rt.block_on(rpc_provider.get_token_balance(
                token_address,
                component_address,
                start_block,
            ))?;
            if balance != node_balance {
                return Err(miette!(
                    "Token balance mismatch for component {} and token {}",
                    component.id,
                    token
                ));
            }
            info!(
                "Token balance for component {} and token {} matches the expected value",
                component.id, token
            );
        }
    }
    Ok(())
}

/// Load executor bytecode from the appropriate file based on solution component
fn load_executor_bytecode(solution: &Solution) -> miette::Result<Vec<u8>> {
    let first_swap = solution.swaps.first().unwrap();
    let component_id = &first_swap.component;

    let executor_file = get_executor_file(&component_id.protocol_system)?;
    let executor_path = PathBuf::from("../evm/test/executors").join(executor_file);
    info!("Loading executor bytecode from: {}", executor_path.display());

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
        if bytecode_str.starts_with("0x") { &bytecode_str[2..] } else { bytecode_str };

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

/// Create execution transaction (no separate approval needed with storage manipulation)
fn create_execution_transaction(
    transaction: &tycho_simulation::tycho_execution::encoding::models::Transaction,
    _solution: &Solution,
    block_header: &Block,
    user_address: Address,
) -> miette::Result<TransactionRequest> {
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

    Ok(execution_tx)
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
        info!("Added bytecode override for executor: {:?}", executor_address);
        info!("Executor bytecode size: {} bytes", executor_bytecode.len());
    }

    // Add balance override for the user to ensure they have enough tokens
    let token_balance_slot = calculate_balance_slot(user_address);
    let token_approval_slot =
        calculate_approval_slot(user_address, Address::from_slice(&transaction.to[..20])); // TychoRouter address

    state_overwrites.insert(
        token_address,
        StateOverride::new()
            .with_state_diff(
                token_balance_slot,
                alloy::primitives::Bytes::from(U256::MAX.to_be_bytes::<32>().to_vec()),
            )
            .with_state_diff(
                token_approval_slot,
                alloy::primitives::Bytes::from(U256::MAX.to_be_bytes::<32>().to_vec()),
            ),
    );
    info!("Added balance override for user {:?} on token {:?}", user_address, token_address);
    info!(
        "Added approval override for TychoRouter {:?} on token {:?}",
        Address::from_slice(&transaction.to[..20]),
        token_address
    );

    // Add ETH balance override for the user to ensure they have enough gas funds
    state_overwrites.insert(
        user_address,
        StateOverride::new().with_balance(U256::from_str("100000000000000000000").unwrap()), // 100 ETH
    );
    info!("Added ETH balance override for user {:?}", user_address);

    Ok(state_overwrites)
}

/// Calculate the storage slot for ERC20 balances
/// Formula: keccak256(owner || balances_slot)
/// For standard ERC20, balances are typically at slot 0
///
/// TODO double check this - what to do for special cases where allowances are not at slot 1?
///  there is a special case for this in the simulation repo I think.
fn calculate_balance_slot(owner: Address) -> alloy::primitives::Bytes {
    let mut hasher = Keccak256::new();
    hasher.update(owner.as_slice());
    hasher.update([0u8; 31]);
    hasher.update([0u8]); // balances mapping is typically at slot 0
    alloy::primitives::Bytes::from(hasher.finalize().to_vec())
}

/// Calculate the storage slot for ERC20 allowances
/// Formula: keccak256(keccak256(owner || allowances_slot) || spender)
/// For standard ERC20, allowances are typically at slot 1
///
/// TODO double check this - what to do for special cases where allowances are not at slot 1?
///  there is a special case for this in the simulation repo I think.
fn calculate_approval_slot(owner: Address, spender: Address) -> alloy::primitives::Bytes {
    // First hash: keccak256(owner || slot)
    let mut first_hasher = Keccak256::new();
    first_hasher.update(owner.as_slice());
    first_hasher.update([0u8; 31]);
    first_hasher.update([1u8]); // allowances mapping is typically at slot 1
    let first_hash = first_hasher.finalize();

    // Second hash: keccak256(first_hash || spender)
    let mut second_hasher = Keccak256::new();
    second_hasher.update(first_hash);
    second_hasher.update(spender.as_slice());
    alloy::primitives::Bytes::from(second_hasher.finalize().to_vec())
}

/// Simulate a trade using eth_call for historical blocks
async fn simulate_trade_with_eth_call(
    rpc_provider: &RPCProvider,
    transaction: &tycho_simulation::tycho_execution::encoding::models::Transaction,
    solution: &Solution,
    block_number: u64,
    _adapter_contract_path: &str,
    block_header: &Block,
) -> miette::Result<()> {
    let executor_bytecode = load_executor_bytecode(solution)?;
    let user_address = Address::from_slice(&solution.sender[..20]);
    let execution_tx =
        create_execution_transaction(transaction, solution, block_header, user_address)?;
    let tycho_router_address = Address::from_slice(&transaction.to[..20]);
    let token_address = Address::from_slice(&solution.given_token[..20]);

    // Copy router storage and code from current block to historical block
    // TODO get this at compile time.
    let router_bytecode_path = "../evm/test/router/TychoRouter.runtime.json";
    info!(
        "Copying router contract storage and code from current block to block {}",
        block_number
    );

    let router_override = rpc_provider
        .copy_contract_storage_and_code(
            tycho_router_address,
            router_bytecode_path,
        )
        .await
        .wrap_err("Failed to copy router contract storage and code")?;

    // Set up state overrides including router override
    let mut state_overwrites =
        setup_state_overrides(solution, transaction, user_address, &executor_bytecode, true)?; // Include executor override for historical blocks

    // Add the router override
    state_overwrites.insert(tycho_router_address, router_override);
    info!("Added router contract override for address {:?}", tycho_router_address);

    info!("Simulating at historical block {}", block_number);
    // TODO decode this result into some nice logs.
    let _execution_result = rpc_provider
        .eth_call_with_overwrites(execution_tx, block_number, state_overwrites)
        .await
        .map_err(|e| {
            info!("Execution transaction failed with error: {}", e);
            e
        })
        .wrap_err("Execution simulation failed")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use dotenv::dotenv;
    use glob::glob;
    use tycho_common::{
        dto::{ProtocolComponent, ResponseProtocolState},
        Bytes,
    };

    use super::*;

    #[test]
    fn test_parse_all_configs() {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let curr_dir = PathBuf::from(manifest_dir);
        let parent_dir = curr_dir.parent().unwrap();
        env::set_current_dir(parent_dir).expect("Failed to set working directory");

        let pattern = "./substreams/*/integration_test.tycho.yaml";
        let mut results = Vec::new();

        if glob(pattern).unwrap().count() == 0 {
            panic!("No integration_test.tycho.yaml files found in substreams/*/");
        }
        for entry in glob(pattern).unwrap() {
            match entry {
                Ok(path) => {
                    if !path.is_file() {
                        results.push(Err(format!("Path is not a file: {}", path.display())));
                    } else {
                        let result = TestRunner::parse_config(&path);
                        if let Err(e) = &result {
                            results.push(Err(format!(
                                "Failed to parse config at {}: {e:?}",
                                path.display(),
                            )));
                        } else {
                            results.push(Ok(()));
                        }
                    }
                }
                Err(e) => results.push(Err(format!("Glob error: {e:?}"))),
            }
        }

        let errors: Vec<_> = results
            .iter()
            .filter_map(|r| r.as_ref().err())
            .collect();
        if !errors.is_empty() {
            for error in errors {
                println!("{error}");
            }
            panic!("One or more config files failed to parse.");
        }
    }

    #[test]
    fn test_token_balance_validation() {
        // Setup mock data
        let block_number = 21998530;
        let token_bytes = Bytes::from_str("0x0000000000000000000000000000000000000000").unwrap();
        let component_id = "0x787B8840100d9BaAdD7463f4a73b5BA73B00C6cA".to_string();

        let component = ProtocolComponent {
            id: component_id.clone(),
            tokens: vec![token_bytes.clone()],
            ..Default::default()
        };

        let mut balances = HashMap::new();
        let balance_bytes = Bytes::from(
            U256::from_str("1070041574684539264153")
                .unwrap()
                .to_be_bytes::<32>(),
        );
        balances.insert(token_bytes.clone(), balance_bytes.clone());
        let protocol_state = ResponseProtocolState {
            component_id: component_id.clone(),
            balances,
            ..Default::default()
        };

        let mut components_by_id = HashMap::new();
        components_by_id.insert(component_id.clone(), component.clone());
        let mut protocol_states_by_id = HashMap::new();
        protocol_states_by_id.insert(component_id.clone(), protocol_state.clone());

        let rt = Runtime::new().unwrap();
        dotenv().ok();
        let result =
            validate_token_balances(&components_by_id, &protocol_states_by_id, block_number, &rt);
        assert!(result.is_ok(), "Should pass when balance check is performed and balances match");
    }

    #[test]
    fn test_token_balance_validation_fails_on_mismatch() {
        // Setup mock data
        let block_number = 21998530;
        let token_bytes = Bytes::from_str("0x0000000000000000000000000000000000000000").unwrap();
        let component_id = "0x787B8840100d9BaAdD7463f4a73b5BA73B00C6cA".to_string();

        let component = ProtocolComponent {
            id: component_id.clone(),
            tokens: vec![token_bytes.clone()],
            ..Default::default()
        };

        // Set expected balance to zero
        let mut balances = HashMap::new();
        let balance_bytes = Bytes::from(U256::from(0).to_be_bytes::<32>());
        balances.insert(token_bytes.clone(), balance_bytes.clone());
        let protocol_state = ResponseProtocolState {
            component_id: component_id.clone(),
            balances,
            ..Default::default()
        };

        let mut components_by_id = HashMap::new();
        components_by_id.insert(component_id.clone(), component.clone());
        let mut protocol_states_by_id = HashMap::new();
        protocol_states_by_id.insert(component_id.clone(), protocol_state.clone());

        let rt = Runtime::new().unwrap();
        dotenv().ok();
        let result =
            validate_token_balances(&components_by_id, &protocol_states_by_id, block_number, &rt);
        assert!(
            result.is_err(),
            "Should fail when balance check is performed and balances do not match"
        );
    }
}
