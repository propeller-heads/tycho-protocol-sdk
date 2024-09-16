use crate::{
    abi,
    pb::contract::v1::{BlockRewardCycles, RewardCycle},
};
use itertools::Itertools;
use std::collections::HashMap;
use substreams::{
    hex,
    pb::substreams::StoreDeltas,
    store::{
        StoreAdd, StoreAddBigInt, StoreAddInt64, StoreGet, StoreGetInt64, StoreGetString, StoreNew,
        StoreSet, StoreSetRaw,
    },
};
use substreams_ethereum::{pb::eth, Event};
use tycho_substreams::{
    balances::aggregate_balances_changes, contract::extract_contract_changes_builder, prelude::*,
};

#[substreams::handlers::map]
pub fn map_components(
    params: String,
    block: eth::v2::Block,
) -> Result<BlockTransactionProtocolComponents, anyhow::Error> {
    let vault_address = hex::decode(params).unwrap();
    let locked_asset = map_vault_to_locked_asset(&vault_address).unwrap();
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
                            Some(create_vault_component(&tx.into(), &vault_address, &locked_asset))
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

// updates the reward rate to be accounted for at each block for the totalAsset locked in the vault
#[substreams::handlers::map]
pub fn map_reward_cycles(
    block: eth::v2::Block,
    components_store: StoreGetString,
) -> Result<BlockRewardCycles, anyhow::Error> {
    let reward_cycles = block
        .logs()
        .filter(|log| {
            components_store
                .get_last(format!("pool:0x{}", hex::encode(log.address())))
                .is_some()
        })
        .filter_map(|vault_log| {
            if let Some(ev) =
                abi::sfraxeth_contract::events::NewRewardsCycle::match_and_decode(vault_log.log)
            {
                Some(RewardCycle {
                    ord: vault_log.ordinal(),
                    next_reward_amount: ev.reward_amount.to_signed_bytes_be(),
                    vault_address: vault_log.address().to_vec(), // be bytes
                })
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    Ok(BlockRewardCycles { reward_cycles })
}

#[substreams::handlers::store]
pub fn store_reward_cycles(block_reward_cycles: BlockRewardCycles, store: StoreSetRaw) {
    block_reward_cycles
        .reward_cycles
        .into_iter()
        .for_each(|reward_cycle| {
            let address_hex = format!("0x{}", hex::encode(&reward_cycle.vault_address));
            store.set(
                reward_cycle.ord,
                format!("reward_cycle:{}", address_hex),
                &reward_cycle.next_reward_amount,
            );
        });
}
#[substreams::handlers::map]
pub fn map_relative_balances(
    block: eth::v2::Block,
    store: StoreGetInt64,
    reward_store: StoreDeltas,
) -> Result<BlockBalanceDeltas, anyhow::Error> {
    let balance_deltas = block
        .logs()
        .filter(|log| map_vault_to_locked_asset(log.address()).is_some())
        .flat_map(|vault_log| {
            let mut deltas = Vec::new();

            if let Some(ev) =
                abi::sfraxeth_contract::events::Withdraw::match_and_decode(vault_log.log)
            {
                let address_bytes_be = vault_log.address();
                let address_hex = format!("0x{}", hex::encode(address_bytes_be));

                if store
                    .get_last(format!("pool:{}", address_hex))
                    .is_some()
                {
                    deltas.extend_from_slice(&[
                        BalanceDelta {
                            ord: vault_log.ordinal(),
                            tx: Some(vault_log.receipt.transaction.into()),
                            token: map_vault_to_locked_asset(address_bytes_be)
                                .unwrap()
                                .to_vec(),
                            delta: ev.assets.neg().to_signed_bytes_be(),
                            component_id: address_hex.as_bytes().to_vec(),
                        },
                        BalanceDelta {
                            ord: vault_log.ordinal(),
                            tx: Some(vault_log.receipt.transaction.into()),
                            token: address_bytes_be.to_vec(),
                            delta: ev.shares.neg().to_signed_bytes_be(),
                            component_id: address_hex.as_bytes().to_vec(),
                        },
                    ])
                }
            } else if let Some(ev) =
                abi::sfraxeth_contract::events::Deposit::match_and_decode(vault_log.log)
            {
                let address_bytes_be = vault_log.address();
                let address_hex = format!("0x{}", hex::encode(address_bytes_be));
                if store
                    .get_last(format!("pool:{}", address_hex))
                    .is_some()
                {
                    deltas.extend_from_slice(&[
                        BalanceDelta {
                            ord: vault_log.ordinal(),
                            tx: Some(vault_log.receipt.transaction.into()),
                            token: map_vault_to_locked_asset(address_bytes_be)
                                .unwrap()
                                .to_vec(),
                            delta: ev.assets.to_signed_bytes_be(),
                            component_id: address_hex.as_bytes().to_vec(),
                        },
                        BalanceDelta {
                            ord: vault_log.ordinal(),
                            tx: Some(vault_log.receipt.transaction.into()),
                            token: address_bytes_be.to_vec(),
                            delta: ev.shares.to_signed_bytes_be(),
                            component_id: address_hex.as_bytes().to_vec(),
                        },
                    ])
                }
            } else if abi::sfraxeth_contract::events::NewRewardsCycle::match_and_decode(vault_log)
                .is_some()
            {
                let address_bytes_be = vault_log.address();
                let address_hex = format!("0x{}", hex::encode(address_bytes_be));
                if store
                    .get_last(format!("pool:{}", address_hex))
                    .is_some()
                {
                    // When the NextRewardsCycle event is emitted:
                    // 1. `lastRewardAmount` is read from storage
                    // 2. `storedTotalAssets` is incremented by the `lastRewardAmount` in the event
                    // 3. `lastRewardAmount` is update with the `nextReward` (2nd parameter) in the
                    //    event
                    // Hence the reward_store at key `reward_cycle:{address_hex}` will is
                    // updated in this block. We want to use the first value of
                    // the record at the beginning of the block (before the store_reward_cycles
                    // writes to that key) ref: https://github.com/FraxFinance/frax-solidity/blob/85039d4dff2fb24d8a1ba6efc1ebf7e464df9dcf/src/hardhat/contracts/FraxETH/sfrxETH.sol.old#L984
                    if let Some(last_reward_amount) = reward_store
                        .deltas
                        .iter()
                        .find(|el| el.key == format!("reward_cycle:{}", address_hex))
                        .map(|el| el.old_value.clone())
                    {
                        deltas.push(BalanceDelta {
                            ord: vault_log.ordinal(),
                            tx: Some(vault_log.receipt.transaction.into()),
                            token: map_vault_to_locked_asset(address_bytes_be)
                                .unwrap()
                                .to_vec(),
                            delta: last_reward_amount,
                            component_id: address_hex.as_bytes().to_vec(),
                        });
                    }
                }
            }

            deltas
        })
        .collect::<Vec<_>>();

    Ok(BlockBalanceDeltas { balance_deltas })
}

/// It's significant to include both the `pool_id` and the `token_id` for each balance delta as the
///  store key to ensure that there's a unique balance being tallied for each.
#[substreams::handlers::store]
pub fn store_balances(deltas: BlockBalanceDeltas, store: StoreAddBigInt) {
    tycho_substreams::balances::store_balance_changes(deltas, store);
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
                .for_each(|bc| builder.add_balance_change(bc));
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

fn create_vault_component(
    tx: &Transaction,
    vault_address: &[u8],
    locked_asset: &[u8],
) -> ProtocolComponent {
    ProtocolComponent::at_contract(vault_address, tx)
        .with_tokens(&[locked_asset, vault_address])
        .as_swap_type("sfraxeth_vault", ImplementationType::Vm)
}

// ref: https://docs.frax.finance/smart-contracts/frxeth-and-sfrxeth-contract-addresses
fn map_vault_to_locked_asset(address_bytes: &[u8]) -> Option<[u8; 20]> {
    // basedo on ADDRESS_MAP create a match condition to return the locked_asset
    match address_bytes {
        hex!("95aB45875cFFdba1E5f451B950bC2E42c0053f39") => {
            // Arbitrum
            Some(hex!("178412e79c25968a32e89b11f63B33F733770c2A"))
        }
        hex!("3Cd55356433C89E50DC51aB07EE0fa0A95623D53") => {
            // BSC
            Some(hex!("64048A7eEcF3a2F1BA9e144aAc3D7dB6e58F555e"))
        }
        hex!("ac3E018457B222d93114458476f3E3416Abbe38F") => {
            // Ethereum
            Some(hex!("5e8422345238f34275888049021821e8e08caa1f"))
        }
        hex!("b90CCD563918fF900928dc529aA01046795ccb4A") => {
            // Fantom
            Some(hex!("9E73F99EE061C8807F69f9c6CCc44ea3d8c373ee"))
        }
        hex!("ecf91116348aF1cfFe335e9807f0051332BE128D") => {
            // Moonbeam
            Some(hex!("82bbd1b6f6De2B7bb63D3e1546e6b1553508BE99"))
        }
        hex!("484c2D6e3cDd945a8B2DF735e079178C1036578c") => {
            // Optimism
            Some(hex!("6806411765Af15Bddd26f8f544A34cC40cb9838B"))
        }
        hex!("6d1FdBB266fCc09A16a22016369210A15bb95761") => {
            // Polygon
            Some(hex!("Ee327F889d5947c1dc1934Bb208a1E792F953E96"))
        }
        _ => None,
    }
}
