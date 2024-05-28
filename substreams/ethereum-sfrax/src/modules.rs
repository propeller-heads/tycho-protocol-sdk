use crate::{
    abi,
    pb::contract::v1::{BlockRewardCycles, RewardCycle}
};
use anyhow::Result;
use itertools::Itertools;
use std::collections::HashMap;
use substreams::{
    hex,
    pb::substreams::StoreDeltas,
    store::{
        StoreAdd, StoreAddBigInt, StoreAddInt64, StoreGet, StoreGetInt64, StoreGetRaw, StoreNew,
        StoreSet, StoreSetRaw,
    },
};
use substreams_ethereum::{pb::eth, Event};
use tycho_substreams::{
    balances::aggregate_balances_changes, contract::extract_contract_changes, prelude::*,
};

const REWARD_CYCLE_LENGTH: u64 = 1000;

// ref: https://docs.frax.finance/smart-contracts/frxeth-and-sfrxeth-contract-addresses
type AddressPair = ([u8; 20], [u8; 20]);
const ADDRESS_MAP: &[AddressPair] = &[
    (
        hex!("95aB45875cFFdba1E5f451B950bC2E42c0053f39"),
        hex!("178412e79c25968a32e89b11f63B33F733770c2A"),
    ), // Arbitrum
    (
        hex!("3Cd55356433C89E50DC51aB07EE0fa0A95623D53"),
        hex!("64048A7eEcF3a2F1BA9e144aAc3D7dB6e58F555e"),
    ), // BSC
    (
        hex!("ac3E018457B222d93114458476f3E3416Abbe38F"),
        hex!("5e8422345238f34275888049021821e8e08caa1f"),
    ), // Ethereum
    (
        hex!("b90CCD563918fF900928dc529aA01046795ccb4A"),
        hex!("9E73F99EE061C8807F69f9c6CCc44ea3d8c373ee"),
    ), // Fantom
    (
        hex!("ecf91116348aF1cfFe335e9807f0051332BE128D"),
        hex!("82bbd1b6f6De2B7bb63D3e1546e6b1553508BE99"),
    ), // Moonbeam
    (
        hex!("484c2D6e3cDd945a8B2DF735e079178C1036578c"),
        hex!("6806411765Af15Bddd26f8f544A34cC40cb9838B"),
    ), // Optimism
    (
        hex!("6d1FdBB266fCc09A16a22016369210A15bb95761"),
        hex!("Ee327F889d5947c1dc1934Bb208a1E792F953E96"),
    ), // Polygon
];

