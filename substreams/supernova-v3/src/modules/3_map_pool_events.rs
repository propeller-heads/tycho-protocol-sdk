use itertools::Itertools;
use std::collections::{HashMap, HashSet};
use substreams::pb::substreams::StoreDeltas;
use substreams::store::{StoreGet, StoreGetProto};
use substreams_ethereum::pb::eth::v2::{self as eth};
use substreams_ethereum::Event;

use crate::abi;
use crate::store_key::StoreKey;
use tycho_substreams::prelude::*;

// Auxiliary struct to serve as a key for the HashMaps.
#[derive(Clone, Hash, Eq, PartialEq)]
struct ComponentKey<T> {
    component_id: String,
    name: T,
}

impl<T> ComponentKey<T> {
    fn new(component_id: String, name: T) -> Self {
        ComponentKey { component_id, name }
    }
}

#[derive(Clone)]
struct PartialChanges {
    transaction: Transaction,
    entity_changes: HashMap<ComponentKey<String>, Attribute>,
    balance_changes: HashMap<ComponentKey<Vec<u8>>, BalanceChange>,
}

impl PartialChanges {
    fn consolidate_entity_changes(self) -> Vec<EntityChanges> {
        self.entity_changes
            .into_iter()
            .map(|(key, attribute)| (key.component_id, attribute))
            .into_group_map()
            .into_iter()
            .map(|(component_id, attributes)| EntityChanges { component_id, attributes })
            .collect()
    }
}

#[substreams::handlers::map]
pub fn map_pool_events(
    block: eth::Block,
    block_entity_changes: BlockChanges,
    pools_store: StoreGetProto<ProtocolComponent>,
    balances_deltas: BlockBalanceDeltas,
    balance_store: StoreDeltas,
) -> Result<BlockChanges, substreams::errors::Error> {
    let mut block_entity_changes = block_entity_changes;
    let mut tx_changes: HashMap<Vec<u8>, PartialChanges> = HashMap::new();

    let created_pools: HashSet<String> = block_entity_changes
        .changes
        .iter()
        .flat_map(|c| {
            c.component_changes
                .iter()
                .map(|pc| pc.id.to_lowercase())
        })
        .collect();

    handle_events(&block, &mut tx_changes, &pools_store, &created_pools);
    merge_block(&mut tx_changes, &mut block_entity_changes);

    tycho_substreams::balances::aggregate_balances_changes(balance_store, balances_deltas)
        .into_iter()
        .for_each(|(_, (tx, balances))| {
            if let Some(change) = block_entity_changes
                .changes
                .iter_mut()
                .find(|c| c.tx.as_ref().unwrap().hash == tx.hash)
            {
                balances
                    .values()
                    .for_each(|token_bc_map| {
                        token_bc_map
                            .values()
                            .for_each(|bc| change.balance_changes.push(bc.clone()))
                    });
            } else {
                let mut new_change =
                    TransactionChanges { tx: Some(tx.clone()), ..Default::default() };
                balances
                    .values()
                    .for_each(|token_bc_map| {
                        token_bc_map.values().for_each(|bc| {
                            new_change
                                .balance_changes
                                .push(bc.clone())
                        })
                    });
                block_entity_changes
                    .changes
                    .push(new_change);
            }
        });

    // Final Stage: Universal Block-Level Deduplicator (Atomic Consolidation)
    // Consolidate all state updates (Attributes & Balances) to ensure ONLY ONE entry per component per block.
    // We attach all final changes for a pool to the LATEST transaction that touched it.
    let mut pool_to_latest_tx: HashMap<String, usize> = HashMap::new();
    let mut final_balances: HashMap<(String, Vec<u8>), BalanceChange> = HashMap::new();
    let mut final_attributes: HashMap<(String, String), Attribute> = HashMap::new();
    let mut pool_created: HashMap<String, ProtocolComponent> = HashMap::new();

    for (tx_idx, change) in block_entity_changes
        .changes
        .iter_mut()
        .enumerate()
    {
        // 1. Scan for Creations (DO NOT DRAIN static_att here, keep the creation record)
        for pc in change.component_changes.drain(..) {
            let id = pc.id.to_lowercase();
            pool_created.insert(id.clone(), pc);

            let current_latest = pool_to_latest_tx.entry(id).or_insert(0);
            if tx_idx > *current_latest {
                *current_latest = tx_idx;
            }
        }

        // 2. Aggregate Balances
        for bc in change.balance_changes.drain(..) {
            let pool_id = String::from_utf8_lossy(&bc.component_id).to_string().to_lowercase();
            let token_id = bc.token.clone();
            let key = (pool_id.clone(), token_id);

            // Update latest tx index for this pool
            let current_latest = pool_to_latest_tx.entry(pool_id.clone()).or_insert(0);
            if tx_idx > *current_latest { *current_latest = tx_idx; }

            // Aggregate delta using big-integer math
            if let Some(existing) = final_balances.get_mut(&key) {
                let mut sum = substreams::scalar::BigInt::from_signed_bytes_be(&existing.balance);
                let delta = substreams::scalar::BigInt::from_signed_bytes_be(&bc.balance);
                sum = sum + delta;
                existing.balance = sum.to_signed_bytes_be();
            } else {
                final_balances.insert(key, bc);
            }
        }

        // 3. Scan and Drain Attributes from entity_changes
        for ec in change.entity_changes.drain(..) {
            let pool_id = ec.component_id.to_lowercase();
            for attr in ec.attributes {
                final_attributes.insert((pool_id.clone(), attr.name.clone()), attr);
            }
            let current_latest = pool_to_latest_tx
                .entry(pool_id)
                .or_insert(0);
            if tx_idx > *current_latest {
                *current_latest = tx_idx;
            }
        }
    }

    // 4. Re-insert Consolidated Creations
    for (pool_id, pc) in pool_created {
        let tx_idx = pool_to_latest_tx[&pool_id];
        block_entity_changes.changes[tx_idx]
            .component_changes
            .push(pc);
    }

    // 5. Re-insert Consolidated Balances
    for ((pool_id, _token_id), mut bc) in final_balances {
        let tx_idx = pool_to_latest_tx[&pool_id];
        bc.component_id = pool_id.clone().into_bytes();
        block_entity_changes.changes[tx_idx]
            .balance_changes
            .push(bc);
    }

    // 6. Re-insert Consolidated Attributes
    for ((pool_id, _attr_name), mut attr) in final_attributes {
        let tx_idx = pool_to_latest_tx[&pool_id];
        // If a pool was created in this block, any additional state updates are handled by Tycho correctly.
        // We mark them as Updates to avoid conflict with the creation's static_att.
        attr.change = ChangeType::Update.into();

        let change = &mut block_entity_changes.changes[tx_idx];
        if let Some(ec) = change
            .entity_changes
            .iter_mut()
            .find(|ec| ec.component_id.to_lowercase() == pool_id)
        {
            ec.attributes.push(attr);
        } else {
            change
                .entity_changes
                .push(EntityChanges { component_id: pool_id.clone(), attributes: vec![attr] });
        }
    }

    block_entity_changes
        .changes
        .sort_by_key(|a| a.tx.as_ref().unwrap().index);

    Ok(block_entity_changes)
}

