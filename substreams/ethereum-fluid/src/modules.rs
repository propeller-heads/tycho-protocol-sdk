//! Template for Protocols with contract factories
//!
//! This template provides foundational maps and store substream modules for indexing a
//! protocol where each component (e.g., pool) is deployed to a separate contract. Each
//! contract is expected to escrow its ERC-20 token balances.
//!
//! If your protocol supports native ETH, you may need to adjust the balance tracking
//! logic in `map_relative_component_balance` to account for native token handling.
//!
//! ## Assumptions
//! - Assumes each pool has a single newly deployed contract linked to it
//! - Assumes pool identifier equals the deployed contract address
//! - Assumes any price or liquidity updated correlates with a pools contract storage update.
//!
//! ## Alternative Module
//! If your protocol uses a vault-like contract to manage balances, or if pools are
//! registered within a singleton contract, refer to the `ethereum-template-singleton`
//! substream for an appropriate alternative.
//!
//! ## Warning
//! This template provides a general framework for indexing a protocol. However, it is
//! likely that you will need to adapt the steps to suit your specific use case. Use the
//! provided code with care and ensure you fully understand each step before proceeding
//! with your implementation.
//!
//! ## Example Use Case
//! For an Uniswap-like protocol where each liquidity pool is deployed as a separate
//! contract, you can use this template to:
//! - Track relative component balances (e.g., ERC-20 token balances in each pool).
//! - Index individual pool contracts as they are created by the factory contract.
//!
//! Adjustments to the template may include:
//! - Handling native ETH balances alongside token balances.
//! - Customizing indexing logic for specific factory contract behavior.

use std::{str::FromStr};
use crate::abi::factory::events::LogDexDeployed;
use anyhow::{Ok, Result};
use itertools::Itertools;
use std::collections::HashMap;
use substreams::{hex, log_info, pb::substreams::StoreDeltas, prelude::*};
use substreams_ethereum::pb::eth;
use tycho_substreams::{balances::{aggregate_balances_changes}, contract::extract_contract_changes_builder,
                       prelude::*,
};

use ethabi::ethereum_types::Address;
use substreams_helper::{event_handler::EventHandler, hex::Hexable};

pub const LIQUIDITY_CONTRACT_ADDRESS: &[u8] = &hex!("52Aa899454998Be5b000Ad077a46Bbe360F4e497");
// TODO: Derive this via an event: https://github.com/Instadapp/fluid-contracts-public/blob/main/contracts/infiniteProxy/events.sol#L12-L15
pub const USER_MODULE_IMPL_ADDRESS: &[u8] = &hex!("6967e68F7f9b3921181f27E66Aa9c3ac7e13dBc0");
// TODO: make this configurable for multichain support
pub const DEX_RESERVES_RESOLVER: &[u8] = &hex!("b387f9C2092cF7c4943F97842887eBff7AE96EB3");

/// Find and create all relevant protocol components
///
/// This method maps over blocks and instantiates ProtocolComponents with a unique ids
/// as well as all necessary metadata for routing and encoding.
#[substreams::handlers::map]
pub fn map_dex_deployed(
    params: String,
    block: eth::v2::Block,
) -> Result<BlockTransactionProtocolComponents> {
    let mut new_dexes = vec![];
    let factory_address = params.as_str();

    get_new_dexes(&block, &mut new_dexes, factory_address);

    Ok(BlockTransactionProtocolComponents { tx_components: new_dexes })
}

