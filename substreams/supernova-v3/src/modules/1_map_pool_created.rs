use std::str::FromStr;

use ethabi::ethereum_types::Address;
use serde::Deserialize;
use substreams_ethereum::pb::eth::v2::{self as eth};
use substreams_ethereum::Event;

use crate::abi;
use crate::abi::algebrapool::events::Plugin as PluginEvent;

use tycho_substreams::prelude::*;

/// Collect every storage write inside `trx` whose target is `target_addr`.
/// Returns them as `ContractSlot` records, ready to attach to a
/// `ContractChange`. This walks the FULL transaction call tree (not just
/// the call that emitted the Pool event) so a constructor that writes its
/// storage from a delegatecalled library is still captured.
///
/// Why we do this even though `extract_contract_changes_builder` in
/// module 3 already collects the same writes: keeping the slot data
/// attached to module 1's `ContractChange` makes the data-flow contract
/// of "module 1 emits a fully-self-describing pool creation" explicit.
/// If `extract_contract_changes_builder` ever tightens its filtering or
/// changes its semantics, the substream output remains complete.
fn collect_storage_writes_for(
    trx: &eth::TransactionTrace,
    target_addr: &[u8],
) -> Vec<ContractSlot> {
    let mut out: Vec<ContractSlot> = Vec::new();
    for call in &trx.calls {
        if call.state_reverted {
            continue;
        }
        for sc in &call.storage_changes {
            if sc.address == target_addr {
                out.push(ContractSlot {
                    slot: sc.key.clone(),
                    value: sc.new_value.clone(),
                    previous_value: sc.old_value.clone(),
                });
            }
        }
    }
    out
}

#[derive(Debug, Deserialize)]
struct Params {
    factory_address: String,
    pool_address: Option<String>,
}

#[substreams::handlers::map]
pub fn map_pools_created(
    params: String,
    block: eth::Block,
) -> Result<BlockChanges, substreams::errors::Error> {
    let mut new_pools: Vec<TransactionChanges> = vec![];

    let query_params: Params =
        serde_qs::from_str(params.as_str()).expect("Unable to deserialize params");

    get_pools(&block, &mut new_pools, &query_params);

    let tycho_block: Block = (&block).into();

    Ok(BlockChanges { block: Some(tycho_block), changes: new_pools, storage_changes: Vec::new() })
}

