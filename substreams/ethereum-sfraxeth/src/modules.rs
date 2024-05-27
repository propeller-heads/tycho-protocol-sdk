use crate::{
    abi,
    pb::contract::v1::{BlockRewardCycles, RewardCycle},
};
use itertools::Itertools;
use std::{collections::HashMap, ops::Div};
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

const VAULT_ADDRESS: [u8; 20] = hex!("ac3E018457B222d93114458476f3E3416Abbe38F");
const LOCKED_ASSET_ADDRESS: [u8; 20] = hex!("5E8422345238F34275888049021821E8E08CAa1f");
const FRAX_ALT_DEPLOYER: [u8; 20] = hex!("4600D3b12c39AF925C2C07C487d31D17c1e32A35"); // https://etherscan.io/tx/0xd78dbe6cba652eb844de5aa473636c202fb6366c1bfc5ff8d5a26c1a24b37b07

#[substreams::handlers::map]
pub fn map_components(
    block: eth::v2::Block,
) -> Result<BlockTransactionProtocolComponents, anyhow::Error> {
    // We store these as a hashmap by tx hash since we need to agg by tx hash later
    Ok(BlockTransactionProtocolComponents {
        tx_components: block
            .transactions()
            .filter_map(|tx| {
                let components = tx
                    .logs_with_calls()
                    .filter(|(_, call)| !call.call.state_reverted)
                    .filter_map(|(log, _)| {
                        if is_deployment_tx_from_deployer(tx, FRAX_ALT_DEPLOYER)
                            && log.address == VAULT_ADDRESS
                        {
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

// updates the reward rate to be accounted for at each block for the totalAsset locked in the vault
#[substreams::handlers::map]
pub fn map_rewards_cycle(block: eth::v2::Block) -> Result<BlockRewardCycles, anyhow::Error> {
    let reward_cycles = block
        .logs()
        .filter(|log| log.address() == VAULT_ADDRESS)
        .filter_map(|vault_log| {
            if let Some(ev) =
                abi::sfraxeth_contract::events::NewRewardsCycle::match_and_decode(vault_log.log)
            {
                let reward_cycle = ev.cycle_end - substreams::scalar::BigInt::from(block.number);
                let reward_rate = ev
                    .reward_amount
                    .div(reward_cycle.to_owned());
                Some(RewardCycle {
                    ord: vault_log.ordinal(),
                    reward_rate: reward_rate.to_signed_bytes_be(),
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
            store.set(
                reward_cycle.ord,
                format!("reward_cycle:{0}", hex::encode(&VAULT_ADDRESS)),
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
        .filter(|log| log.address() == VAULT_ADDRESS)
        .flat_map(|vault_log| {
            let mut deltas = Vec::new();

            if let Some(ev) =
                abi::sfraxeth_contract::events::Withdraw::match_and_decode(vault_log.log)
            {
                if store
                    .get_last(format!("pool:{0}", hex::encode(VAULT_ADDRESS)))
                    .is_some()
                {
                    deltas.extend_from_slice(&[
                        BalanceDelta {
                            ord: vault_log.ordinal(),
                            tx: Some(vault_log.receipt.transaction.into()),
                            token: LOCKED_ASSET_ADDRESS.to_vec(),
                            delta: ev.assets.neg().to_signed_bytes_be(),
                            component_id: VAULT_ADDRESS.to_vec(),
                        },
                        BalanceDelta {
                            ord: vault_log.ordinal(),
                            tx: Some(vault_log.receipt.transaction.into()),
                            token: VAULT_ADDRESS.to_vec(),
                            delta: ev.shares.neg().to_signed_bytes_be(),
                            component_id: VAULT_ADDRESS.to_vec(),
                        },
                    ])
                }
            } else if let Some(ev) =
                abi::sfraxeth_contract::events::Deposit::match_and_decode(vault_log.log)
            {
                if store
                    .get_last(format!("pool:{0}", hex::encode(VAULT_ADDRESS)))
                    .is_some()
                {
                    deltas.extend_from_slice(&[
                        BalanceDelta {
                            ord: vault_log.ordinal(),
                            tx: Some(vault_log.receipt.transaction.into()),
                            token: LOCKED_ASSET_ADDRESS.to_vec(),
                            delta: ev.assets.to_signed_bytes_be(),
                            component_id: VAULT_ADDRESS.to_vec(),
                        },
                        BalanceDelta {
                            ord: vault_log.ordinal(),
                            tx: Some(vault_log.receipt.transaction.into()),
                            token: VAULT_ADDRESS.to_vec(),
                            delta: ev.shares.to_signed_bytes_be(),
                            component_id: VAULT_ADDRESS.to_vec(),
                        },
                    ])
                }
            }

            deltas
        })
        .collect::<Vec<_>>();

    // once per block increase the fraxEth (storedTotalAssets) by the per block reward amount
    // use the first tx as placeholder
    if let Some(first_tx) = block.transactions().next() {
        if let Some(reward_rate_signed_be_bytes) =
            reward_store.get_last(format!("reward_cycle:{0}", hex::encode(&VAULT_ADDRESS)))
        {
            balance_deltas.push(BalanceDelta {
                ord: first_tx.begin_ordinal,
                tx: Some(first_tx.into()),
                token: LOCKED_ASSET_ADDRESS.to_vec(),
                delta: reward_rate_signed_be_bytes,
                component_id: VAULT_ADDRESS.to_vec(),
            })
        }
    }

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
) -> Result<BlockContractChanges, anyhow::Error> {
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
                if change.contract_changes.is_empty()
                    && change.component_changes.is_empty()
                    && change.balance_changes.is_empty()
                {
                    None
                } else {
                    Some(change)
                }
            })
            .collect::<Vec<_>>(),
    })
}

fn is_deployment_tx_from_deployer(
    tx: &eth::v2::TransactionTrace,
    deployer_address: [u8; 20],
) -> bool {
    let zero_address = hex!("0000000000000000000000000000000000000000");
    tx.to.as_slice() == zero_address && tx.from.as_slice() == deployer_address
}

fn create_vault_component(tx: &Transaction) -> ProtocolComponent {
    ProtocolComponent::at_contract(VAULT_ADDRESS.as_slice(), tx)
        .with_tokens(&[LOCKED_ASSET_ADDRESS])
        .as_swap_type("sfraxeth_vault", ImplementationType::Vm)
}
