use crate::erc20_transfer::decode_erc20_transfer;
use anyhow::Result;
use hex_literal::hex;
use itertools::Itertools;
use num_bigint::BigInt;
use std::{collections::HashMap, vec};
use substreams::{
    log,
    pb::substreams::StoreDeltas,
    scalar::{self, BigInt as ScalarBigInt},
    store::{
        StoreAdd, StoreAddBigInt, StoreAddInt64, StoreGet, StoreGetInt64, StoreGetString, StoreNew,
    },
};
use substreams_database_change::{change::AsString, tables::ToDatabaseValue};
use substreams_ethereum::{
    pb::eth::{self},
    Event,
};
use tycho_substreams::{
    balances::aggregate_balances_changes, contract::extract_contract_changes_builder, prelude::*,
};

// pub const ETH_ADDRESS: [u8; 20] = [238u8; 20]; // 0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee

#[substreams::handlers::map]
pub fn map_components(
    params: String,
    block: eth::v2::Block,
) -> Result<BlockTransactionProtocolComponents, anyhow::Error> {
    let vault_address = hex::decode(params).unwrap();
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
                            let ignazio =
                                ProtocolComponent::at_contract(&vault_address, &tx.into())
                                    .as_swap_type("bancor_master_vault", ImplementationType::Vm);

                            let mut cur_tokens = Vec::new();

                            ignazio.tokens.iter().for_each(|token| {
                                if !cur_tokens.contains(token) {
                                    cur_tokens.push(token.clone());
                                }
                            });

                            Some(
                                ProtocolComponent::at_contract(&vault_address, &tx.into())
                                    .as_swap_type("bancor_master_vault", ImplementationType::Vm),
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

/// Simply stores the `ProtocolComponent`s with the pool address as the key and the pool id as value
#[substreams::handlers::store]
pub fn store_components(map: BlockTransactionProtocolComponents, store: StoreAddInt64) {
    store.add_many(
        0,
        &map.tx_components
            .iter()
            .flat_map(|tx_components| &tx_components.components)
            .map(|component| format!("vault:{}", &component.id))
            .collect::<Vec<_>>(),
        1,
    );
}

#[substreams::handlers::map]
pub fn map_relative_balances(
    block: eth::v2::Block,
    store: StoreGetInt64,
) -> Result<BlockBalanceDeltas, anyhow::Error> {
    let balance_deltas = block
        .transactions()
        .flat_map(|tx| {
            let mut deltas = Vec::new();

            let mut tx_value: ScalarBigInt = ScalarBigInt::zero();

            if let Some(value) = &tx.value {
                tx_value = ScalarBigInt::from_signed_bytes_be(&value.bytes);
            }

            if tx_value.gt(&ScalarBigInt::zero()) {
                let tx_from = format!("0x{}", hex::encode(&tx.from));
                let tx_to = format!("0x{}", hex::encode(&tx.to));
                let vault_address = if store
                    .get_last(format!("vault:{}", tx_from))
                    .is_some()
                {
                    tx_from.clone()
                } else if store
                    .get_last(format!("vault:{}", tx_to))
                    .is_some()
                {
                    tx_to.clone()
                } else {
                    "".to_string()
                };

                let vault_exists: bool = !vault_address.is_empty();

                if vault_exists {
                    let address_hex = format!("0x{}", vault_address);
                    if &tx_from == &vault_address {
                        deltas.push(BalanceDelta {
                            ord: tx.begin_ordinal,
                            tx: Some(tx.into()),
                            token: vec![0; 20],
                            delta: tx_value.neg().to_signed_bytes_be(),
                            component_id: address_hex.as_bytes().to_vec(),
                        });
                    } else if &tx_to == &vault_address {
                        deltas.push(BalanceDelta {
                            ord: tx.begin_ordinal,
                            tx: Some(tx.into()),
                            token: vec![0; 20],
                            delta: tx_value.to_signed_bytes_be(),
                            component_id: address_hex.as_bytes().to_vec(),
                        });
                    }
                }
            }

            // Track ERC20 transfers
            tx.logs_with_calls()
                .filter_map(|(log, _call)| {
                    if let Some(transfer) = decode_erc20_transfer(&log) {
                        let tx_from = hex::encode(&transfer.from);
                        let tx_to = hex::encode(&transfer.from);
                        let vault_address = if store
                            .get_last(format!("vault:{}", tx_from))
                            .is_some()
                        {
                            tx_from.clone()
                        } else if store
                            .get_last(format!("vault:{}", tx_to))
                            .is_some()
                        {
                            tx_to.clone()
                        } else {
                            "".to_string()
                        };

                        let vault_exists: bool = !vault_address.is_empty();

                        if vault_exists {
                            let address_hex = format!("0x{}", vault_address);

                            if tx_from == vault_address {
                                Some(BalanceDelta {
                                    ord: log.ordinal,
                                    tx: Some(tx.into()),
                                    token: log.address.to_vec(),
                                    delta: transfer
                                        .value
                                        .neg()
                                        .to_signed_bytes_be(),
                                    component_id: address_hex.as_bytes().to_vec(),
                                })
                            } else if tx_to == vault_address {
                                Some(BalanceDelta {
                                    ord: log.ordinal,
                                    tx: Some(tx.into()),
                                    token: log.address.to_vec(),
                                    delta: transfer.value.to_signed_bytes_be(),
                                    component_id: address_hex.as_bytes().to_vec(),
                                })
                            } else {
                                Some(BalanceDelta {
                                    ord: log.ordinal,
                                    tx: Some(tx.into()),
                                    token: log.address.to_vec(),
                                    delta: BigInt::from(123).to_signed_bytes_be(),
                                    component_id: address_hex.as_bytes().to_vec(),
                                })
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .for_each(|delta| deltas.push(delta));

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
///  corresponding changes (balance, component, contract), inserting a new one if it doesn't exist.
///  At the very end, the map can easily be sorted by index to ensure the final
/// `BlockContractChanges`  is ordered by transactions properly.
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
                .get_last(format!("0x{0}", hex::encode(addr)))
                .is_some()
        },
        &mut transaction_changes,
    );

    // Process all `transaction_contract_changes` for final output in the `BlockContractChanges`,
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
