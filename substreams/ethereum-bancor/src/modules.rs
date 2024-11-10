use crate::abi::pool_contract::events::{PoolAdded, TokensTraded};
use anyhow::Result;
use itertools::Itertools;
use std::collections::HashMap;
use substreams::{
    hex,
    pb::substreams::StoreDeltas,
    store::{
        StoreAddBigInt, StoreGet, StoreNew, StoreSet, StoreSetString, StoreGetString
    },
};
use substreams_ethereum::{
    pb::{
        eth,
        eth::v2::{Log, TransactionTrace},
    },
    Event,
};
use tycho_substreams::{
    balances::aggregate_balances_changes, contract::extract_contract_changes_builder, prelude::*,
};

pub const FACTORY_ADDRESS: [u8; 20] = hex!("eEF417e1D5CC832e619ae18D2F140De2999dD4fB");
pub const BNT_ADDRESS: [u8; 20] = hex!("1F573D6Fb3F13d689FF844B4cE37794d79a7FF1C");


// Define a map handler function named `map_components` using Substreams
#[substreams::handlers::map]
// Purpose: Map Ethereum blocks to components by filtering out transactions that are reverted and
// creating a collection of protocol components.
pub fn map_components(
    block: eth::v2::Block,  // Take in an Ethereum block as input
) -> Result<BlockTransactionProtocolComponents, anyhow::Error> { // Return a result with protocol components or an error

    // Return a successful result containing a BlockTransactionProtocolComponents struct
    Ok(BlockTransactionProtocolComponents {
        // Process each transaction in the block to find components related to the protocol
        tx_components: block
            .transactions()  // Access the list of transactions within the block
            .filter_map(|tx| {  // Use filter_map to collect only transactions with relevant components

                // For each transaction, gather logs and call data to find components
                let components = tx
                    .logs_with_calls()  // Get logs along with call details for each transaction
                    .filter(|(_, call)| !call.call.state_reverted)  // Filter out logs where the call was reverted
                    .filter_map(|(log, _)| address_map(&log, &tx))  // Map logs to protocol components, skipping if `address_map` returns None
                    .collect::<Vec<_>>();  // Collect all components for this transaction into a vector

                // If we found any components, return a TransactionProtocolComponents struct for this transaction
                if !components.is_empty() {
                    Some(TransactionProtocolComponents { 
                        tx: Some(tx.into()),  // Convert the transaction data into the expected format
                        components  // Attach the vector of components found for this transaction
                    })
                } else {
                    None  // If no components were found, return None for this transaction
                }
            })
            .collect::<Vec<_>>(),  // Collect all transactions with components into a vector
    })
}


/// Simply stores the `ProtocolComponent`s with the pool address as the key and the pool id as value
#[substreams::handlers::store]
pub fn store_components(
    map: BlockTransactionProtocolComponents,  // Input: contains transactions with their components
    store: StoreSetString                     // Store interface for setting string values
) {
    map.tx_components // Get all transactions that have components
        .into_iter() // Convert into an iterator
        .for_each(|tx_pc| { // For each transaction with components...
            tx_pc
                .components // Get the components from this transaction
                .into_iter()
                .for_each(|pc| { // For each component
                    let key = format!("pool:{0}", &pc.id[..42]);
                    substreams::log::info!("Storing component with key: {}, value: {}", key, &pc.id);
                    store.set(
                        0, // Starting ordinal (always 0 in this case)
                        format!("pool:{0}", &pc.id[..42]), // Key: "pool:" prefix + first 42 chars of component ID
                        &pc.id // Value: the full component ID
                    )
                });
        });
}

/// we need to leverage a
/// map and a  store to be able to tally up final balances for tokens in a pool.
#[substreams::handlers::map]
pub fn map_relative_balances(
    block: eth::v2::Block,
    store: StoreGetString // Input parameter: Store interface for retrieving strings
) -> Result<BlockBalanceDeltas, anyhow::Error> {
    let balance_deltas = block
        .logs() // Get all the logs from the block
        .filter(|log| log.address() == &FACTORY_ADDRESS) // Only keep logs from the FACTORY_ADDRESS
        .flat_map(|log| {   // Transform each log into multiple BalanceDeltas
            let mut deltas = Vec::new(); // Create an empty vector to store deltas

            // Try to decode the log received from `store_components` as a TokenTraded event
            if let Some(event) = TokensTraded::match_and_decode(log) {
                // Convert token addresses to hex strings
                let source_component_id = address_to_hex(&event.source_token);
                let target_component_id = address_to_hex(&event.target_token);

                // Check if both tokens exist in the store
                if let (Some(_source_component), Some(_target_component)) = (
                    store.get_last(format!("pool:{}", source_component_id)),
                    store.get_last(format!("pool:{}", target_component_id)),
                ) {
                    // Create delta for source token (positive amount)
                    deltas.push(BalanceDelta {
                        ord: log.ordinal(), // Event ordering number
                        tx: Some(log.receipt.transaction.into()), // Transaction info
                        token: event.source_token.clone(), // Source token address
                        delta: event.source_amount.to_signed_bytes_be(), // Source token amount as sygned bytes
                        component_id: source_component_id.clone().into_bytes(), // Component ID
                    });

                    // Create delta for target token (negative amount)
                    deltas.push(BalanceDelta {
                        ord: log.ordinal(),
                        tx: Some(log.receipt.transaction.into()),
                        token: event.target_token.clone(),
                        delta: event        // Negate the target amount
                            .target_amount
                            .neg()
                            .to_signed_bytes_be(),
                        component_id: target_component_id.clone().into_bytes(),
                    });
                }
            }

            deltas  // Return the vector of deltas for this log
        })
        .collect::<Vec<_>>();  // Collect all deltas into a vector

    // Return the balance deltas wrapped in BlockBalanceDeltas struct
    Ok(BlockBalanceDeltas { balance_deltas })
}


