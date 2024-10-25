use crate::{
    abi,
    consts::{self, ADDRESS_ZERO, LIQUIDITY_POOL_CREATION_HASH, WEETH_CREATION_HASH},
};
use anyhow::Result;
use consts::{EETH_ADDRESS, ETH_ADDRESS, LIQUIDITY_POOL_ADDRESS, WEETH_ADDRESS};
use itertools::Itertools;
use std::collections::HashMap;
use substreams::{
    pb::substreams::StoreDeltas,
    store::{
        StoreAddBigInt, StoreGet, StoreGetProto, StoreGetString, StoreNew, StoreSet, StoreSetProto,
    },
};
use substreams_ethereum::{pb::eth, Event};
use tycho_substreams::{
    balances::aggregate_balances_changes, contract::extract_contract_changes_builder, prelude::*,
};

#[substreams::handlers::map]
pub fn map_components(
    block: eth::v2::Block,
) -> Result<BlockTransactionProtocolComponents, anyhow::Error> {
    Ok(BlockTransactionProtocolComponents {
        tx_components: block
            .transactions()
            .filter_map(|tx| {
                let mut components: Vec<ProtocolComponent> = vec![];
                if tx.hash == LIQUIDITY_POOL_CREATION_HASH {
                    components.push(
                        ProtocolComponent::at_contract(&LIQUIDITY_POOL_ADDRESS, &tx.into())
                            .with_tokens(&[EETH_ADDRESS, ETH_ADDRESS])
                            .as_swap_type("etherfi_liquidity_pool", ImplementationType::Vm),
                    )
                } else if tx.hash == WEETH_CREATION_HASH {
                    components.push(
                        ProtocolComponent::at_contract(&WEETH_ADDRESS, &tx.into())
                            .with_tokens(&[EETH_ADDRESS, WEETH_ADDRESS])
                            .as_swap_type("etherfi_weeth_pool", ImplementationType::Vm),
                    )
                }

                if !components.is_empty() {
                    Some(TransactionProtocolComponents { tx: Some(tx.into()), components })
                } else {
                    None
                }
            })
            .collect::<Vec<_>>(),
    })
}

/// Stores the `ProtocolComponent`s with the pool id as the key, together with the token pair as
/// events do not contain the pair info
#[substreams::handlers::store]
pub fn store_components(
    map: BlockTransactionProtocolComponents,
    store: StoreSetProto<ProtocolComponent>,
) {
    map.tx_components
        .iter()
        .for_each(|tx_components| {
            tx_components
                .components
                .iter()
                .for_each(|component| store_component(&store, component));
        })
}

