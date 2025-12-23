use crate::{
    abi::{
        eeth::functions::MintShares,
        weeth::functions::{Unwrap, Wrap, WrapWithPermit},
    },
    consts::{
        EETH_ADDRESS, ETH_ADDRESS, LIQUIDITY_POOL_ADDRESS, REDEMPTION_MANAGER_ADDRESS,
        REDEMPTION_MANAGER_CREATION_BLOCK, REDEMPTION_MANAGER_CREATION_TX, WEETH_ADDRESS,
        WEETH_CREATION_BLOCK, WEETH_CREATION_TX,
    },
    storage::{get_changed_attributes, EETH_POOL_TRACKED_SLOTS, WEETH_POOL_TRACKED_SLOTS},
};
use anyhow::{Ok, Result};
use itertools::Itertools;
use std::collections::HashMap;
use substreams::{pb::substreams::StoreDeltas, prelude::*};
use substreams_ethereum::{
    pb::eth::{
        self,
        v2::{StorageChange, TransactionTrace},
    },
    Function,
};
use substreams_helper::hex::Hexable;
use tycho_substreams::{
    balances::aggregate_balances_changes, block_storage::get_block_storage_changes, prelude::*,
};

/// Find and create all relevant protocol components
///
/// This method maps over blocks and instantiates ProtocolComponents with a unique ids
/// as well as all necessary metadata for routing and encoding.
#[substreams::handlers::map]
fn map_protocol_components(block: eth::v2::Block) -> Result<BlockTransactionProtocolComponents> {
    let mut tx_components: Vec<TransactionProtocolComponents> = Vec::new();
    if block.number == WEETH_CREATION_BLOCK {
        if let Some(tx) = block
            .transactions()
            .find(|tx| hex::encode(&tx.hash) == WEETH_CREATION_TX)
        {
            tx_components.push(TransactionProtocolComponents {
                tx: Some(tx.into()),
                components: vec![ProtocolComponent {
                    id: format!("0x{}", hex::encode(WEETH_ADDRESS)),
                    tokens: vec![WEETH_ADDRESS.into(), EETH_ADDRESS.into()],
                    contracts: vec![],
                    static_att: vec![],
                    change: ChangeType::Creation.into(),
                    protocol_type: Some(ProtocolType {
                        name: "ethereum-etherfi".into(),
                        financial_type: FinancialType::Swap.into(),
                        attribute_schema: Vec::new(),
                        implementation_type: ImplementationType::Custom.into(),
                    }),
                }],
            });
        }
    }
    if block.number == REDEMPTION_MANAGER_CREATION_BLOCK {
        if let Some(tx) = block
            .transactions()
            .find(|tx| hex::encode(&tx.hash) == REDEMPTION_MANAGER_CREATION_TX)
        {
            tx_components.push(TransactionProtocolComponents {
                tx: Some(tx.into()),
                components: vec![ProtocolComponent {
                    id: format!("0x{}", hex::encode(EETH_ADDRESS)),
                    tokens: vec![EETH_ADDRESS.into(), ETH_ADDRESS.into()],
                    contracts: vec![],
                    static_att: vec![],
                    change: ChangeType::Creation.into(),
                    protocol_type: Some(ProtocolType {
                        name: "ethereum-etherfi".into(),
                        financial_type: FinancialType::Swap.into(),
                        attribute_schema: Vec::new(),
                        implementation_type: ImplementationType::Custom.into(),
                    }),
                }],
            });
        }
    }
    Ok(BlockTransactionProtocolComponents { tx_components })
}

#[substreams::handlers::store]
fn store_component_tokens(
    map_protocol_components: BlockTransactionProtocolComponents,
    store: StoreSetString,
) {
    map_protocol_components
        .tx_components
        .into_iter()
        .for_each(|tx_pc| {
            tx_pc
                .components
                .into_iter()
                .for_each(|pc| {
                    store.set(
                        0,
                        format!("pool:{0}", pc.id),
                        &pc.tokens
                            .iter()
                            .map(hex::encode)
                            .join(":"),
                    );
                })
        });
}