fn handle_events(
    block: &eth::Block,
    tx_changes: &mut HashMap<Vec<u8>, PartialChanges>,
    store: &StoreGetProto<ProtocolComponent>,
    created_pools: &HashSet<String>,
) {
    for trx in block.transactions() {
        if trx.status != 1 {
            continue;
        }
        let hash = trx.hash.clone();
        if let Some(receipt) = trx.receipt.as_ref() {
            for log in &receipt.logs {
                let pool_address_hex = format!("{}", substreams::Hex(&log.address)).to_lowercase();

                // Is this log's address a known pool or just created?
                if store
                    .get_last(StoreKey::Pool.get_unique_pool_key(&pool_address_hex))
                    .is_none()
                    && !created_pools.contains(&pool_address_hex)
                {
                    continue;
                }

                if let Some(event) = abi::algebrapool::events::Swap::match_and_decode(log) {
                    substreams::log::info!(
                        "Event (Swap): pool={} liquidity={}",
                        pool_address_hex,
                        event.liquidity
                    );
                    let tx_change = tx_changes
                        .entry(hash.clone())
                        .or_insert_with(|| PartialChanges {
                            transaction: trx.into(),
                            entity_changes: HashMap::new(),
                            balance_changes: HashMap::new(),
                        });

                    tx_change.entity_changes.insert(
                        ComponentKey::new(pool_address_hex.clone(), "liquidity".to_string()),
                        Attribute {
                            name: "liquidity".to_string(),
                            value: substreams::scalar::BigInt::from(event.liquidity)
                                .to_signed_bytes_be(),
                            change: ChangeType::Update.into(),
                        },
                    );
                    tx_change.entity_changes.insert(
                        ComponentKey::new(pool_address_hex.clone(), "tick".to_string()),
                        Attribute {
                            name: "tick".to_string(),
                            value: substreams::scalar::BigInt::from(event.tick)
                                .to_signed_bytes_be(),
                            change: ChangeType::Update.into(),
                        },
                    );
                } else if let Some(event) = abi::algebrapool::events::Burn::match_and_decode(log) {
                    substreams::log::info!(
                        "Event (Burn): pool={} liquidity={}",
                        pool_address_hex,
                        event.liquidity_amount
                    );
                    let tx_change = tx_changes
                        .entry(hash.clone())
                        .or_insert_with(|| PartialChanges {
                            transaction: trx.into(),
                            entity_changes: HashMap::new(),
                            balance_changes: HashMap::new(),
                        });

                    tx_change.entity_changes.insert(
                        ComponentKey::new(pool_address_hex.clone(), "liquidity".to_string()),
                        Attribute {
                            name: "liquidity".to_string(),
                            value: substreams::scalar::BigInt::from(event.liquidity_amount.clone())
                                .to_signed_bytes_be(),
                            change: ChangeType::Update.into(),
                        },
                    );
                } else if let Some(event) = abi::algebrapool::events::Mint::match_and_decode(log) {
                    substreams::log::info!(
                        "Event (Mint): pool={} liquidity={}",
                        pool_address_hex,
                        event.liquidity_amount
                    );
                    let tx_change = tx_changes
                        .entry(hash.clone())
                        .or_insert_with(|| PartialChanges {
                            transaction: trx.into(),
                            entity_changes: HashMap::new(),
                            balance_changes: HashMap::new(),
                        });

                    tx_change.entity_changes.insert(
                        ComponentKey::new(pool_address_hex.clone(), "liquidity".to_string()),
                        Attribute {
                            name: "liquidity".to_string(),
                            value: substreams::scalar::BigInt::from(event.liquidity_amount.clone())
                                .to_signed_bytes_be(),
                            change: ChangeType::Update.into(),
                        },
                    );

                    // Add tick liquidity changes
                    tx_change.entity_changes.insert(
                        ComponentKey::new(
                            pool_address_hex.clone(),
                            format!("tick_{}_liquidity_net", event.bottom_tick),
                        ),
                        Attribute {
                            name: format!("tick_{}_liquidity_net", event.bottom_tick),
                            value: substreams::scalar::BigInt::from(event.liquidity_amount.clone())
                                .to_signed_bytes_be(),
                            change: ChangeType::Update.into(),
                        },
                    );
                    tx_change.entity_changes.insert(
                        ComponentKey::new(
                            pool_address_hex.clone(),
                            format!("tick_{}_liquidity_net", event.top_tick),
                        ),
                        Attribute {
                            name: format!("tick_{}_liquidity_net", event.top_tick),
                            value: substreams::scalar::BigInt::from(event.liquidity_amount.clone())
                                .to_signed_bytes_be(),
                            change: ChangeType::Update.into(),
                        },
                    );
                } else if let Some(event) = abi::algebrapool::events::Fee::match_and_decode(log) {
                    substreams::log::info!(
                        "Event (Fee): pool={} new_fee={}",
                        pool_address_hex,
                        event.fee
                    );
                    let tx_change = tx_changes
                        .entry(hash.clone())
                        .or_insert_with(|| PartialChanges {
                            transaction: trx.into(),
                            entity_changes: HashMap::new(),
                            balance_changes: HashMap::new(),
                        });

                    tx_change.entity_changes.insert(
                        ComponentKey::new(pool_address_hex.clone(), "fee".to_string()),
                        Attribute {
                            name: "fee".to_string(),
                            value: substreams::scalar::BigInt::from(event.fee).to_signed_bytes_be(),
                            change: ChangeType::Update.into(),
                        },
                    );
                } else if let Some(event) =
                    abi::algebrapool::events::TickSpacing::match_and_decode(log)
                {
                    substreams::log::info!(
                        "Event (TickSpacing): pool={} new_spacing={}",
                        pool_address_hex,
                        event.new_tick_spacing
                    );
                    let tx_change = tx_changes
                        .entry(hash.clone())
                        .or_insert_with(|| PartialChanges {
                            transaction: trx.into(),
                            entity_changes: HashMap::new(),
                            balance_changes: HashMap::new(),
                        });

                    tx_change.entity_changes.insert(
                        ComponentKey::new(pool_address_hex.clone(), "tick_spacing".to_string()),
                        Attribute {
                            name: "tick_spacing".to_string(),
                            value: substreams::scalar::BigInt::from(event.new_tick_spacing)
                                .to_signed_bytes_be(),
                            change: ChangeType::Update.into(),
                        },
                    );
                }
            }
        }
    }
}