/// we need to leverage a
/// map and a  store to be able to tally up final balances for tokens in a pool.
#[substreams::handlers::map]
pub fn map_relative_balances(
    block: eth::v2::Block,
    store: StoreGetProto<ProtocolComponent>,
) -> Result<BlockBalanceDeltas, anyhow::Error> {
    let liquidity_pool_hex = format!("0x{}", hex::encode(LIQUIDITY_POOL_ADDRESS));
    let weeth_hex = format!("0x{}", hex::encode(WEETH_ADDRESS));
    let balance_deltas = block
        .logs()
        // .filter(|log| log.address()) // filter out logs that are not from the pool contract
        .flat_map(|log| {
            let mut deltas = Vec::new();

            // Liquidty Pool Deposit:
            // Contract balance just becomes += ETH, eeth balance is handled by eeth TransferShares
            // event
            if let Some(ev) = abi::pool_contract::events::Deposit::match_and_decode(log.log) {
                if store
                    .get_last(format!("pool:{}", liquidity_pool_hex))
                    .is_some()
                {
                    substreams::log::info!("Liquidity Pool Deposit: +ETH {}", ev.amount);

                    deltas.push(BalanceDelta {
                        ord: log.ordinal(),
                        tx: Some(log.receipt.transaction.into()),
                        token: ETH_ADDRESS.to_vec(),
                        delta: ev.amount.to_signed_bytes_be(),
                        component_id: LIQUIDITY_POOL_ADDRESS.to_vec(),
                    });
                }
            }
            // Liquidty Pool Withdraw:
            // Contract balance just becomes -= ETH, eeth balance is handled by eeth TransferShares
            // event
            else if let Some(ev) = abi::pool_contract::events::Withdraw::match_and_decode(log.log)
            {
                if store
                    .get_last(format!("pool:{}", liquidity_pool_hex))
                    .is_some()
                {
                    // Shares are burnt, therefore not held by the contract; Contract balance just
                    // becomes -= ETH
                    substreams::log::info!("Liquidity Pool Withdraw: -ETH {}", ev.amount);

                    deltas.push(BalanceDelta {
                        ord: log.ordinal(),
                        tx: Some(log.receipt.transaction.into()),
                        token: ETH_ADDRESS.to_vec(),
                        delta: ev.amount.neg().to_signed_bytes_be(),
                        component_id: LIQUIDITY_POOL_ADDRESS.to_vec(),
                    });
                }
            }
            // EETH transfer shares:
            // Contract balance just becomes +/-= eETH, burn: Receiver is address(0), mint: Sender
            // is address(0)
            else if let Some(ev) =
                abi::eeth_contract::events::TransferShares::match_and_decode(log.log)
            {
                // Actions to liquidity pool
                if store
                    .get_last(format!("pool:{}", liquidity_pool_hex))
                    .is_some()
                {
                    // Mint Shares: Contract Balance += eETH
                    if ev.from == ADDRESS_ZERO {
                        substreams::log::info!(
                            "Liquidity Pool Deposit eeTH: +eETH {}",
                            ev.shares_value
                        );

                        deltas.push(BalanceDelta {
                            ord: log.ordinal(),
                            tx: Some(log.receipt.transaction.into()),
                            token: EETH_ADDRESS.to_vec(),
                            delta: ev.shares_value.to_signed_bytes_be(),
                            component_id: LIQUIDITY_POOL_ADDRESS.to_vec(),
                        });
                    }
                    // Burn Shares: Contract Balance -= eETH
                    else if ev.to == ADDRESS_ZERO {
                        substreams::log::info!(
                            "Liquidity Pool Withdraw eeTH: -eETH {}",
                            ev.shares_value
                        );

                        deltas.push(BalanceDelta {
                            ord: log.ordinal(),
                            tx: Some(log.receipt.transaction.into()),
                            token: EETH_ADDRESS.to_vec(),
                            delta: ev
                                .shares_value
                                .neg()
                                .to_signed_bytes_be(),
                            component_id: LIQUIDITY_POOL_ADDRESS.to_vec(),
                        });
                    }
                }

                // Actions to weeth
                if store
                    .get_last(format!("pool:{}", weeth_hex))
                    .is_some()
                {
                    // Deposit eETH(wrap) into weETH contract: weETH Balane += eETH
                    if ev.to == WEETH_ADDRESS {
                        substreams::log::info!(
                            "Deposit eeTH into weETH: +eETH {}",
                            ev.shares_value
                        );

                        deltas.push(BalanceDelta {
                            ord: log.ordinal(),
                            tx: Some(log.receipt.transaction.into()),
                            token: EETH_ADDRESS.to_vec(),
                            delta: ev.shares_value.to_signed_bytes_be(),
                            component_id: WEETH_ADDRESS.to_vec(),
                        });
                    }
                    // Withdraw eEth(unwrap) from weETH contract: weETH Balane -= eETH
                    else if ev.from == WEETH_ADDRESS {
                        substreams::log::info!(
                            "Withdraw eeTH from weETH: -eETH {}",
                            ev.shares_value
                        );

                        deltas.push(BalanceDelta {
                            ord: log.ordinal(),
                            tx: Some(log.receipt.transaction.into()),
                            token: EETH_ADDRESS.to_vec(),
                            delta: ev
                                .shares_value
                                .neg()
                                .to_signed_bytes_be(),
                            component_id: WEETH_ADDRESS.to_vec(),
                        });
                    }
                }
            }
            // weETH transfer:
            // Mint: Contract Balance becomes += eETH, += weETH, Burn: Contract Balance becomes -=
            // eETH, -= weETH
            else if let Some(ev) = abi::erc20::events::Transfer::match_and_decode(log.log) {
                if store
                    .get_last(format!("pool:{}", weeth_hex))
                    .is_some()
                {
                    // Mint Shares: Contract Balance += weETH
                    if ev.from == ADDRESS_ZERO {
                        substreams::log::info!("Mint weeTH: +weETH {}", ev.value);

                        deltas.push(BalanceDelta {
                            ord: log.ordinal(),
                            tx: Some(log.receipt.transaction.into()),
                            token: WEETH_ADDRESS.to_vec(),
                            delta: ev.value.to_signed_bytes_be(),
                            component_id: WEETH_ADDRESS.to_vec(),
                        });
                    }
                    // Burn Shares: Contract Balance -= weETH
                    else if ev.to == ADDRESS_ZERO {
                        substreams::log::info!("Burn weeTH: -eETH {}", ev.value);

                        deltas.push(BalanceDelta {
                            ord: log.ordinal(),
                            tx: Some(log.receipt.transaction.into()),
                            token: WEETH_ADDRESS.to_vec(),
                            delta: ev.value.neg().to_signed_bytes_be(),
                            component_id: WEETH_ADDRESS.to_vec(),
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
            balances.values().for_each(|bc| {
                builder.add_balance_change(bc);
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

    // Process all `transaction_changes` for final output in the `BlockChanges`,
    //  sorted by transaction index (the key).

    let block_changes = BlockChanges {
        block: Some((&block).into()),
        changes: transaction_changes
            .drain()
            .sorted_unstable_by_key(|(index, _)| *index)
            .filter_map(|(_, builder)| builder.build())
            .collect::<Vec<_>>(),
    };

    for change in &block_changes.changes {
        substreams::log::info!("🚨 Balance changes {:?}", change.balance_changes);
        substreams::log::info!("🚨 Component changes {:?}", change.component_changes);
    }
    Ok(block_changes)
}

fn store_component(store: &StoreSetProto<ProtocolComponent>, component: &ProtocolComponent) {
    store.set(1, format!("pool:{}", component.id), component);
}