#[substreams::handlers::map]
fn map_relative_balances(
    block: eth::v2::Block,
    _store: StoreGetInt64,
) -> Result<BlockBalanceDeltas> {
    let mut deltas: Vec<BalanceDelta> = block
        .transactions()
        .flat_map(|tx| {
            let mut tx_balance_deltas = emit_balance_deltas_for_weeth(tx);
            tx_balance_deltas.extend(emit_balance_deltas_for_eeth(tx));
            tx_balance_deltas
        })
        .collect();
    // Keep it consistent with how it's inserted in the store. This step is important
    // because we use a zip on the store deltas and balance deltas later.
    deltas.sort_unstable_by(|a, b| a.ord.cmp(&b.ord));
    Ok(BlockBalanceDeltas { balance_deltas: deltas })
}

fn emit_balance_deltas_for_weeth(tx_trace: &TransactionTrace) -> Vec<BalanceDelta> {
    // tokens: eETH and weETH
    // function calls: weETH.wrap and weETH.unwrap
    let mut deltas = vec![];
    for call in tx_trace
        .calls()
        .filter(|call| !call.call.state_reverted)
    {
        if let Some(wrap) = Wrap::match_and_decode(call) {
            deltas.push(BalanceDelta {
                ord: call.call.end_ordinal,
                tx: Some(tx_trace.into()),
                token: EETH_ADDRESS.to_vec(),
                delta: wrap.e_eth_amount.to_signed_bytes_be(),
                component_id: WEETH_ADDRESS
                    .to_vec()
                    .to_hex()
                    .as_bytes()
                    .to_vec(),
            });
        }
        if let Some(wrap) = WrapWithPermit::match_and_decode(call) {
            deltas.push(BalanceDelta {
                ord: call.call.end_ordinal,
                tx: Some(tx_trace.into()),
                token: EETH_ADDRESS.to_vec(),
                delta: wrap.e_eth_amount.to_signed_bytes_be(),
                component_id: WEETH_ADDRESS
                    .to_vec()
                    .to_hex()
                    .as_bytes()
                    .to_vec(),
            });
        }
        if Unwrap::match_and_decode(call).is_some() {
            let eeth_amount = Unwrap::output_call(call.call).expect("Failed to decode unwrap call");
            deltas.push(BalanceDelta {
                ord: call.call.end_ordinal,
                tx: Some(tx_trace.into()),
                token: EETH_ADDRESS.to_vec(),
                delta: eeth_amount.neg().to_signed_bytes_be(),
                component_id: WEETH_ADDRESS
                    .to_vec()
                    .to_hex()
                    .as_bytes()
                    .to_vec(),
            });
        }
    }
    deltas
}

fn emit_balance_deltas_for_eeth(tx_trace: &TransactionTrace) -> Vec<BalanceDelta> {
    let mut deltas = vec![];
    for call in tx_trace
        .calls()
        .filter(|call| !call.call.state_reverted)
    {
        for balance_change in &call.call.balance_changes {
            if balance_change.address == LIQUIDITY_POOL_ADDRESS.to_vec() {
                let delta = BigInt::from_unsigned_bytes_be(
                    &balance_change
                        .new_value
                        .clone()
                        .unwrap_or_default()
                        .bytes,
                ) - BigInt::from_unsigned_bytes_be(
                    &balance_change
                        .old_value
                        .clone()
                        .unwrap_or_default()
                        .bytes,
                );
                deltas.push(BalanceDelta {
                    ord: call.call.end_ordinal,
                    tx: Some(tx_trace.into()),
                    token: ETH_ADDRESS.to_vec(),
                    delta: delta.to_signed_bytes_be(),
                    component_id: balance_change
                        .address
                        .to_hex()
                        .as_bytes()
                        .to_vec(),
                });
            }
        }
        if let Some(mint_shares) = MintShares::match_and_decode(call) {
            deltas.push(BalanceDelta {
                ord: call.call.end_ordinal,
                tx: Some(tx_trace.into()),
                token: EETH_ADDRESS.to_vec(),
                delta: mint_shares.share.to_signed_bytes_be(),
                component_id: LIQUIDITY_POOL_ADDRESS
                    .to_vec()
                    .to_hex()
                    .as_bytes()
                    .to_vec(),
            });
        }
    }
    deltas
}

