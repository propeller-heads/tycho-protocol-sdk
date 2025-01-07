use crate::abi;
use anyhow::Result;
use itertools::Itertools;
use std::collections::HashMap;
use substreams::{
    hex,
    pb::substreams::StoreDeltas,
    store::{StoreAddBigInt, StoreGet, StoreGetString, StoreNew, StoreSet, StoreSetString},
};
use substreams_ethereum::{pb::eth, Function};
use tycho_substreams::{
    balances::aggregate_balances_changes, contract::extract_contract_changes_builder, prelude::*,
};

const WSTETH_ADDRESS: [u8; 20] = hex!("7f39C581F595B53c5cb19bD0b3f8dA6c935E2Ca0"); //wstETH
const LOCKED_ASSET_ADDRESS: [u8; 20] = hex!("e19fc582dd93FA876CF4061Eb5456F310144F57b"); //stETH

#[substreams::handlers::map]
pub fn map_components(block: eth::v2::Block) -> Result<BlockTransactionProtocolComponents> {
    // We store these as a hashmap by tx hash since we need to agg by tx hash later
    Ok(BlockTransactionProtocolComponents {
        tx_components: block
            .transactions()
            .filter_map(|tx| {
                let components = tx
                    .calls()
                    .filter(|call| !call.call.state_reverted)
                    .filter_map(|_| {
                        if is_deployment_tx(tx, &WSTETH_ADDRESS) {
                            Some(create_vault_component(&tx.into()))
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
pub fn store_components(map: BlockTransactionProtocolComponents, store: StoreSetString) {
    map.tx_components
        .into_iter()
        .for_each(|tx_pc| {
            tx_pc
                .components
                .into_iter()
                .for_each(|pc| store.set(0, format!("pool:{0}", &pc.id[..42]), &pc.id))
        });
}

#[substreams::handlers::map]
pub fn map_relative_balances(block: eth::v2::Block) -> Result<BlockBalanceDeltas, anyhow::Error> {
    // # Initial state can be obtained from the initialize() call or first TokenRebased event
    // # Then track the following events/calls:

    // 1. "Submitted" events
    // total_pooled_eth += event.amount

    // 2. "ETHDistributed" events
    // # Updates from consensus layer and withdrawals
    // total_pooled_eth += (event.postCLBalance - event.preCLBalance)  # CL balance change
    // total_pooled_eth += event.executionLayerRewardsWithdrawn        # EL rewards
    // total_pooled_eth -= event.withdrawalsWithdrawn                  # Processed withdrawals

    // 3. "Unbuffered" events
    // # No change in total_pooled_eth as this just moves ETH from buffer to deposits

    // 4. Track deposit() function calls
    // # No change in total_pooled_eth as this just moves ETH from buffer to deposits

    // 5. "TokenRebased" events (as a verification)
    // # This event provides the absolute total_pooled_eth value
    // verify total_pooled_eth == event.postTotalEther
    let balance_deltas = block
        .transactions()
        .flat_map(|tx| {
            let mut deltas = Vec::new();
            if tx.to == WSTETH_ADDRESS {
                tx.calls.iter().for_each(|call| {
                    // Wrap function
                    if let (Some(unwrap_call), Ok(output_amount)) = (
                        abi::wsteth_contract::functions::Unwrap::match_and_decode(call),
                        abi::wsteth_contract::functions::Unwrap::output(&call.return_data),
                    ) {
                        let amount_in = unwrap_call
                            .u_wst_eth_amount
                            .to_signed_bytes_be();
                        let amount_out = output_amount.neg().to_signed_bytes_be();
                        deltas.extend_from_slice(&[
                            BalanceDelta {
                                ord: call.begin_ordinal,
                                tx: Some(tx.into()),
                                token: LOCKED_ASSET_ADDRESS.to_vec(),
                                delta: amount_out,
                                component_id: WSTETH_ADDRESS.to_vec(),
                            },
                            BalanceDelta {
                                ord: call.begin_ordinal,
                                tx: Some(tx.into()),
                                token: WSTETH_ADDRESS.to_vec(),
                                delta: amount_in,
                                component_id: WSTETH_ADDRESS.to_vec(),
                            },
                        ])
                    }
                    if let (Some(unwrap_call), Ok(output_amount)) = (
                        abi::wsteth_contract::functions::Unwrap::match_and_decode(call),
                        abi::wsteth_contract::functions::Unwrap::output(&call.return_data),
                    ) {
                        let amount_in = unwrap_call
                            .u_wst_eth_amount
                            .to_signed_bytes_be();
                        let amount_out = output_amount.neg().to_signed_bytes_be();
                        deltas.extend_from_slice(&[
                            BalanceDelta {
                                ord: call.begin_ordinal,
                                tx: Some(tx.into()),
                                token: LOCKED_ASSET_ADDRESS.to_vec(),
                                delta: amount_out,
                                component_id: WSTETH_ADDRESS.to_vec(),
                            },
                            BalanceDelta {
                                ord: call.begin_ordinal,
                                tx: Some(tx.into()),
                                token: WSTETH_ADDRESS.to_vec(),
                                delta: amount_in,
                                component_id: WSTETH_ADDRESS.to_vec(),
                            },
                        ])
                    }
                })
            }
            deltas
        })
        .collect_vec();
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
///  cooresponding changes (balance, component, contract), inserting a new one if it doesn't exist.
///  At the very end, the map can easily be sorted by index to ensure the final
/// `BlockContractChanges`  is ordered by transactions properly.
#[substreams::handlers::map]
pub fn map_protocol_changes(
    block: eth::v2::Block,
    grouped_components: BlockTransactionProtocolComponents,
    deltas: BlockBalanceDeltas,
    components_store: StoreGetString,
    balance_store: StoreDeltas, // Note, this map module is using the `deltas` mode for the store.
) -> Result<BlockChanges> {
    // We merge contract changes by transaction (identified by transaction index) making it easy to
    //  sort them at the very end.
    let mut transaction_changes: HashMap<_, TransactionChangesBuilder> = HashMap::new();

    // `ProtocolComponents` are gathered from `map_pools_created` which just need a bit of work to
    //   convert into `TransactionChanges`
    let default_attributes = vec![Attribute {
        name: "update_marker".to_string(),
        value: vec![1u8],
        change: ChangeType::Creation.into(),
    }];
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
                    let entity_change = EntityChanges {
                        component_id: component.id.clone(),
                        attributes: default_attributes.clone(),
                    };
                    builder.add_entity_change(&entity_change)
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

    transaction_changes
        .iter_mut()
        .for_each(|(_, change)| {
            // this indirection is necessary due to borrowing rules.
            let addresses = change
                .changed_contracts()
                .map(|e| e.to_vec())
                .collect::<Vec<_>>();
            addresses
                .into_iter()
                .for_each(|address| {
                    // We reconstruct the component_id from the address here
                    let id = components_store
                        .get_last(format!("pool:0x{}", hex::encode(address)))
                        .unwrap(); // Shouldn't happen because we filter by known components in
                                   // `extract_contract_changes_builder`
                    change.mark_component_as_updated(&id);
                })
        });

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

fn create_vault_component(tx: &Transaction) -> ProtocolComponent {
    ProtocolComponent::at_contract(WSTETH_ADDRESS.as_slice(), tx)
        .with_tokens(&[LOCKED_ASSET_ADDRESS])
        .as_swap_type("lido_wsteth", ImplementationType::Vm)
}