fn merge_block(
    tx_changes: &mut HashMap<Vec<u8>, PartialChanges>,
    block_entity_changes: &mut BlockChanges,
) {
    let mut tx_to_changes: HashMap<Vec<u8>, TransactionChanges> = HashMap::new();

    // 1. Unify transactions from previous module
    for change in block_entity_changes.changes.drain(..) {
        let hash = change.tx.as_ref().unwrap().hash.clone();
        tx_to_changes
            .entry(hash)
            .or_insert(change);
    }

    // 2. Merge local events
    for (hash, partial) in tx_changes.drain() {
        let tx_change = tx_to_changes
            .entry(hash)
            .or_insert_with(|| TransactionChanges {
                tx: Some(partial.transaction.clone()),
                ..Default::default()
            });

        for partial_ec in partial
            .clone()
            .consolidate_entity_changes()
        {
            let pool_id = partial_ec.component_id.to_lowercase();
            if let Some(existing_ec) = tx_change
                .entity_changes
                .iter_mut()
                .find(|ec| ec.component_id.to_lowercase() == pool_id)
            {
                existing_ec
                    .attributes
                    .extend(partial_ec.attributes);
            } else {
                tx_change
                    .entity_changes
                    .push(partial_ec);
            }
        }

        // Balances
        tx_change
            .balance_changes
            .extend(partial.balance_changes.into_values());
    }

    block_entity_changes.changes = tx_to_changes.into_values().collect();
}
