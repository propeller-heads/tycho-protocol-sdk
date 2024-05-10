use crate::abi;
use anyhow::Result;
use itertools::Itertools;
use std::collections::HashMap;
use substreams::{
    hex,
    pb::substreams::StoreDeltas,
    store::{StoreAdd, StoreAddBigInt, StoreAddInt64, StoreGet, StoreGetInt64, StoreNew},
};
use substreams_ethereum::{pb::eth, Event, Function};
use tycho_substreams::{
    balances::aggregate_balances_changes, contract::extract_contract_changes, prelude::*,
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

/// Since the `PoolBalanceChanged` and `Swap` events administer only deltas, we need to leverage a
/// map and a  store to be able to tally up final balances for tokens in a pool.
#[substreams::handlers::map]
pub fn map_relative_balances(
    block: eth::v2::Block,
    store: StoreGetInt64,
) -> Result<BlockBalanceDeltas, anyhow::Error> {
    let balance_deltas = block
        .logs()
        .filter(|log| log.address() == RELAYER_PROXY)
        .flat_map(|vault_log| {
            let mut deltas = Vec::new();

            if store
                .get_last(format!("pool:{}", hex::encode(RELAYER_PROXY)))
                .is_none()
            {
                return vec![]
            }

            if let Some(ev) = abi::relayer::events::Sell::match_and_decode(vault_log.log) {
                deltas.push(BalanceDelta {
                    ord: vault_log.ordinal(),
                    tx: Some(vault_log.receipt.transaction.into()),
                    token: ev.token_in.to_vec(),
                    delta: ev.amount_in.to_signed_bytes_be(),
                    component_id: RELAYER_PROXY.to_vec(),
                });
                deltas.push(BalanceDelta {
                    ord: vault_log.ordinal(),
                    tx: Some(vault_log.receipt.transaction.into()),
                    token: ev.token_out.to_vec(),
                    delta: ev.amount_out.neg().to_signed_bytes_be(),
                    component_id: RELAYER_PROXY.to_vec(),
                });
            } else if let Some(ev) = abi::relayer::events::Buy::match_and_decode(vault_log.log) {
                deltas.push(BalanceDelta {
                    ord: vault_log.ordinal(),
                    tx: Some(vault_log.receipt.transaction.into()),
                    token: ev.token_in.to_vec(),
                    delta: ev.amount_in.to_signed_bytes_be(),
                    component_id: RELAYER_PROXY.to_vec(),
                });
                deltas.push(BalanceDelta {
                    ord: vault_log.ordinal(),
                    tx: Some(vault_log.receipt.transaction.into()),
                    token: ev.token_out.to_vec(),
                    delta: ev.amount_out.neg().to_signed_bytes_be(),
                    component_id: RELAYER_PROXY.to_vec(),
                });
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
