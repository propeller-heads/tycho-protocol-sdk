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

use std::{fmt::format, str::FromStr};
use crate::abi::factory::events::LogDexDeployed;
use anyhow::{Ok, Result};
use itertools::Itertools;
use std::collections::HashMap;
use substreams::{hex, pb::substreams::StoreDeltas, prelude::*};
use substreams_ethereum::pb::eth;
use tycho_substreams::{ balances::{aggregate_balances_changes, extract_balance_deltas_from_tx}, contract::extract_contract_changes_builder,
    prelude::*,
};

use ethabi::ethereum_types::Address;
use substreams_helper::{event_handler::EventHandler, hex::Hexable};

pub const LIQUIDITY_CONTRACT_ADDRESS: &[u8] = &hex!("52Aa899454998Be5b000Ad077a46Bbe360F4e497");

/// Find and create all relevant protocol components
///
/// This method maps over blocks and instantiates ProtocolComponents with a unique ids
/// as well as all necessary metadata for routing and encoding.
#[substreams::handlers::map]
pub fn map_dex_deployed(
    params: String,
    block: eth::v2::Block,
) -> Result<BlockChanges, substreams::errors::Error> {
    let mut new_dexes = vec![];
    let factory_address = params.as_str();

    get_new_dexes(&block, &mut new_dexes, factory_address);

    Ok(BlockChanges { block: Some((&block).into()), changes: new_dexes })
}

fn get_new_dexes(
    block: &eth::v2::Block,
    new_dexes: &mut Vec<TransactionChanges>,
    factory_address: &str,
) {
    // Extract new dex pools from LogDexDeployed events
    let mut on_dex_deployed =
        |event: LogDexDeployed, _tx: &eth::v2::TransactionTrace, _log: &eth::v2::Log| {
            let tycho_tx: Transaction = _tx.into();

            new_dexes.push(TransactionChanges {
                tx: Some(tycho_tx.clone()),
                contract_changes: vec![],
                entity_changes: vec![],
                component_changes: vec![ProtocolComponent {
                    id: event.dex.to_hex(),
                    tokens: vec![],
                    contracts: vec![],
                    static_att: vec![
                        Attribute {
                            name: "dex_id".to_string(),
                            value: BigInt::from(event.dex_id).to_signed_bytes_be(),
                            change: ChangeType::Creation.into(),
                        },
                        Attribute {
                            name: "dex_address".to_string(),
                            value: event.dex.clone(),
                            change: ChangeType::Creation.into(),
                        },
                        Attribute {
                            name: "balance_owner".to_string(),
                            value: LIQUIDITY_CONTRACT_ADDRESS.to_vec(),
                            change: ChangeType::Creation.into(),
                        }
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
                balance_changes: vec![],
            })
        };

    let mut eh = EventHandler::new(block);

    eh.filter_by_address(vec![Address::from_str(factory_address).unwrap()]);

    eh.on::<LogDexDeployed, _>(&mut on_dex_deployed);
    eh.handle_events();
}

/// Stores all dex pools in a store with dex address as key and corresponding protocol component as value.
#[substreams::handlers::store]
pub fn store_dexes(
    dexes_deployed: BlockChanges,
    store: StoreSetIfNotExistsProto<ProtocolComponent>,
) {
    for change in dexes_deployed.changes {
        for new_protocol_component in change.component_changes {
            store.set_if_not_exists(
                0,
                &new_protocol_component.id,
                &new_protocol_component,
            );
        }
    }
}

/// Extracts balance changes per component
///
/// This template function uses ERC20 transfer events to extract balance changes. It
/// assumes that each component is deployed at a dedicated contract address. If a
/// transfer to the component is detected, it's balanced is increased and if a balance
/// from the component is detected its balance is decreased.
///
/// ## Note:
/// Changes are necessary if your protocol uses native ETH, uses a vault contract or if
/// your component burn or mint tokens without emitting transfer events.
///
/// You may want to ignore LP tokens if your protocol emits transfer events for these
/// here.
#[substreams::handlers::map]
fn map_relative_component_balance(
    block: eth::v2::Block,
    store: StoreGetRaw,
) -> Result<BlockBalanceDeltas, anyhow::Error>{

}

/// Aggregates relative balances values into absolute values
///
/// Aggregate the relative balances in an additive store since tycho-indexer expects
/// absolute balance inputs.
///
/// ## Note:
/// This method should usually not require any changes.
#[substreams::handlers::store]
pub fn store_balances(deltas: BlockBalanceDeltas, store: StoreAddBigInt) {
    tycho_substreams::balances::store_balance_changes(deltas, store);
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
    components_store: StoreGetRaw,
    balance_store: StoreDeltas,
    deltas: BlockBalanceDeltas,
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
                    // TODO: In case you require to add any dynamic attributes to the
                    //  component you can do so here:
                    /*
                        builder.add_entity_change(&EntityChanges {
                            component_id: component.id.clone(),
                            attributes: default_attributes.clone(),
                        });
                    */
                });
        });

    // Aggregate absolute balances per transaction.
    aggregate_balances_changes(balance_store, deltas)
        .into_iter()
        .for_each(|(_, (tx, balances))| {
            let builder = transaction_changes
                .entry(tx.index)
                .or_insert_with(|| TransactionChangesBuilder::new(&tx));
            balances
                .values()
                .for_each(|token_bc_map| {
                    token_bc_map
                        .values()
                        .for_each(|bc| builder.add_balance_change(bc))
                });
        });

    // Extract and insert any storage changes that happened for any of the components.
    extract_contract_changes_builder(
        &block,
        |addr| {
            // we assume that the store holds contract addresses as keys and if it
            // contains a value, that contract is of relevance.
            // TODO: if you have any additional static contracts that need to be indexed,
            //  please add them here.
            components_store
                .get_last(hex::encode(addr))
                .is_some() ||
                addr.eq(LIQUIDITY_CONTRACT_ADDRESS)
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
