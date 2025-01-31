use crate::abi;
use anyhow::Result;
use itertools::Itertools;
use std::collections::HashMap;
use substreams::{
    hex,
    pb::substreams::StoreDeltas,
    store::{StoreAddBigInt, StoreGet, StoreGetString, StoreNew, StoreSet as _, StoreSetString},
};
use substreams_ethereum::{pb::eth, Event, Function};
use tycho_substreams::{
    balances::aggregate_balances_changes, contract::extract_contract_changes_builder, prelude::*,
};

const RELAYER_PROXY: &[u8] = &hex!("d17b3c9784510E33cD5B87b490E79253BcD81e2E");
const RELAYER_TX_HASH: &[u8] =
    &hex!("acada5a9c928026fb1080a569579cc8073b4e916c9a3b7a56a7dd3c5cfb053fc");

const FACTORY: &[u8] = &hex!("C480b33eE5229DE3FbDFAD1D2DCD3F3BAD0C56c6");

/// Gather components by indexing `PairCreated` events from the factory address.
#[substreams::handlers::map]
pub fn map_components(block: eth::v2::Block) -> Result<BlockTransactionProtocolComponents> {
    Ok(BlockTransactionProtocolComponents {
        tx_components: block
            .transactions()
            .flat_map(|tx| {
                let mut components = tx
                    .logs_with_calls()
                    .filter_map(|(log, call)| {
                        let event = abi::factory::events::PairCreated::match_and_decode(log)?;
                        let create_pair =
                            abi::factory::functions::CreatePair::match_and_decode(call)?;

                        let uniswap_pair = abi::oracle::functions::UniswapPair {}
                            .call(create_pair.oracle.clone())
                            .expect(&format!(
                                "Uniswap Pair address not found from oracle {} RPC",
                                hex::encode(&create_pair.oracle),
                            ));

                        Some(TransactionProtocolComponents {
                            components: vec![ProtocolComponent::at_contract(
                                &event.pair,
                                &tx.into(),
                            )
                            .with_tokens(&[event.token0, event.token1])
                            .with_attributes(&[("pool_type", "TwapPair".as_bytes())])
                            .with_attributes(&[("oracle", create_pair.oracle)])
                            .with_attributes(&[("delay_contract", create_pair.trader.clone())])
                            .with_attributes(&[("uniswap_pair", uniswap_pair)])
                            .with_contracts(&[event.pair, create_pair.trader])
                            .as_swap_type("TwapPair", ImplementationType::Vm)],
                            tx: Some(tx.into()),
                        })
                    })
                    .collect::<Vec<_>>();
                if tx.hash == RELAYER_TX_HASH {
                    components.push(TransactionProtocolComponents {
                        components: vec![ProtocolComponent::at_contract(RELAYER_PROXY, &tx.into())
                            .with_attributes(&[("proxy", hex::encode(RELAYER_PROXY))])
                            .with_attributes(&[("proxy_standard", "EIP1967")])
                            .with_contracts(&[FACTORY])
                            .as_swap_type("RelayerProxy", ImplementationType::Vm)],
                        tx: Some(tx.into()),
                    });
                }
                components
            })
            .collect::<Vec<_>>(),
    })
}

/// Get result `map_components` and stores the created `ProtocolComponent`s with the pool id as the
/// key and tokens as the value
#[substreams::handlers::store]
pub fn store_component_tokens(map: BlockChanges, store: StoreSetString) {
    map.changes
        .iter()
        .flat_map(|tx_changes| &tx_changes.component_changes)
        .for_each(|component| {
            store.set(
                0,
                format!("pool:{0}", component.id),
                &component
                    .tokens
                    .iter()
                    .map(hex::encode)
                    .join(":"),
            );
        });
}

