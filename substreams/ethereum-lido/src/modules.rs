use crate::abi::{
    lido::events::{Submitted, TokenRebased},
    withdrawal_queue::events::WithdrawalClaimed,
};
use anyhow::Result;
use itertools::Itertools;
use std::collections::HashMap;
use substreams::{
    hex,
    pb::substreams::StoreDeltas,
    scalar::BigInt,
    store::{StoreAddBigInt, StoreGet, StoreGetString, StoreNew, StoreSet, StoreSetString},
};
use substreams_ethereum::{
    pb::eth::{self},
    Event,
};
use tycho_substreams::{
    balances::{aggregate_balances_changes, extract_balance_deltas_from_tx},
    contract::extract_contract_changes_builder,
    prelude::*,
};

const ZERO_ADDRESS: [u8; 20] = hex!("0000000000000000000000000000000000000000");
const WITHDRAWAL_QUEUE_ADDRESS: [u8; 20] = hex!("889edC2eDab5f40e902b864aD4d7AdE8E412F9B1");
const WSTETH_ADDRESS: [u8; 20] = hex!("7f39C581F595B53c5cb19bD0b3f8dA6c935E2Ca0"); //wstETH
const LIDO_STETH_ADDRESS: [u8; 20] = hex!("ae7ab96520DE3A18E5e111B5EaAb095312D7fE84"); //stETH
const ETH_ADDRESS: [u8; 20] = hex!("0000000000000000000000000000000000000000"); //ETH
const LIDO_STETH_CREATION_TX: [u8; 32] =
    hex!("3feabd79e8549ad68d1827c074fa7123815c80206498946293d5373a160fd866"); //stETH creation tx
const WSTETH_CREATION_TX: [u8; 32] =
    hex!("af2c1a501d2b290ef1e84ddcfc7beb3406f8ece2c46dee14e212e8233654ff05"); //wstETH creation tx