fn get_pools(block: &eth::Block, new_pools: &mut Vec<TransactionChanges>, query_params: &Params) {
    let factory_addr = Address::from_str(&query_params.factory_address).unwrap();
    let target_pool = query_params.pool_address.as_ref().map(|p| Address::from_str(p).unwrap());

    for trx in block.transactions() {
        let tycho_tx: Transaction = trx.into();

        for (log, call_view) in trx.logs_with_calls() {
            if log.address != factory_addr.as_bytes() {
                continue;
            }

            let pool_addr =
                if let Some(event) = abi::algebrafactory::events::Pool::match_and_decode(log) {
                    substreams::log::info!(
                        "Pool Created: address=0x{} token0=0x{} token1=0x{}",
                        substreams::Hex(&event.pool),
                        substreams::Hex(&event.token0),
                        substreams::Hex(&event.token1)
                    );
                    Some((event.pool, event.token0, event.token1))
                } else if let Some(event) =
                    abi::algebrafactory::events::CustomPool::match_and_decode(log)
                {
                    substreams::log::info!(
                        "CustomPool Created: address={} token0={} token1={}",
                        substreams::Hex(&event.pool),
                        substreams::Hex(&event.token0),
                        substreams::Hex(&event.token1)
                    );
                    Some((event.pool, event.token0, event.token1))
                } else {
                    None
                };

            if let Some((pool_address, token0, token1)) = pool_addr {
                // Address Filtering
                if let Some(target) = &target_pool {
                    if pool_address != target.as_bytes() {
                        continue;
                    }
                }

                let pool_id = format!("0x{}", hex::encode(&pool_address)).to_lowercase();
                let _factory_id = format!("0x{}", hex::encode(&factory_addr)).to_lowercase();

                let mut pool_change = ContractChange {
                    address: pool_address.clone(),
                    ..Default::default()
                };
                pool_change.change = ChangeType::Creation.into();

                // 1. Capture bytecode if present in this transaction
                for code_change in &call_view.call.code_changes {
                    if code_change.address == pool_address {
                        pool_change.code = code_change.new_code.clone();
                    }
                }

                // 1b. Capture every storage write the pool's constructor (and
                //     any same-tx initialisation calls) made to its own address.
                //     This includes slot 2 (globalState), slot 6 (plugin), slot
                //     7 (communityVault), and the rest of the swap-critical
                //     slots — all of which are written exactly once at creation
                //     time and (because the indexer doesn't version slots
                //     historically) need to be present in the substream output
                //     so the simulator can rebuild the pool's state at any
                //     query block.
                pool_change.slots = collect_storage_writes_for(trx, &pool_address);

                // 2. Find the plugin contract address by scanning this tx's logs for the
                //    Plugin(address) event emitted by the new pool during construction
                //    (AlgebraPoolBase._setPlugin → emit Plugin(_plugin)).
                let mut plugin_address: Option<Vec<u8>> = None;
                if let Some(receipt) = trx.receipt.as_ref() {
                    for plugin_log in &receipt.logs {
                        if plugin_log.address == pool_address.as_slice()
                            && PluginEvent::match_log(plugin_log)
                        {
                            if let Ok(ev) = PluginEvent::decode(plugin_log) {
                                if !ev.new_plugin_address.iter().all(|b| *b == 0) {
                                    plugin_address = Some(ev.new_plugin_address);
                                    break;
                                }
                            }
                        }
                    }
                }

                // 3. If a plugin was registered, capture its bytecode (if deployed in
                //    this same tx) and ensure it's part of the tracked contracts list.
                let mut contracts_list = vec![pool_address.clone()];
                let mut contract_changes = vec![pool_change];
                if let Some(plugin_addr) = plugin_address.as_ref() {
                    let mut plugin_change = ContractChange {
                        address: plugin_addr.clone(),
                        ..Default::default()
                    };
                    plugin_change.change = ChangeType::Creation.into();
                    for code_change in &call_view.call.code_changes {
                        if code_change.address == plugin_addr.as_slice() {
                            plugin_change.code = code_change.new_code.clone();
                        }
                    }
                    // Same belt-and-suspenders capture as the pool: pull every
                    // constructor storage write that targets the plugin so its
                    // initial state (including dependency addresses we discover
                    // at simulation time via the harness's plugin scan) is in
                    // the substream output from block 0.
                    plugin_change.slots = collect_storage_writes_for(trx, plugin_addr);
                    contracts_list.push(plugin_addr.clone());
                    contract_changes.push(plugin_change);
                    substreams::log::info!(
                        "Plugin registered for pool 0x{}: 0x{}",
                        substreams::Hex(&pool_address),
                        substreams::Hex(plugin_addr)
                    );
                }

                let mut static_att = vec![
                    Attribute {
                        name: "token0".to_string(),
                        value: token0.clone(),
                        change: ChangeType::Creation.into(),
                    },
                    Attribute {
                        name: "token1".to_string(),
                        value: token1.clone(),
                        change: ChangeType::Creation.into(),
                    },
                    // Algebra Integral pools are CREATE2'd by `AlgebraPoolDeployer`,
                    // NOT by the factory directly. The factory CREATE2 prefix is
                    //   keccak256(0xff || poolDeployer || keccak256(token0,token1) || INIT_CODE_HASH)
                    // (see Algebra/src/core/contracts/AlgebraFactory.sol#L91).
                    //
                    // We expose the factory address here under the name `factory`
                    // (NOT `deployer`) so downstream consumers don't accidentally
                    // recompute pool addresses against the wrong contract. The
                    // actual pool deployer is `IAlgebraFactory(factory).poolDeployer()`
                    // — substreams can't issue eth_calls cheaply, so consumers that
                    // need it should resolve it once via RPC.
                    Attribute {
                        name: "factory".to_string(),
                        value: factory_addr.as_bytes().to_vec(),
                        change: ChangeType::Creation.into(),
                    },
                ];
                if let Some(plugin_addr) = plugin_address.as_ref() {
                    static_att.push(Attribute {
                        name: "plugin".to_string(),
                        value: plugin_addr.clone(),
                        change: ChangeType::Creation.into(),
                    });
                }

                new_pools.push(TransactionChanges {
                    tx: Some(tycho_tx.clone()),
                    contract_changes,
                    component_changes: vec![ProtocolComponent {
                        id: pool_id.clone(),
                        tokens: vec![token0.clone(), token1.clone()],
                        contracts: contracts_list,
                        static_att,
                        change: i32::from(ChangeType::Creation),
                        protocol_type: Some(ProtocolType {
                            name: "supernova_algebra_pool_vm".to_string(),
                            financial_type: FinancialType::Swap.into(),
                            attribute_schema: vec![],
                            implementation_type: ImplementationType::Vm.into(),
                        }),
                    }],
                    ..Default::default()
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use substreams_ethereum::pb::eth::v2::{Call, StorageChange};

    /// Build a synthetic single-call `TransactionTrace` whose call has
    /// the given storage changes (each tuple is `(slot_key, old_value,
    /// new_value)`) and an explicit `state_reverted` flag.
    fn make_trx(
        target: &[u8],
        changes: Vec<(Vec<u8>, Vec<u8>, Vec<u8>)>,
        reverted: bool,
    ) -> eth::TransactionTrace {
        let storage_changes: Vec<StorageChange> = changes
            .into_iter()
            .map(|(key, old, new)| StorageChange {
                address: target.to_vec(),
                key,
                old_value: old,
                new_value: new,
                ordinal: 0,
            })
            .collect();
        eth::TransactionTrace {
            calls: vec![Call {
                state_reverted: reverted,
                storage_changes,
                ..Default::default()
            }],
            ..Default::default()
        }
    }

    /// Build a multi-call transaction where two calls touch a "pool"
    /// address and one call touches an unrelated address. Returns the
    /// trace plus the two addresses for assertion convenience.
    fn make_trx_multi_call() -> (eth::TransactionTrace, Vec<u8>, Vec<u8>) {
        let pool = vec![0xaau8; 20];
        let other = vec![0xbbu8; 20];
        let trx = eth::TransactionTrace {
            calls: vec![
                Call {
                    state_reverted: false,
                    storage_changes: vec![
                        StorageChange {
                            address: pool.clone(),
                            key: vec![0x01],
                            old_value: vec![0x00],
                            new_value: vec![0x42],
                            ordinal: 0,
                        },
                        StorageChange {
                            address: other.clone(),
                            key: vec![0x99],
                            old_value: vec![0x00],
                            new_value: vec![0xff],
                            ordinal: 1,
                        },
                    ],
                    ..Default::default()
                },
                Call {
                    state_reverted: false,
                    storage_changes: vec![StorageChange {
                        address: pool.clone(),
                        key: vec![0x02],
                        old_value: vec![0x00],
                        new_value: vec![0x43],
                        ordinal: 2,
                    }],
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        (trx, pool, other)
    }

    /// Across multiple calls, all writes to the target address are
    /// captured and writes to other addresses are ignored.
    #[test]
    fn collect_storage_writes_returns_only_target_address() {
        let (trx, pool, _other) = make_trx_multi_call();
        let slots = collect_storage_writes_for(&trx, &pool);
        assert_eq!(slots.len(), 2, "should pick up both writes to the pool");
        assert_eq!(slots[0].slot, vec![0x01]);
        assert_eq!(slots[0].value, vec![0x42]);
        assert_eq!(slots[1].slot, vec![0x02]);
        assert_eq!(slots[1].value, vec![0x43]);
    }

    /// Mirror of the above for the unrelated address — only the one
    /// write that targets it should come back.
    #[test]
    fn collect_storage_writes_other_address_isolated() {
        let (trx, _pool, other) = make_trx_multi_call();
        let slots = collect_storage_writes_for(&trx, &other);
        assert_eq!(slots.len(), 1);
        assert_eq!(slots[0].slot, vec![0x99]);
        assert_eq!(slots[0].value, vec![0xff]);
    }

    /// Reverted calls must contribute zero storage writes — those state
    /// changes never landed on chain and including them would corrupt
    /// the substream output.
    #[test]
    fn collect_storage_writes_skips_reverted_calls() {
        let target = vec![0xccu8; 20];
        let trx = make_trx(
            &target,
            vec![(vec![0x05], vec![0x00], vec![0x42])],
            true, // reverted
        );
        let slots = collect_storage_writes_for(&trx, &target);
        assert!(slots.is_empty(), "reverted calls must not contribute slots");
    }

    /// Both old and new values must round-trip into the emitted
    /// `ContractSlot`. The indexer relies on `previous_value` to do
    /// efficient deduplication.
    #[test]
    fn collect_storage_writes_preserves_old_and_new_values() {
        let target = vec![0xddu8; 20];
        let trx = make_trx(
            &target,
            vec![(vec![0x07], vec![0xaa, 0xbb], vec![0xcc, 0xdd])],
            false,
        );
        let slots = collect_storage_writes_for(&trx, &target);
        assert_eq!(slots.len(), 1);
        assert_eq!(slots[0].slot, vec![0x07]);
        assert_eq!(slots[0].previous_value, vec![0xaa, 0xbb]);
        assert_eq!(slots[0].value, vec![0xcc, 0xdd]);
    }

    /// An empty transaction is the trivial input — must not panic and
    /// must produce zero slots.
    #[test]
    fn collect_storage_writes_empty_transaction() {
        let trx = eth::TransactionTrace::default();
        let slots = collect_storage_writes_for(&trx, &[0u8; 20]);
        assert!(slots.is_empty());
    }

    /// Address comparison must be byte-exact. Two addresses that
    /// differ in only the last byte must NOT match.
    #[test]
    fn collect_storage_writes_byte_exact_address_match() {
        let target = [0x10u8; 20];
        let mut almost = target.to_vec();
        almost[19] = 0x11; // last byte differs
        let trx = make_trx(
            &almost,
            vec![(vec![0x01], vec![0x00], vec![0x42])],
            false,
        );
        let slots = collect_storage_writes_for(&trx, &target);
        assert!(
            slots.is_empty(),
            "address that differs by even one byte must not be considered a match"
        );
    }

    /// A constructor that writes the same slot twice (e.g. once to
    /// initialise, then to update) is a real pattern. We do NOT
    /// dedupe in the substream — the indexer-side merger handles
    /// last-write-wins. Verify both entries are emitted in order.
    #[test]
    fn collect_storage_writes_multiple_writes_same_slot_preserved() {
        let target = vec![0xeeu8; 20];
        let trx = make_trx(
            &target,
            vec![
                (vec![0x01], vec![0x00], vec![0x10]),
                (vec![0x01], vec![0x10], vec![0x20]),
            ],
            false,
        );
        let slots = collect_storage_writes_for(&trx, &target);
        assert_eq!(slots.len(), 2);
        assert_eq!(slots[0].value, vec![0x10]);
        assert_eq!(slots[1].value, vec![0x20]);
    }

    /// One call has reverted writes, another has succeeded writes.
    /// Only the latter should be captured.
    #[test]
    fn collect_storage_writes_mixed_reverted_and_successful_calls() {
        let target = vec![0x33u8; 20];
        let trx = eth::TransactionTrace {
            calls: vec![
                Call {
                    state_reverted: true,
                    storage_changes: vec![StorageChange {
                        address: target.clone(),
                        key: vec![0x01],
                        old_value: vec![],
                        new_value: vec![0xaa],
                        ordinal: 0,
                    }],
                    ..Default::default()
                },
                Call {
                    state_reverted: false,
                    storage_changes: vec![StorageChange {
                        address: target.clone(),
                        key: vec![0x02],
                        old_value: vec![],
                        new_value: vec![0xbb],
                        ordinal: 1,
                    }],
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        let slots = collect_storage_writes_for(&trx, &target);
        assert_eq!(slots.len(), 1, "only the successful call's writes should land");
        assert_eq!(slots[0].slot, vec![0x02]);
        assert_eq!(slots[0].value, vec![0xbb]);
    }
}
