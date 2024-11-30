use anyhow::{Context, Result};
use itertools::Itertools;
use serde::Deserialize;
use std::collections::HashMap;
use substreams::{
    hex,
    pb::substreams::StoreDeltas,
    store::{
        StoreAdd, StoreAddBigInt, StoreAddInt64, StoreGet, StoreGetInt64, StoreGetString, StoreNew,
    },
};
use substreams_ethereum::pb::eth;
use tycho_substreams::{
    balances::{aggregate_balances_changes, extract_balance_deltas_from_tx},
    contract::extract_contract_changes_builder,
    prelude::*,
};

#[derive(Debug, Deserialize, PartialEq)]
struct ComponentParams {
    vault_address: String,
    dai_address: String,
}

pub const MASTER_VAULT_ADDRESS: [u8; 20] = hex!("649765821D9f64198c905eC0B2B037a4a52Bc373");
pub const ETH_ADDRESS: [u8; 20] = hex!("eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee");

#[substreams::handlers::map]
pub fn map_components(
    params: String,
    block: eth::v2::Block,
) -> Result<BlockTransactionProtocolComponents, anyhow::Error> {
    let comp = parse_params(&params);

    if let Some(pool) = comp.unwrap().get("bancor_component") {
        let vault_address = hex::decode(pool.vault_address.clone()).unwrap();
        let dai_address = hex::decode(pool.dai_address.clone()).unwrap();
        Ok(BlockTransactionProtocolComponents {
            tx_components: block
                .transactions()
                .filter_map(|tx| {
                    let components = tx
                        .calls()
                        .filter(|call| !call.call.state_reverted)
                        .filter_map(|call| {
                            // address doesn't exist before contract deployment, hence the first tx
                            // with a log.address = vault_address is the
                            // deployment tx
                            if is_deployment_call(call.call, &vault_address) {
                                Some(
                                    ProtocolComponent::at_contract(&vault_address, &tx.into())
                                        .with_tokens(&[
                                            dai_address.as_slice(),
                                            ETH_ADDRESS.as_slice(),
                                        ])
                                        .as_swap_type(
                                            "bancor_master_vault",
                                            ImplementationType::Vm,
                                        ),
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
    } else {
        Err(anyhow::Error::msg("Params mismatch or not found"))
    }
}

/// Simply stores the `ProtocolComponent`s with the pool address as the key and the pool id as value
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
pub fn map_relative_balances(
    block: eth::v2::Block,
    store: StoreGetInt64,
) -> Result<BlockBalanceDeltas, anyhow::Error> {
    let balance_deltas = block
        .transactions()
        .flat_map(|tx| {
            let mut deltas = Vec::new();

            let erc20_deltas =
                extract_balance_deltas_from_tx(tx, |token_address, transfer_or_receiver| {
                    // token refers to a component being tracked AND tx destination is the master
                    // vault
                    store
                        .get_last(format!("pool:0x{0}", hex::encode(token_address)))
                        .is_some() &&
                        transfer_or_receiver == MASTER_VAULT_ADDRESS
                });

            deltas.extend(erc20_deltas);

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
                .get_last(format!("pool:0x{0}", hex::encode(addr)))
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

fn is_deployment_call(call: &eth::v2::Call, vault_address: &[u8]) -> bool {
    call.account_creations
        .iter()
        .any(|ac| ac.account.as_slice() == vault_address)
}

fn parse_params(params: &str) -> Result<HashMap<String, ComponentParams>, anyhow::Error> {
    let pools: HashMap<String, ComponentParams> = params
        .split(",")
        .map(|param| {
            let pool: ComponentParams = serde_qs::from_str(param)
                .with_context(|| format!("Failed to parse query params: {0}", param))?;
            Ok(("bancor_component".into(), pool))
        })
        .collect::<Result<HashMap<_, _>>>()
        .with_context(|| "Failed to parse all query params")?;
    Ok(pools)
}