#[substreams::handlers::map]
pub fn map_components(block: eth::v2::Block) -> Result<BlockTransactionProtocolComponents> {
    // We store these as a hashmap by tx hash since we need to agg by tx hash later
    Ok(BlockTransactionProtocolComponents {
        tx_components: block
            .transactions()
            .filter_map(|tx| {
                let mut components = vec![];
                if tx.hash == WSTETH_CREATION_TX {
                    components.extend([ProtocolComponent::at_contract(WSTETH_ADDRESS.as_slice())
                        .with_tokens(&[LIDO_STETH_ADDRESS, WSTETH_ADDRESS])
                        .with_attributes(&[("vault_type", "wsteth".as_bytes())])
                        .as_swap_type("lido_vault", ImplementationType::Vm)]);
                }
                if tx.hash == LIDO_STETH_CREATION_TX {
                    components.extend([ProtocolComponent::at_contract(
                        LIDO_STETH_ADDRESS.as_slice(),
                    )
                    .with_tokens(&[ETH_ADDRESS, LIDO_STETH_ADDRESS])
                    .with_attributes(&[("vault_type", "steth".as_bytes())])
                    .as_swap_type("lido_vault", ImplementationType::Vm)]);
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

/// Simply stores the `ProtocolComponent`s with the pool address as the key and the pool id as value
#[substreams::handlers::store]
pub fn store_components(map: BlockTransactionProtocolComponents, store: StoreSetString) {
    map.tx_components
        .into_iter()
        .for_each(|tx_pc| {
            tx_pc
                .components
                .into_iter()
                .for_each(|pc| store.set(0, format!("pool:{0}", &pc.id[..42]), &pc.id))
        });
}

#[substreams::handlers::map]
pub fn map_relative_balances(
    block: eth::v2::Block,
    components_store: StoreGetString,
) -> Result<BlockBalanceDeltas, anyhow::Error> {
    let mut balance_deltas = Vec::new();

    // parse non reverted calls
    block.transactions().for_each(|tx| {
        let wst_component_id = format!("0x{}", hex::encode(WSTETH_ADDRESS));
        // the following 2 lines will extract all mint and burn wstETH balance deltas, mint will
        // have a positive sign (corresponding to a wrap operation) and burn will have a
        // negative sign (corresponding to an unwrap operation)
        let mut wsteth_balance_deltas =
            extract_balance_deltas_from_tx(tx, |log_addr, from_or_to_addr| {
                log_addr == WSTETH_ADDRESS && from_or_to_addr == ZERO_ADDRESS
            });
        // change the sign due to extract balance logic mint (from = address(0) should have a
        // positive sign) opposite holds for burn (to = address(0))
        wsteth_balance_deltas
            .iter_mut()
            .for_each(|bd| {
                bd.delta = BigInt::from_signed_bytes_be(bd.delta.as_slice())
                    .neg()
                    .to_signed_bytes_be();
                bd.component_id = wst_component_id.as_bytes().to_vec();
            });
        // match the corresponding wstETH variation with a equal variation of stETH
        let steth_balance_deltas = wsteth_balance_deltas
            .iter()
            .map(|balance_delta| BalanceDelta {
                token: LIDO_STETH_ADDRESS.to_vec(),
                ..balance_delta.clone()
            })
            .collect::<Vec<_>>();

        if components_store
            .get_last(format!("pool:{}", &wst_component_id[..42]))
            .is_some()
        {
            balance_deltas.extend_from_slice(steth_balance_deltas.as_slice());
            balance_deltas.extend_from_slice(wsteth_balance_deltas.as_slice());
        }

        tx.logs_with_calls()
            .for_each(|(log, _)| {
                if log.address == LIDO_STETH_ADDRESS {
                    if let Some(TokenRebased { pre_total_ether, post_total_ether, .. }) =
                        TokenRebased::match_and_decode(log)
                    {
                        let component_id = format!("0x{}", hex::encode(LIDO_STETH_ADDRESS));
                        if components_store
                            .get_last(format!("pool:{}", &component_id[..42]))
                            .is_some()
                        {
                            let delta_eth = post_total_ether - pre_total_ether;
                            balance_deltas.extend_from_slice(&[
                                BalanceDelta {
                                    ord: log.ordinal,
                                    tx: Some(tx.into()),
                                    token: LIDO_STETH_ADDRESS.to_vec(),
                                    delta: delta_eth.to_signed_bytes_be(),
                                    component_id: component_id.as_bytes().to_vec(),
                                },
                                BalanceDelta {
                                    ord: log.ordinal,
                                    tx: Some(tx.into()),
                                    token: ETH_ADDRESS.to_vec(),
                                    delta: delta_eth.to_signed_bytes_be(),
                                    component_id: component_id.as_bytes().to_vec(),
                                },
                            ])
                        }
                    }
                    if let Some(Submitted { amount, .. }) = Submitted::match_and_decode(log) {
                        let component_id = format!("0x{}", hex::encode(LIDO_STETH_ADDRESS));
                        if components_store
                            .get_last(format!("pool:{}", &component_id[..42]))
                            .is_some()
                        {
                            balance_deltas.extend_from_slice(&[
                                BalanceDelta {
                                    ord: log.ordinal,
                                    tx: Some(tx.into()),
                                    token: LIDO_STETH_ADDRESS.to_vec(),
                                    delta: amount.to_signed_bytes_be(),
                                    component_id: component_id.as_bytes().to_vec(),
                                },
                                BalanceDelta {
                                    ord: log.ordinal,
                                    tx: Some(tx.into()),
                                    token: ETH_ADDRESS.to_vec(),
                                    delta: amount.to_signed_bytes_be(),
                                    component_id: component_id.as_bytes().to_vec(),
                                },
                            ]);
                        }
                    }
                }
                if log.address == WITHDRAWAL_QUEUE_ADDRESS {
                    if let Some(WithdrawalClaimed { amount_of_eth, .. }) =
                        WithdrawalClaimed::match_and_decode(log)
                    {
                        let component_id = format!("0x{}", hex::encode(LIDO_STETH_ADDRESS));
                        if components_store
                            .get_last(format!("pool:{}", &component_id[..42]))
                            .is_some()
                        {
                            balance_deltas.extend_from_slice(&[
                                BalanceDelta {
                                    ord: log.ordinal,
                                    tx: Some(tx.into()),
                                    token: LIDO_STETH_ADDRESS.to_vec(),
                                    delta: amount_of_eth.neg().to_signed_bytes_be(),
                                    component_id: component_id.as_bytes().to_vec(),
                                },
                                BalanceDelta {
                                    ord: log.ordinal,
                                    tx: Some(tx.into()),
                                    token: ETH_ADDRESS.to_vec(),
                                    delta: amount_of_eth.neg().to_signed_bytes_be(),
                                    component_id: component_id.as_bytes().to_vec(),
                                },
                            ]);
                        }
                    }
                }
            });
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
    components_store: StoreGetString,
    balance_store: StoreDeltas, // Note, this map module is using the `deltas` mode for the store.
) -> Result<BlockChanges> {
    // We merge contract changes by transaction (identified by transaction index) making it easy to
    //  sort them at the very end.
    let mut transaction_changes: HashMap<_, TransactionChangesBuilder> = HashMap::new();

    let default_attributes = vec![
        Attribute {
            // proxy
            name: "stateless_contract_addr_0".into(),
            value: address_to_utf8_vec(&hex!("17144556fd3424EDC8Fc8A4C940B2D04936d17eb")),
            change: ChangeType::Creation.into(),
        },
        Attribute {
            name: "stateless_contract_addr_1".to_string(),
            value: address_to_utf8_vec(&hex!("b8ffc3cd6e7cf5a098a1c92f48009765b24088dc")),
            change: ChangeType::Creation.into(),
        },
    ];
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
                        attributes: default_attributes.clone(),
                    };
                    builder.add_entity_change(&entity_change)
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

/// Converts address bytes into a Vec<u8> containing a leading `0x`.
fn address_to_utf8_vec(address: &[u8; 20]) -> Vec<u8> {
    format!("0x{}", hex::encode(address)).into_bytes()
}
