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