#[substreams::handlers::map]
pub fn map_components(param: String, block: eth::v2::Block) -> Result<BlockTransactionProtocolComponents> {
    let (vault_address, locked_asset) = find_deployed_vault_address(param.as_bytes()).unwrap();

    // We store these as a hashmap by tx hash since we need to agg by tx hash later
    Ok(BlockTransactionProtocolComponents {
        tx_components: block
            .transactions()
            .filter_map(|tx| {
                let components = tx
                    .logs_with_calls()
                    .filter(|(_, call)| !call.call.state_reverted)
                    .filter_map(|(log, _)| {
                        if is_deployment_tx(tx) && log.address == vault_address {
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

#[substreams::handlers::map]
pub fn map_rewards_cycle(block: eth::v2::Block) -> Result<BlockRewardCycles, anyhow::Error> {
    let reward_cycles = block
        .logs()
        .filter_map(|vault_log| {
            if let Some(ev) =
                abi::stakedfrax_contract::events::SyncRewards::match_and_decode(vault_log.log)
            {
                let reward_cycle = ev.cycle_end - substreams::scalar::BigInt::from(block.number);
                let reward_rate = ev
                    .reward_cycle_amount.div_rem(&reward_cycle.to_owned()).0;
                Some(RewardCycle {
                    ord: vault_log.ordinal(),
                    reward_rate: reward_rate.to_signed_bytes_be(),
                    component_id: hex::encode(vault_log.address()),
                })
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    Ok(BlockRewardCycles { reward_cycles })
}

pub fn store_reward_cycles(block_reward_cycles: BlockRewardCycles, store: StoreSetRaw) {
    block_reward_cycles
        .reward_cycles
        .into_iter()
        .for_each(|reward_cycle| {
            store.set(
                reward_cycle.ord,
                format!("reward_cycle:{0}", reward_cycle.component_id),
                &reward_cycle.reward_rate,
            );
        });
}

/// Since the `PoolBalanceChanged` and `Swap` events administer only deltas, we need to leverage a
/// map and a  store to be able to tally up final balances for tokens in a pool.
#[substreams::handlers::map]
pub fn map_relative_balances(
    block: eth::v2::Block,
    store: StoreGetInt64,
    reward_store: StoreGetRaw,
) -> Result<BlockBalanceDeltas, anyhow::Error> {
    let mut balance_deltas = block
        .logs()
        .filter(|log| find_deployed_vault_address(log.address()).is_some())
        .flat_map(|vault_log| {
            let mut deltas = Vec::new();

            if let Some(ev) =
                abi::stakedfrax_contract::events::Withdraw::match_and_decode(vault_log.log)
            {
                let contract_address = vault_log.address();
                if store
                    .get_last(format!("pool:{0}", hex::encode(contract_address)))
                    .is_some()
                {
                    deltas.extend_from_slice(&[
                        BalanceDelta {
                            ord: vault_log.ordinal(),
                            tx: Some(vault_log.receipt.transaction.into()),
                            token: match_underlying_asset(contract_address)
                                .unwrap()
                                .to_vec(),
                            delta: ev.assets.neg().to_signed_bytes_be(),
                            component_id: contract_address.to_vec(),
                        }, 
                        BalanceDelta {
                            ord: vault_log.ordinal(),
                            tx: Some(vault_log.receipt.transaction.into()),
                            token: contract_address.to_vec(),
                            delta: ev.shares.neg().to_signed_bytes_be(),
                            component_id: contract_address.to_vec(),
                        }
                    ])
                }
            } else if let Some(ev) =
                abi::stakedfrax_contract::events::Deposit::match_and_decode(vault_log.log)
            {
                let contract_address = vault_log.address();
                if store
                    .get_last(format!("pool:{0}", hex::encode(contract_address)))
                    .is_some()
                {
                    deltas.extend_from_slice(&[
                        BalanceDelta {
                            ord: vault_log.ordinal(),
                            tx: Some(vault_log.receipt.transaction.into()),
                            token: match_underlying_asset(contract_address)
                                .unwrap()
                                .to_vec(),
                            delta: ev.assets.to_signed_bytes_be(),
                            component_id: contract_address.to_vec(),
                        }, 
                        BalanceDelta {
                            ord: vault_log.ordinal(),
                            tx: Some(vault_log.receipt.transaction.into()),
                            token: contract_address.to_vec(),
                            delta: ev.shares.to_signed_bytes_be(),
                            component_id: contract_address.to_vec(),
                        }
                    ])
                }
            }

            deltas
        })
        .collect::<Vec<_>>();
    
    // once per block increase the fraxEth (i.e. value of sfraxEth.totalAssets()) by the reward rate
    // use first tx as placeholder
    ADDRESS_MAP
        .iter()
        .for_each(|(vault_address, _)| {

            if let Some(reward_rate_signed_be_bytes) =
                reward_store.get_last(format!("reward_cycle:{0}", hex::encode(vault_address)))
            {
                // ensure ord must be strictly increasing for each token address
                let ord = block
                    .transactions()
                    .last()
                    .map_or(1, |tx| tx.end_ordinal + 1);
                balance_deltas.push(BalanceDelta {
                    ord,
                    tx: None,
                    token: match_underlying_asset(vault_address)
                        .unwrap()
                        .to_vec(),
                    delta: reward_rate_signed_be_bytes,
                    component_id: vault_address.to_vec(),
                })
            }
            return;
        });

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
    components_store: StoreGetInt64,
    balance_store: StoreDeltas, // Note, this map module is using the `deltas` mode for the store.
) -> Result<BlockContractChanges> {
    // We merge contract changes by transaction (identified by transaction index) making it easy to
    //  sort them at the very end.
    let mut transaction_contract_changes: HashMap<_, TransactionContractChanges> = HashMap::new();

    // `ProtocolComponents` are gathered from `map_pools_created` which just need a bit of work to
    //   convert into `TransactionContractChanges`
    grouped_components
        .tx_components
        .iter()
        .for_each(|tx_component| {
            let tx = tx_component.tx.as_ref().unwrap();
            transaction_contract_changes
                .entry(tx.index)
                .or_insert_with(|| TransactionContractChanges::new(tx))
                .component_changes
                .extend_from_slice(&tx_component.components);
        });

    // Balance changes are gathered by the `StoreDelta` based on `PoolBalanceChanged` creating
    //  `BlockBalanceDeltas`. We essentially just process the changes that occurred to the `store`
    // this  block. Then, these balance changes are merged onto the existing map of tx contract
    // changes,  inserting a new one if it doesn't exist.
    aggregate_balances_changes(balance_store, deltas)
        .into_iter()
        .for_each(|(_, (tx, balances))| {
            transaction_contract_changes
                .entry(tx.index)
                .or_insert_with(|| TransactionContractChanges::new(&tx))
                .balance_changes
                .extend(balances.into_values());
        });

    // Extract and insert any storage changes that happened for any of the components.
    extract_contract_changes(
        &block,
        |addr| {
            components_store
                .get_last(format!("pool:0x{0}", hex::encode(addr)))
                .is_some()
        },
        &mut transaction_contract_changes,
    );

    // Process all `transaction_contract_changes` for final output in the `BlockContractChanges`,
    //  sorted by transaction index (the key).
    Ok(BlockContractChanges {
        block: Some((&block).into()),
        changes: transaction_contract_changes
            .drain()
            .sorted_unstable_by_key(|(index, _)| *index)
            .filter_map(|(_, change)| {
                if change.contract_changes.is_empty() &&
                    change.component_changes.is_empty() &&
                    change.balance_changes.is_empty()
                {
                    None
                } else {
                    Some(change)
                }
            })
            .collect::<Vec<_>>(),
    })
}

fn is_deployment_tx(tx: &eth::v2::TransactionTrace) -> bool {
    let zero_address = hex!("0000000000000000000000000000000000000000");
    tx.to.as_slice() == zero_address || tx.to.is_empty()
}

fn create_vault_component(
    tx: &Transaction,
    component_id: &[u8],
    locked_asset: &[u8],
) -> ProtocolComponent {
    ProtocolComponent::at_contract(component_id, tx)
        .with_tokens(&[locked_asset, component_id])
        .as_swap_type("sfrax_vault", ImplementationType::Vm)
}

fn match_underlying_asset(address: &[u8]) -> Option<[u8; 20]> {
    find_deployed_vault_address(address).map(|(_, underlying_asset)| underlying_asset)
}

fn find_deployed_vault_address(vault_address: &[u8]) -> Option<AddressPair> {
    ADDRESS_MAP
        .iter()
        .find(|(_, addr)| addr == vault_address)
        .copied()
}