/// It's significant to include both the `pool_id` and the `token_id` for each balance delta as the
///  store key to ensure that there's a unique balance being tallied for each.
#[substreams::handlers::store]
pub fn store_balances(deltas: BlockBalanceDeltas, store: StoreAddBigInt) {
    tycho_substreams::balances::store_balance_changes(deltas, store);
}

/// This is the main map that handles most of the indexing of this substream.
/// Every contract change is grouped by transaction index via the `transaction_contract_changes`
///  map. Each block of code will extend the `TransactionContractChanges` struct with the
///  corresponding changes (balance, component, contract), inserting a new one if it doesn't exist.
///  At the very end, the map can easily be sorted by index to ensure the final
/// `BlockContractChanges`  is ordered by transactions properly.
#[substreams::handlers::map]
pub fn map_protocol_changes(
    block: eth::v2::Block,
    grouped_components: BlockTransactionProtocolComponents,
    deltas: BlockBalanceDeltas,
    components_store: StoreGetString,
    balance_store: StoreDeltas, // Note, this map module is using the `deltas` mode for the store.
) -> Result<BlockChanges, anyhow::Error> {
    // We merge contract changes by transaction (identified by transaction index) making it easy to
    //  sort them at the very end.
    let mut transaction_changes: HashMap<_, TransactionChangesBuilder> = HashMap::new();

    // `ProtocolComponents` are gathered from `map_pools_created` which just need a bit of work to
    //   convert into `TransactionChanges`
    grouped_components
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

    // Balance changes are gathered by the `StoreDelta` based on `PoolBalanceChanged` creating
    //  `BlockBalanceDeltas`. We essentially just process the changes that occurred to the `store`
    // this  block. Then, these balance changes are merged onto the existing map of tx contract
    // changes,  inserting a new one if it doesn't exist.
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
            components_store
                .get_last(format!("pool:0x{0}", hex::encode(addr)))
                .is_some()
        },
        &mut transaction_changes,
    );

    // Process all `transaction_contract_changes` for final output in the `BlockContractChanges`,
    //  sorted by transaction index (the key).
    let block_changes = BlockChanges {
        block: Some((&block).into()),
        changes: transaction_changes
            .drain()
            .sorted_unstable_by_key(|(index, _)| *index)
            .filter_map(|(_, builder)| builder.build())
            .collect::<Vec<_>>(),
    };

    for change in &block_changes.changes {
        substreams::log::info!("🚨 Balance changes {:?}", change.balance_changes);
        substreams::log::info!("🚨 Component changes {:?}", change.component_changes);
    }
    Ok(block_changes)
}

fn address_to_hex(address: &[u8]) -> String {
    format!("0x{}", hex::encode(address))
}

// Define a function `address_map` that takes a reference to a log and transaction trace
fn address_map(log: &Log, tx: &TransactionTrace) -> Option<ProtocolComponent> {
    
    // Clone the address from the log for comparison
    // Log.address is the address of the contract that emitted the event
    let address = log.address.to_owned();

    // Check if the address matches the specific `FACTORY_ADDRESS`
    if address == FACTORY_ADDRESS {

        // Checks if the log matches the event signature of PoolAdded
        // If it matches, decodes the log data into the PoolAdded struct
        // .map(|pool_added| { ... }) is using Rust's Option combinators:
        // match_and_decode returns an Option<PoolAdded>
        // .map() will only execute if the Option is Some
        // The closure |pool_added| receives the decoded PoolAdded event data
        PoolAdded::match_and_decode(log).map(|pool_added| {

            // Log information about the new pool that was added, displaying the pool's address
            substreams::log::info!("🏦 Pool added for token: 0x{}", hex::encode(&pool_added.pool));

            // Construct a `ProtocolComponent` at the `pool_added` address with the transaction data
            ProtocolComponent::at_contract(&pool_added.pool, &(tx.into()))
                
                // Specify the tokens related to this pool, which include `pool_added.pool` and the `BNT_ADDRESS`
                .with_tokens(&[pool_added.pool, BNT_ADDRESS.to_vec()])
                
                // Set the component type to "bancor_v3_pool" with a swap type of "Vm"
                .as_swap_type("bancor_v3_pool", ImplementationType::Vm)
        })
    } else {
        // If the address does not match `FACTORY_ADDRESS`, return None
        None
    }
}

