use std::collections::{HashMap, HashSet};
use substreams::pb::substreams::StoreDeltas;
use substreams::store::{StoreGet, StoreGetProto};
use substreams_ethereum::pb::eth::v2::{self as eth};

use crate::events::get_log_changed_attributes;
use crate::store_key::StoreKey;
use tycho_substreams::prelude::*;
use tycho_substreams::contract::extract_contract_changes_builder;

#[substreams::handlers::map]
pub fn map_pool_events(
    block: eth::Block,
    block_entity_changes: BlockChanges,
    pools_store: StoreGetProto<ProtocolComponent>,
    balances_deltas: BlockBalanceDeltas,
    balance_store: StoreDeltas,
) -> Result<BlockChanges, substreams::errors::Error> {
    let mut builders: HashMap<u64, TransactionChangesBuilder> = HashMap::new();
    let mut created_pools: HashSet<String> = HashSet::new();

    // 1. Initialise builders from existing block_entity_changes (e.g. from pool creation module)
    for change in block_entity_changes.changes {
        let tx = change.tx.as_ref().expect("Transaction missing in BlockChanges");
        let builder = builders.entry(tx.index)
            .or_insert_with(|| TransactionChangesBuilder::new(tx));
        
        for pc in &change.component_changes {
            builder.add_protocol_component(pc);
            created_pools.insert(pc.id.to_lowercase());
            // Also register every contract this component owns (e.g. plugin),
            // so pool_check matches plugin storage changes in this same block.
            for contract_addr in &pc.contracts {
                created_pools.insert(format!("0x{}", hex::encode(contract_addr)).to_lowercase());
            }
        }
        for bc in &change.balance_changes {
            builder.add_balance_change(bc);
        }
        for ec in &change.entity_changes {
            builder.add_entity_change(ec);
        }
        for cc in &change.contract_changes {
            let mut interim = InterimContractChange::new(&cc.address, cc.change == i32::from(ChangeType::Creation));
            if !cc.code.is_empty() { interim.set_code(&cc.code); }
            if !cc.balance.is_empty() { interim.set_balance(&cc.balance); }
            for slot in &cc.slots {
                interim.upsert_slot(&eth::StorageChange {
                    address: cc.address.clone(),
                    key: slot.slot.clone(),
                    new_value: slot.value.clone(),
                    old_value: slot.previous_value.clone(),
                    ordinal: 0,
                });
            }
            builder.add_contract_changes(&interim);
        }
    }

    // 2. Identify all pools (Store + new)
    let pool_check = |address: &[u8]| {
        let addr_hex = format!("0x{}", hex::encode(address)).to_lowercase();
        created_pools.contains(&addr_hex) || 
        pools_store.get_last(StoreKey::Pool.get_unique_pool_key(&addr_hex)).is_some()
    };

    // 3. Extract raw EVM storage and code changes for all pools
    // This helper automatically populates the builders map
    extract_contract_changes_builder(&block, pool_check, &mut builders);

    // 3b. Decode known pool events into semantic Attributes (entity_changes).
    //     Walk every (log, call) pair in the block, and for each log emitted by
    //     a tracked pool address, decode it into Initialize/Swap/Mint/Burn and
    //     produce decoded attributes (price, tick, liquidity, ticks/{idx}/net-liquidity, ...)
    //     using the per-call storage_changes so each event sees only its own diffs.
    //     This runs side-by-side with raw slot capture: raw slots stay in
    //     contract_changes, decoded attributes go in entity_changes.
    let is_tracked_pool = |address: &[u8]| -> bool {
        let addr_hex = format!("0x{}", hex::encode(address)).to_lowercase();
        created_pools.contains(&addr_hex)
            || pools_store
                .get_last(StoreKey::Pool.get_unique_pool_key(&addr_hex))
                .is_some()
    };

    for trx in block.transactions() {
        let mut tx_attrs_by_pool: HashMap<Vec<u8>, Vec<Attribute>> = HashMap::new();

        for (log, call_view) in trx.logs_with_calls() {
            if !is_tracked_pool(&log.address) {
                continue;
            }
            let pool_addr_arr: [u8; 20] = match log.address.as_slice().try_into() {
                Ok(a) => a,
                Err(_) => continue,
            };
            let attrs = get_log_changed_attributes(
                log,
                &call_view.call.storage_changes,
                &pool_addr_arr,
            );
            if attrs.is_empty() {
                continue;
            }
            tx_attrs_by_pool
                .entry(log.address.clone())
                .or_default()
                .extend(attrs);
        }

        if tx_attrs_by_pool.is_empty() {
            continue;
        }

        let tycho_tx: Transaction = trx.into();
        let builder = builders
            .entry(tycho_tx.index)
            .or_insert_with(|| TransactionChangesBuilder::new(&tycho_tx));

        for (pool_addr, attributes) in tx_attrs_by_pool {
            let component_id = format!("0x{}", hex::encode(&pool_addr)).to_lowercase();
            builder.add_entity_change(&EntityChanges { component_id, attributes });
        }
    }

    // 4. Aggregate balance changes and add them to the builders
    tycho_substreams::balances::aggregate_balances_changes(balance_store, balances_deltas)
        .into_iter()
        .for_each(|(_, (tx, balances))| {
            let builder = builders.entry(tx.index)
                .or_insert_with(|| TransactionChangesBuilder::new(&tx));
            
            for bc_map in balances.values() {
                for bc in bc_map.values() {
                    builder.add_balance_change(bc);
                }
            }
        });

    // 5. Final consolidation: Build TransactionChanges from builders
    let mut final_changes: Vec<TransactionChanges> = builders.into_values()
        .filter_map(|b| b.build())
        .collect();

    final_changes.sort_by_key(|a| a.tx.as_ref().map(|t| t.index).unwrap_or(0));

    Ok(BlockChanges {
        block: Some((&block).into()),
        changes: final_changes,
        storage_changes: Vec::new(),
    })
}
