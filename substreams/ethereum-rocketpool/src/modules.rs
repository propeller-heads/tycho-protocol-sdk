use crate::abi;
use anyhow::Result;
use itertools::Itertools;
use std::collections::HashMap;
use substreams::{
    hex,
    pb::substreams::StoreDeltas,
    store::{
        StoreAdd, StoreAddBigInt, StoreAddInt64, StoreGet, StoreGetInt64, StoreGetString, StoreNew,
    },
};
use substreams_ethereum::{pb::eth, Event};
use tycho_substreams::{
    balances::aggregate_balances_changes, contract::extract_contract_changes_builder, prelude::*,
};

const RETH_ADDRESS: [u8; 20] = hex!("D33526068D116cE69F19A9ee46F0bd304F21A51f"); //RPL rocketPool Token
const VAULT_ADDRESS: [u8; 20] = hex!("3bDC69C4E5e13E52A65f5583c23EFB9636b469d6"); //vault
const ETHER_TOKEN_ADDRESS: [u8; 20] = hex!("0000000000000000000000000000000000000000"); //ETH

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
                        if is_deployment_tx(tx, &VAULT_ADDRESS) {
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

/// Simply stores the `ProtocolComponent`s with the pool id as the key
#[substreams::handlers::store]
pub fn store_components(map: BlockTransactionProtocolComponents, store: StoreAddInt64) {
    store.add_many(
        0,
        &map.tx_components
            .iter()
            .flat_map(|tx_components| &tx_components.components)
            .map(|component| format!("pool:{0}", component.id))
            .collect::<Vec<_>>(),
        1,
    );
}

#[substreams::handlers::map]
pub fn map_relative_balances(
    block: eth::v2::Block,
    store: StoreGetInt64,
) -> Result<BlockBalanceDeltas, anyhow::Error> {
    let balance_deltas = block
        .logs()
        .filter(|log| log.address() == VAULT_ADDRESS)
        .flat_map(|vault_log| {
            let mut deltas = Vec::new();
            let address_bytes_be = vault_log.address();
            let address_hex = format!("0x{}", hex::encode(address_bytes_be));

            if store
                .get_last(format!("pool:{}", address_hex))
                .is_some()
            {
                if let Some(ev) =
                    abi::rocketvault_contract::events::EtherDeposited::match_and_decode(
                        vault_log.log,
                    )
                {
                    deltas.push(BalanceDelta {
                        ord: vault_log.ordinal(),
                        tx: Some(vault_log.receipt.transaction.into()),
                        token: ETHER_TOKEN_ADDRESS.to_vec(), // ETH address
                        delta: ev.amount.to_signed_bytes_be(),
                        component_id: address_hex.as_bytes().to_vec(),
                    });
                } else if let Some(ev) =
                    abi::rocketvault_contract::events::EtherWithdrawn::match_and_decode(
                        vault_log.log,
                    )
                {
                    deltas.push(BalanceDelta {
                        ord: vault_log.ordinal(),
                        tx: Some(vault_log.receipt.transaction.into()),
                        token: ETHER_TOKEN_ADDRESS.to_vec(), // ETH address
                        delta: ev.amount.neg().to_signed_bytes_be(),
                        component_id: address_hex.as_bytes().to_vec(),
                    });
                } else if let Some(ev) =
                    abi::rocketvault_contract::events::TokenDeposited::match_and_decode(
                        vault_log.log,
                    )
                {
                    deltas.push(BalanceDelta {
                        ord: vault_log.ordinal(),
                        tx: Some(vault_log.receipt.transaction.into()),
                        token: RETH_ADDRESS.to_vec(),
                        delta: ev.amount.to_signed_bytes_be(),
                        component_id: address_hex.as_bytes().to_vec(),
                    });
                } else if let Some(ev) =
                    abi::rocketvault_contract::events::TokenWithdrawn::match_and_decode(
                        vault_log.log,
                    )
                {
                    deltas.push(BalanceDelta {
                        ord: vault_log.ordinal(),
                        tx: Some(vault_log.receipt.transaction.into()),
                        token: RETH_ADDRESS.to_vec(),
                        delta: ev.amount.neg().to_signed_bytes_be(),
                        component_id: address_hex.as_bytes().to_vec(),
                    });
                } else if let Some(ev) =
                    abi::rocketvault_contract::events::TokenBurned::match_and_decode(vault_log.log)
                {
                    deltas.push(BalanceDelta {
                        ord: vault_log.ordinal(),
                        tx: Some(vault_log.receipt.transaction.into()),
                        token: RETH_ADDRESS.to_vec(),
                        delta: ev.amount.neg().to_signed_bytes_be(),
                        component_id: address_hex.as_bytes().to_vec(),
                    });
                }
            }
            deltas
        })
        .collect::<Vec<_>>();

    Ok(BlockBalanceDeltas { balance_deltas })
}

/// This is the main map that handles most of the indexing of this substream.
/// Every contract change is grouped by transaction index via the `transaction_changes`
///  map. Each block of code will extend the `TransactionChanges` struct with the
///  cooresponding changes (balance, component, contract), inserting a new one if it doesn't exist.
///  At the very end, the map can easily be sorted by index to ensure the final
/// `BlockChanges`  is ordered by transactions properly.
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
    let default_attributes = vec![
        Attribute {
            name: "balance_owner".to_string(),
            value: VAULT_ADDRESS.to_vec(),
            change: ChangeType::Creation.into(),
        },
        Attribute {
            name: "update_marker".to_string(),
            value: vec![1u8],
            change: ChangeType::Creation.into(),
        },
    ];
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
                .for_each(|bc| builder.add_balance_change(bc));
        });

    // Extract and insert any storage changes that happened for any of the components.
    extract_contract_changes_builder(
        &block,
        |addr| {
            components_store
                .get_last(format!("pool:0x{0}", hex::encode(addr)))
                .is_some()
                || addr.eq(&VAULT_ADDRESS)
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
                    if address != VAULT_ADDRESS {
                        // We reconstruct the component_id from the address here
                        let id = components_store
                            .get_last(format!("pool:0x{}", hex::encode(address)))
                            .unwrap(); // Shouldn't happen because we filter by known components in
                                       // `extract_contract_changes_builder`
                        change.mark_component_as_updated(&id);
                    }
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

/// It's significant to include both the `pool_id` and the `token_id` for each balance delta as the
///  store key to ensure that there's a unique balance being tallied for each.
#[substreams::handlers::store]
pub fn store_balances(deltas: BlockBalanceDeltas, store: StoreAddBigInt) {
    tycho_substreams::balances::store_balance_changes(deltas, store);
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
    ProtocolComponent::at_contract(VAULT_ADDRESS.as_slice(), tx)
        .with_tokens(&[RETH_ADDRESS, ETHER_TOKEN_ADDRESS])
        .as_swap_type("rocketvault", ImplementationType::Vm)
}
