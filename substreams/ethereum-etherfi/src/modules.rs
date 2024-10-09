use crate::{abi, consts};
use anyhow::Result;
use consts::{EETH_ADDRESS, ETH_ADDRESS, LIQUIDITY_POOL_ADDRESS, WEETH_ADDRESS};
use itertools::Itertools;
use std::collections::HashMap;
use substreams::{
    hex,
    pb::substreams::StoreDeltas,
    store::{
        StoreAddBigInt, StoreGet, StoreGetString, StoreGetProto, StoreNew, StoreSet, StoreSetProto,
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
    Ok(BlockTransactionProtocolComponents {
        tx_components: block
            .transactions()
            .filter_map(|tx| {
                let components = tx
                    .logs_with_calls()
                    .filter(|(_, call)| !call.call.state_reverted)
                    .filter_map(|(log, _)| {
                        if let Some(_ev) =
                            abi::pool_contract::events::Initialized::match_and_decode(log)
                        {
                            Some(
                                ProtocolComponent::at_contract(&LIQUIDITY_POOL_ADDRESS, &tx.into())
                                    .with_tokens(&[EETH_ADDRESS, WEETH_ADDRESS, ETH_ADDRESS])
                                    .as_swap_type("etherfi_pool", ImplementationType::Vm),
                            )
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
    let balance_deltas = block
        .logs()
        .flat_map(|log| {
            let mut deltas = Vec::new();
            let address_bytes_be = log.address();
            let address_hex = format!("0x{}", hex::encode(address_bytes_be));

            if let Some(ev) = abi::pool_contract::events::Deposit::match_and_decode(log.log) {

                if store
                    .get_last(format!("pool:{}", address_hex))
                    .is_some()
                {
                    // Shares are minted, therefore not held by the contract; Contract balance just becomes += ETH
                    substreams::log::info!("Deposit: -ETH {}", ev.amount);

                    deltas.push(BalanceDelta {
                        ord: log.ordinal(),
                        tx: Some(log.receipt.transaction.into()),
                        token: ETH_ADDRESS.to_vec(),
                        delta: ev.amount.to_signed_bytes_be(),
                        component_id: address_bytes_be.to_vec(),
                    });
                }
            } else if let Some(ev) = abi::pool_contract::events::Withdraw::match_and_decode(log.log)
            {
                let address_bytes_be = log.address();
                let address_hex = format!("0x{}", hex::encode(address_bytes_be));

                if store
                    .get_last(format!("pool:{}", address_hex))
                    .is_some()
                {
                    // Shares are minted, therefore not held by the contract; Contract balance just becomes -= ETH
                    substreams::log::info!("Withdraw: +ETH {}", ev.amount);

                    deltas.push(BalanceDelta {
                        ord: log.ordinal(),
                        tx: Some(log.receipt.transaction.into()),
                        token: ETH_ADDRESS.to_vec(),
                        delta: ev.amount.neg().to_signed_bytes_be(),
                        component_id: address_bytes_be.to_vec(),
                    });
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
fn maybe_get_pool_tokens(
    store: &StoreGetProto<ProtocolComponent>,
    component_id: &str,
) -> Option<(Vec<u8>, Vec<u8>)> {
    store
        .get_last(format!("pool:{}", component_id))
        .map(|component| (component.tokens[0].to_vec(), component.tokens[1].to_vec()))
}

fn address_to_hex(address: &[u8]) -> String {
    format!("0x{}", hex::encode(address))
}

fn string_to_bytes(string: &str) -> Vec<u8> {
    string.as_bytes().to_vec()
}

fn store_component(store: &StoreSetProto<ProtocolComponent>, component: &ProtocolComponent) {
    store.set(1, format!("pool:{}", component.id), component);
}