/// Since the `PoolBalanceChanged` and `Swap` events administer only deltas, we need to leverage a
///  map and a store to be able to tally up final balances for tokens in a pool.
#[substreams::handlers::map]
pub fn map_relative_balances(
    block: eth::v2::Block,
    store: StoreGetString,
) -> Result<BlockBalanceDeltas, anyhow::Error> {
    let balance_deltas = block
        .logs()
        .filter_map(|log| {
            let mut deltas = Vec::new();
            let pool_tokens = store.get_first(format!("pool:{0}", hex::encode(log.address())))?;
            let tokens = pool_tokens
                .split(':')
                .map(|token| hex::decode(token).unwrap())
                .collect::<Vec<_>>();

            if let Some(ev) = abi::relayer::events::Sell::match_and_decode(log.log) {
                deltas.push(BalanceDelta {
                    ord: log.ordinal(),
                    tx: Some(log.receipt.transaction.into()),
                    token: ev.token_in.to_vec(),
                    delta: ev.amount_in.to_signed_bytes_be(),
                    component_id: RELAYER_PROXY.to_vec(),
                });
                deltas.push(BalanceDelta {
                    ord: log.ordinal(),
                    tx: Some(log.receipt.transaction.into()),
                    token: ev.token_out.to_vec(),
                    delta: ev.amount_out.neg().to_signed_bytes_be(),
                    component_id: RELAYER_PROXY.to_vec(),
                });
            } else if let Some(ev) = abi::relayer::events::Buy::match_and_decode(log.log) {
                deltas.push(BalanceDelta {
                    ord: log.ordinal(),
                    tx: Some(log.receipt.transaction.into()),
                    token: ev.token_in.to_vec(),
                    delta: ev.amount_in.to_signed_bytes_be(),
                    component_id: RELAYER_PROXY.to_vec(),
                });
                deltas.push(BalanceDelta {
                    ord: log.ordinal(),
                    tx: Some(log.receipt.transaction.into()),
                    token: ev.token_out.to_vec(),
                    delta: ev.amount_out.neg().to_signed_bytes_be(),
                    component_id: RELAYER_PROXY.to_vec(),
                });
            } else if let Some(ev) = abi::relayer::events::Buy::match_and_decode(log.log) {
                deltas.push(BalanceDelta {
                    ord: log.ordinal(),
                    tx: Some(log.receipt.transaction.into()),
                    token: ev.token_in.to_vec(),
                    delta: ev.amount_in.to_signed_bytes_be(),
                    component_id: RELAYER_PROXY.to_vec(),
                });
                deltas.push(BalanceDelta {
                    ord: log.ordinal(),
                    tx: Some(log.receipt.transaction.into()),
                    token: ev.token_out.to_vec(),
                    delta: ev.amount_out.neg().to_signed_bytes_be(),
                    component_id: RELAYER_PROXY.to_vec(),
                });
            } else if let Some(swap) = abi::pair::events::Swap::match_and_decode(log.log) {
                if swap.amount1_in.is_zero() {
                    deltas.push(BalanceDelta {
                        ord: log.ordinal(),
                        tx: Some(log.receipt.transaction.into()),
                        token: tokens[0].clone(),
                        delta: swap.amount0_in.to_signed_bytes_be(),
                        component_id: log.address().to_vec(),
                    });
                    deltas.push(BalanceDelta {
                        ord: log.ordinal(),
                        tx: Some(log.receipt.transaction.into()),
                        token: tokens[1].clone(),
                        delta: swap
                            .amount1_out
                            .neg()
                            .to_signed_bytes_be(),
                        component_id: log.address().to_vec(),
                    });
                } else if swap.amount0_in.is_zero() {
                    deltas.push(BalanceDelta {
                        ord: log.ordinal(),
                        tx: Some(log.receipt.transaction.into()),
                        token: tokens[1].clone(),
                        delta: swap.amount0_in.to_signed_bytes_be(),
                        component_id: log.address().to_vec(),
                    });
                    deltas.push(BalanceDelta {
                        ord: log.ordinal(),
                        tx: Some(log.receipt.transaction.into()),
                        token: tokens[0].clone(),
                        delta: swap
                            .amount1_out
                            .neg()
                            .to_signed_bytes_be(),
                        component_id: log.address().to_vec(),
                    });
                }
            }

            Some(deltas)
        })
        .flatten()
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
/// Every contract change is grouped by transaction index via the `transaction_contract_changes`
///  map. Each block of code will extend the `TransactionChanges` struct with the
///  cooresponding changes (balance, component, contract), inserting a new one if it doesn't exist.
///  At the very end, the map can easily be sorted by index to ensure the final
/// `BlockChanges` is ordered by transactions properly.
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
                        attributes: component.static_att.clone(),
                    };
                    builder.add_entity_change(&entity_change)
                });
        });

    // Balance changes are gathered by the `StoreDelta` based on `PoolBalanceChanged` creating
    //  `BlockBalanceDeltas`. We essentially just process the changes that occurred to the `store`
    // this block. Then, these balance changes are merged onto the existing map of tx contract
    // changes, inserting a new one if it doesn't exist.
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

    // Extract and insert any storage changes that happened for any of the components.
    extract_contract_changes_builder(
        &block,
        |addr| {
            components_store
                .get_last(format!("pool:0x{0}", hex::encode(addr)))
                .is_some()
                || addr.eq(RELAYER_PROXY)
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