#[substreams::handlers::store]
pub fn store_component_balances(deltas: BlockBalanceDeltas, store: StoreAddBigInt) {
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
    balance_store: StoreDeltas,
    deltas: BlockBalanceDeltas,
) -> Result<BlockChanges, substreams::errors::Error> {
    // We merge contract changes by transaction (identified by transaction index)
    // making it easy to sort them at the very end.
    let mut transaction_changes: HashMap<u64, TransactionChangesBuilder> = HashMap::new();

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

    let block_storage_changes = get_block_storage_changes(&block);

    weeth_entity_changes(&block_storage_changes, &mut transaction_changes)?;
    eeth_entity_changes(&block_storage_changes, &mut transaction_changes)?;

    // Process all `transaction_changes` for final output in the `BlockChanges`,
    //  sorted by transaction index (the key).
    Ok(BlockChanges {
        block: Some((&block).into()),
        changes: transaction_changes
            .drain()
            .sorted_unstable_by_key(|(index, _)| *index)
            .filter_map(|(_, builder)| builder.build())
            .collect::<Vec<_>>(),
        storage_changes: block_storage_changes,
    })
}

fn weeth_entity_changes(
    storage_changes: &Vec<TransactionStorageChanges>,
    transaction_changes: &mut HashMap<u64, TransactionChangesBuilder>,
) -> Result<()> {
    for change in storage_changes {
        let tx = match &change.tx {
            Some(tx) => tx,
            None => continue,
        };
        let builder = transaction_changes
            .entry(tx.index)
            .or_insert_with(|| TransactionChangesBuilder::new(tx));

        let filtered_storage_changes: Vec<StorageChange> = change
            .storage_changes
            .iter()
            .filter(|storage_change| {
                storage_change.address == LIQUIDITY_POOL_ADDRESS.to_vec() ||
                    storage_change.address == EETH_ADDRESS.to_vec()
            })
            .flat_map(|storage_change| {
                storage_change
                    .slots
                    .iter()
                    .map(|slot| StorageChange {
                        address: storage_change.address.clone(),
                        key: slot.slot.clone(),
                        old_value: slot.previous_value.clone(),
                        new_value: slot.value.clone(),
                        ordinal: 0,
                    })
            })
            .collect();

        let changed_attributes = get_changed_attributes(
            &filtered_storage_changes,
            WEETH_POOL_TRACKED_SLOTS
                .to_vec()
                .iter()
                .collect(),
        );

        builder.add_entity_change(&EntityChanges {
            component_id: format!("0x{}", hex::encode(WEETH_ADDRESS)),
            attributes: changed_attributes,
        });
    }
    Ok(())
}

fn eeth_entity_changes(
    storage_changes: &Vec<TransactionStorageChanges>,
    transaction_changes: &mut HashMap<u64, TransactionChangesBuilder>,
) -> Result<()> {
    for change in storage_changes {
        let tx = match &change.tx {
            Some(tx) => tx,
            None => continue,
        };
        let builder = transaction_changes
            .entry(tx.index)
            .or_insert_with(|| TransactionChangesBuilder::new(tx));

        let filtered_storage_changes: Vec<StorageChange> = change
            .storage_changes
            .iter()
            .filter(|storage_change| {
                storage_change.address == LIQUIDITY_POOL_ADDRESS.to_vec() ||
                    storage_change.address == EETH_ADDRESS.to_vec() ||
                    storage_change.address == REDEMPTION_MANAGER_ADDRESS.to_vec()
            })
            .flat_map(|storage_change| {
                storage_change
                    .slots
                    .iter()
                    .map(|slot| StorageChange {
                        address: storage_change.address.clone(),
                        key: slot.slot.clone(),
                        old_value: slot.previous_value.clone(),
                        new_value: slot.value.clone(),
                        ordinal: 0,
                    })
            })
            .collect();

        let changed_attributes = get_changed_attributes(
            &filtered_storage_changes,
            EETH_POOL_TRACKED_SLOTS
                .to_vec()
                .iter()
                .collect(),
        );

        builder.add_entity_change(&EntityChanges {
            component_id: format!("0x{}", hex::encode(WEETH_ADDRESS)),
            attributes: changed_attributes,
        });
    }
    Ok(())
}