fn get_new_dexes(
    block: &eth::v2::Block,
    new_dexes: &mut Vec<TransactionProtocolComponents>,
    factory_address: &str,
) {
    // Extract new dex pools from LogDexDeployed events
    let mut on_dex_deployed =
        |event: LogDexDeployed, tx: &eth::v2::TransactionTrace, _log: &eth::v2::Log| {
            let tycho_tx: Transaction = tx.into();
            let constants_view_rpc_call = crate::abi::dex_implementation::functions::ConstantsView {};
            let constants_view = constants_view_rpc_call.call(event.dex.clone()).unwrap();
            let implementations = constants_view.3;

            new_dexes.push(TransactionProtocolComponents {
                tx: Some(tycho_tx.clone()),
                components: vec![ProtocolComponent {
                    id: event.dex.to_hex(),
                    tokens: vec![
                        hex!("4956b52aE2fF65D74CA2d61207523288e4528f96").to_vec(),
                        hex!("A0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48").to_vec()
                    ],
                    contracts: vec![
                        LIQUIDITY_CONTRACT_ADDRESS.to_vec(),
                        USER_MODULE_IMPL_ADDRESS.to_vec(),
                        DEX_RESERVES_RESOLVER.to_vec(),
                        event.dex.clone(),
                        implementations.2.to_vec(),
                        implementations.3.to_vec(),
                        implementations.4.to_vec(),
                    ],
                    static_att: vec![
                        Attribute {
                            name: "dex_id".to_string(),
                            value: BigInt::from(event.dex_id).to_signed_bytes_be(),
                            change: ChangeType::Creation.into(),
                        },
                        Attribute {
                            name: "dex_address".to_string(),
                            value: event.dex,
                            change: ChangeType::Creation.into(),
                        },
                        Attribute {
                            name: "manual_updates".to_string(),
                            value: vec![1u8],
                            change: ChangeType::Creation.into(),
                        },
                        Attribute {
                            name: "supply_token_0_slot".to_string(),
                            value: constants_view.7.to_vec(),
                            change: ChangeType::Creation.into(),
                        },
                        Attribute {
                            name: "borrow_token_0_slot".to_string(),
                            value: constants_view.8.to_vec(),
                            change: ChangeType::Creation.into(),
                        },
                        Attribute {
                            name: "supply_token_1_slot".to_string(),
                            value: constants_view.9.to_vec(),
                            change: ChangeType::Creation.into(),
                        },
                        Attribute {
                            name: "borrow_token_1_slot".to_string(),
                            value: constants_view.10.to_vec(),
                            change: ChangeType::Creation.into(),
                        },
                        Attribute {
                            name: "exchange_price_token_0_slot".to_string(),
                            value: constants_view.11.to_vec(),
                            change: ChangeType::Creation.into(),
                        },
                        Attribute {
                            name: "exchange_price_token_1_slot".to_string(),
                            value: constants_view.12.to_vec(),
                            change: ChangeType::Creation.into(),
                        },
                    ],
                    change: i32::from(ChangeType::Creation),
                    protocol_type: Some(ProtocolType {
                        name: "fluid_dex_pool".to_string(),
                        financial_type: FinancialType::Swap.into(),
                        attribute_schema: vec![],
                        implementation_type: ImplementationType::Vm.into(),
                    }),
                    tx: Some(tycho_tx.clone()),
                }],
            })
        };

    let mut eh = EventHandler::new(block);

    eh.filter_by_address(vec![Address::from_str(factory_address).unwrap()]);

    eh.on::<LogDexDeployed, _>(&mut on_dex_deployed);
    eh.handle_events();
}

/// Stores all contracts involved in quoting.
#[substreams::handlers::store]
pub fn store_contract_addresses(
    dexes_deployed: BlockTransactionProtocolComponents,
    store: StoreSetInt64,
) {
    for change in dexes_deployed.tx_components {
        for new_protocol_component in change.components.iter() {
            let addresses = new_protocol_component
                .contracts
                .iter()
                .map(hex::encode)
                .collect();
            store.set_many(0, &addresses, &1i64);
        }
    }
}

/// Aggregates protocol components and balance changes by transaction.
///
/// This is the main method that will aggregate all changes as well as extract all
/// relevant contract storage deltas.
///
/// ## Note:
/// You may have to change this method if your components have any default dynamic
/// attributes, or if you need any additional static contracts indexed.
#[substreams::handlers::map]
fn map_protocol_changes(
    block: eth::v2::Block,
    new_components: BlockTransactionProtocolComponents,
    contracts_store: StoreGetRaw,
) -> Result<BlockChanges, substreams::errors::Error> {
    // We merge contract changes by transaction (identified by transaction index)
    // making it easy to sort them at the very end.
    let mut transaction_changes: HashMap<_, TransactionChangesBuilder> = HashMap::new();

    // Aggregate newly created components per tx
    new_components
        .tx_components
        .iter()
        .for_each(|tx_component| {
            // initialise builder if not yet present for this tx
            let tx = tx_component.tx.as_ref().unwrap();
            let builder = transaction_changes
                .entry(tx.index)
                .or_insert_with(|| TransactionChangesBuilder::new(tx));

            // iterate over individual components created within this tx
            tx_component
                .components
                .iter()
                .for_each(|component| {
                    builder.add_protocol_component(component);
                });
        });

    // Extract and insert any storage changes that happened for any of the components.
    extract_contract_changes_builder(
        &block,
        |addr| {
            contracts_store
                .get_last(hex::encode(addr))
                .is_some()
        },
        &mut transaction_changes,
    );

    // Process all `transaction_changes` for final output in the `BlockChanges`,
    //  sorted by transaction index (the key).
    Ok(BlockChanges {
        block: Some((&block).into()),
        changes: transaction_changes
            .drain()
            .sorted_unstable_by_key(|(index, _)| *index)
            .filter_map(|(_, builder)| builder.build())
            .collect::<Vec<_>>(),
    })
}
