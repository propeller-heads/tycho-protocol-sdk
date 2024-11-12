use crate::abi::{self};
use anyhow::Result;
use itertools::Itertools;
use std::collections::HashMap;
use substreams::{
    hex,
    pb::substreams::StoreDeltas,
    scalar::BigInt as ScalarBigInt,
    store::{StoreAddBigInt, StoreGet, StoreGetString, StoreNew, StoreSet, StoreSetString},
};
use substreams_ethereum::{pb::eth, Event};
use tycho_substreams::{
    balances::aggregate_balances_changes, contract::extract_contract_changes_builder, prelude::*,
};

pub const ETH_ADDRESS: [u8; 20] = [238u8; 20]; // 0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee

#[substreams::handlers::map]
pub fn map_components(
    params: String,
    block: eth::v2::Block,
) -> Result<BlockTransactionProtocolComponents, anyhow::Error> {
    let network_address = hex::decode(params).unwrap();
    let vault_address = get_master_vault_address(&network_address).unwrap();
    // We store these as a hashmap by tx hash since we need to agg by tx hash later
    Ok(BlockTransactionProtocolComponents {
        tx_components: block
            .transactions()
            .filter_map(|tx| {
                let components = tx
                    .calls()
                    .filter(|call| !call.call.state_reverted)
                    .filter_map(|_| {
                        // address doesn't exist before contract deployment, hence the first tx with
                        // a log.address = vault_address is the deployment tx
                        if is_deployment_tx(tx, &vault_address) {
                            Some(
                                ProtocolComponent::at_contract(&vault_address, &tx.into())
                                    .as_swap_type("bancor_master_vault", ImplementationType::Vm),
                            )
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();

                if !components.is_empty() {
                    Some(TransactionProtocolComponents { tx: Some(tx.into()), components })
                } else {
                    None
                }
            })
            .collect::<Vec<_>>(),
    })
}

/// Simply stores the `ProtocolComponent`s with the pool address as the key and the pool id as value
#[substreams::handlers::store]
pub fn store_components(
    map: BlockTransactionProtocolComponents, // Input: contains transactions with their components
    store: StoreSetString,                   // Store interface for setting string values
) {
    map.tx_components // Get all transactions that have components
        .into_iter() // Convert into an iterator
        .for_each(|tx_pc| {
            // For each transaction with components...
            tx_pc
                .components // Get the components from this transaction
                .into_iter()
                .for_each(|pc| {
                    // For each component
                    let key = format!("pool:{0}", &pc.id[..42]);
                    substreams::log::info!(
                        "Storing component with key: {}, value: {}",
                        key,
                        &pc.id
                    );
                    store.set(
                        0, /* Starting ordinal (always 0 in this
                            * case) */
                        format!("pool:{0}", &pc.id[..42]), /* Key: "pool:" prefix + first 42
                                                            * chars of component ID */
                        &pc.id, // Value: the full component ID
                    )
                });
        });
}

/// we need to leverage a
/// map and a  store to be able to tally up final balances for tokens in a pool.
#[substreams::handlers::map]
pub fn map_relative_balances(
    block: eth::v2::Block,
    store: StoreGetString, // Input parameter: Store interface for retrieving strings
) -> Result<BlockBalanceDeltas, anyhow::Error> {
    // Iterate for transactions (find ETH transfers)
    let balance_deltas = block
        .transactions() // Get all the logs from the block
        .flat_map(|tx| {
            // Transform each log into multiple BalanceDeltas
            let mut deltas = Vec::new(); // Create an empty vector to store deltas

            // convert tx value to scalar BigInt
            let tx_value = ScalarBigInt::from_signed_bytes_be(&tx.value.as_ref().unwrap().bytes);

            // lookup for this tx's executor in store
            let address_bytes_be_from = tx.from.to_vec();
            let address_hex_from = format!("0x{}", hex::encode(&address_bytes_be_from));

            let address_bytes_be_to = tx.to.to_vec();
            let address_hex_to = format!("0x{}", hex::encode(&address_bytes_be_to));

            let mut component_id = Vec::new();
            if store
                .get_last(format!("pool:{}", address_hex_from))
                .is_some()
            {
                component_id = address_bytes_be_to.clone();
            } else {
                if store
                    .get_last(format!("pool:{}", address_hex_to))
                    .is_some()
                {
                    component_id = address_bytes_be_from.clone();
                }
            }

            if !component_id.is_empty() {
                let mut tx_trace = None;
                // detect ERC20 transfers
                tx.logs_with_calls()
                    .for_each(|(log, _)| {
                        if let Some(transfer) = abi::erc20::events::Transfer::match_and_decode(log)
                        {
                            if transfer.from.to_vec() == component_id {
                                tx_trace = Some(tx.into());
                                deltas.push(BalanceDelta {
                                    ord: tx.begin_ordinal,       // Event ordering number
                                    tx: Some(tx.into()),         // Transaction receipt
                                    token: log.address.to_vec(), // Source token address
                                    delta: transfer
                                        .value
                                        .neg()
                                        .to_signed_bytes_be(),
                                    component_id: component_id.clone(), // Component ID
                                });
                            } else if transfer.to.to_vec() == component_id {
                                tx_trace = Some(tx.into());
                                deltas.push(BalanceDelta {
                                    ord: tx.begin_ordinal,       // Event ordering number
                                    tx: Some(tx.into()),         // Transaction receipt
                                    token: log.address.to_vec(), // Destination token address
                                    delta: transfer.value.to_signed_bytes_be(),
                                    component_id: component_id.clone(), // Component ID
                                });
                            }
                        }

                        // detect ETH transfers
                        if tx_trace.is_some() && tx_value.gt(&ScalarBigInt::zero()) {
                            if tx.to.to_vec() == component_id {
                                deltas.push(BalanceDelta {
                                    ord: tx.begin_ordinal,       // Event ordering number
                                    tx: tx_trace.clone(),        // Transaction receipt
                                    token: ETH_ADDRESS.to_vec(), // Source token address
                                    delta: tx_value.to_signed_bytes_be(),
                                    component_id: component_id.clone(), // Component ID
                                });
                            } else if tx.from.to_vec() == component_id {
                                deltas.push(BalanceDelta {
                                    ord: tx.begin_ordinal,       /* Event ordering
                                                                  * number */
                                    tx: tx_trace.clone(), // Transaction receipt
                                    token: ETH_ADDRESS.to_vec(), // Source token address
                                    delta: tx_value.neg().to_signed_bytes_be(), /* Source token
                                                           * amount as sygned
                                                           * bytes */
                                    component_id: component_id.clone(), // Component ID
                                });
                            }
                        }
                    });
            }

            deltas // Return the vector of deltas for this log
        })
        .collect::<Vec<_>>();

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

fn is_deployment_tx(tx: &eth::v2::TransactionTrace, vault_address: &[u8]) -> bool {
    let created_accounts = tx
        .calls
        .iter()
        .flat_map(|call| {
            call.account_creations
                .iter()
                .map(|ac| ac.account.to_owned())
        })
        .collect::<Vec<_>>();

    if let Some(deployed_address) = created_accounts.first() {
        return deployed_address.as_slice() == vault_address;
    }
    false
}

fn get_master_vault_address(network_address: &[u8]) -> Option<[u8; 20]> {
    match network_address {
        hex!("eEF417e1D5CC832e619ae18D2F140De2999dD4fB") => {
            Some(hex!("2f14750b0d267be47dcd30a134796c2e4b1638a3"))
        }
        _ => None,
    }
}
