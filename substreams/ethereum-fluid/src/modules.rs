use crate::abi::{
    dex_t1_admin::events::{LogPauseSwapAndArbitrage, LogUnpauseSwapAndArbitrage},
    factory::events::LogDexDeployed,
};
use anyhow::{Ok, Result};
use ethabi::ethereum_types::Address;
use itertools::Itertools;
use std::{collections::HashMap, str::FromStr};
use substreams::{hex, prelude::*};
use substreams_ethereum::{pb::eth, Event};
use substreams_helper::{common::HasAddresser, event_handler::EventHandler, hex::Hexable};
use tycho_substreams::{
    block_storage::get_block_storage_changes,
    contract::extract_contract_changes_builder,
    entrypoint::create_entrypoint,
    prelude::{entry_point_params::TraceData, *},
};

pub const LIQUIDITY_CONTRACT_ADDRESS: &[u8] = &hex!("52Aa899454998Be5b000Ad077a46Bbe360F4e497");
// TODO: make this configurable for multichain support
pub const DEX_RESERVES_RESOLVER: &[u8] = &hex!("b387f9C2092cF7c4943F97842887eBff7AE96EB3");
pub const DEX_RESERVES_RESOLVER_2: &[u8] = &hex!("C93876C0EEd99645DD53937b25433e311881A27C");
/// Find and create all relevant protocol components
///
/// This method maps over blocks and instantiates ProtocolComponents with a unique ids
/// as well as all necessary metadata for routing and encoding.
#[substreams::handlers::map]
pub fn map_dex_deployed(
    params: String,
    block: eth::v2::Block,
) -> Result<BlockTransactionProtocolComponents> {
    let mut new_dexes = vec![];
    let factory_address = params.as_str();

    get_new_dexes(&block, &mut new_dexes, factory_address);

    Ok(BlockTransactionProtocolComponents { tx_components: new_dexes })
}

fn get_new_dexes(
    block: &eth::v2::Block,
    new_dexes: &mut Vec<TransactionProtocolComponents>,
    factory_address: &str,
) {
    // Extract new dex pools from LogDexDeployed events
    let mut on_dex_deployed = |event: LogDexDeployed,
                               tx: &eth::v2::TransactionTrace,
                               _log: &eth::v2::Log| {
        let tycho_tx: Transaction = tx.into();
        let constants_view_rpc_call = crate::abi::dex_implementation::functions::ConstantsView {};
        let constants_view = constants_view_rpc_call
            .call(event.dex.clone())
            .unwrap();
        let implementations = constants_view.3;
        let resolver_address = get_reserves_resolver(block.number);

        new_dexes.push(TransactionProtocolComponents {
            tx: Some(tycho_tx.clone()),
            components: vec![ProtocolComponent {
                id: event.dex.to_hex(),
                tokens: vec![constants_view.5.clone(), constants_view.6.clone()],
                contracts: vec![
                    LIQUIDITY_CONTRACT_ADDRESS.to_vec(),
                    DEX_RESERVES_RESOLVER.to_vec(),
                    event.dex.clone(),
                    implementations.2.to_vec(),
                    implementations.3.to_vec(),
                    implementations.4.to_vec(),
                ],
                static_att: vec![
                    Attribute {
                        name: "dex_id".to_string(),
                        value: BigInt::from(event.dex_id).to_signed_bytes_be(),
                        change: ChangeType::Creation.into(),
                    },
                    Attribute {
                        name: "dex_address".to_string(),
                        value: event.dex,
                        change: ChangeType::Creation.into(),
                    },
                    Attribute {
                        name: "reserves_resolver_address".to_string(),
                        value: resolver_address.into(),
                        change: ChangeType::Creation.into(),
                    },
                    Attribute {
                        name: "supply_token_0_slot".to_string(),
                        value: constants_view.7.to_vec(),
                        change: ChangeType::Creation.into(),
                    },
                    Attribute {
                        name: "borrow_token_0_slot".to_string(),
                        value: constants_view.8.to_vec(),
                        change: ChangeType::Creation.into(),
                    },
                    Attribute {
                        name: "supply_token_1_slot".to_string(),
                        value: constants_view.9.to_vec(),
                        change: ChangeType::Creation.into(),
                    },
                    Attribute {
                        name: "borrow_token_1_slot".to_string(),
                        value: constants_view.10.to_vec(),
                        change: ChangeType::Creation.into(),
                    },
                    Attribute {
                        name: "exchange_price_token_0_slot".to_string(),
                        value: constants_view.11.to_vec(),
                        change: ChangeType::Creation.into(),
                    },
                    Attribute {
                        name: "exchange_price_token_1_slot".to_string(),
                        value: constants_view.12.to_vec(),
                        change: ChangeType::Creation.into(),
                    },
                ],
                change: i32::from(ChangeType::Creation),
                protocol_type: Some(ProtocolType {
                    name: "fluid_dex_pool".to_string(),
                    financial_type: FinancialType::Swap.into(),
                    attribute_schema: vec![],
                    implementation_type: ImplementationType::Vm.into(),
                }),
            }],
        })
    };

    let mut eh = EventHandler::new(block);

    eh.filter_by_address(vec![Address::from_str(factory_address).unwrap()]);

    eh.on::<LogDexDeployed, _>(&mut on_dex_deployed);
    eh.handle_events();
}

/// Stores all contracts involved in quoting.
#[substreams::handlers::store]
pub fn store_contract_addresses(
    dexes_deployed: BlockTransactionProtocolComponents,
    store: StoreSetInt64,
) {
    for change in dexes_deployed.tx_components {
        for new_protocol_component in change.components.iter() {
            let addresses = new_protocol_component
                .contracts
                .iter()
                .map(hex::encode)
                .collect();
            store.set_many(0, &addresses, &1i64);
        }
    }
}

pub fn add_paused_attributes(
    dex_addresses: StoreGetRaw,
    tx_changes: &mut HashMap<u64, TransactionChangesBuilder>,
    block: &eth::v2::Block,
) {
    for log_view in block.logs() {
        let log = log_view.log;
        let tx = log_view.receipt.transaction;
        if !dex_addresses.has_address(Address::from_slice(log.address.as_slice())) {
            continue;
        }

        if let Some(_) = LogPauseSwapAndArbitrage::match_and_decode(log) {
            let builder = tx_changes
                .entry(tx.index as u64)
                .or_insert_with(|| TransactionChangesBuilder::new(&tx.into()));

            builder.add_entity_change(&EntityChanges {
                component_id: log.address.to_hex(),
                attributes: vec![Attribute {
                    name: "paused".to_string(),
                    value: vec![1u8],
                    change: ChangeType::Creation.into(),
                }],
            })
        }

        if let Some(_) = LogUnpauseSwapAndArbitrage::match_and_decode(log) {
            let builder = tx_changes
                .entry(tx.index as u64)
                .or_insert_with(|| TransactionChangesBuilder::new(&tx.into()));

            builder.add_entity_change(&EntityChanges {
                component_id: log.address.to_hex(),
                attributes: vec![Attribute {
                    name: "paused".to_string(),
                    value: vec![],
                    change: ChangeType::Deletion.into(),
                }],
            })
        }
    }
}

fn get_reserves_resolver(block_number: u64) -> &'static [u8] {
    if block_number >= 22487434 {
        DEX_RESERVES_RESOLVER_2
    } else {
        DEX_RESERVES_RESOLVER
    }
}

/// Aggregates protocol components and balance changes by transaction.
///
/// This is the main method that will aggregate all changes as well as extract all
/// relevant contract storage deltas.
///
/// ## Note:
/// You may have to change this method if your components have any default dynamic
/// attributes, or if you need any additional static contracts indexed.
#[substreams::handlers::map]
fn map_protocol_changes(
    block: eth::v2::Block,
    new_components: BlockTransactionProtocolComponents,
    contracts_store: StoreGetRaw,
) -> Result<BlockChanges, substreams::errors::Error> {
    // We merge contract changes by transaction (identified by transaction index)
    // making it easy to sort them at the very end.
    let mut transaction_changes: HashMap<_, TransactionChangesBuilder> = HashMap::new();

    let default_attributes = vec![Attribute {
        name: "paused".to_string(),
        value: vec![0u8],
        change: ChangeType::Creation.into(),
    }];

    // Aggregate newly created components per tx
    new_components
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
                    let calldata = hex::decode("bd964d38000000000000000000000000")
                        .expect("above literal should be safe to decode")
                        .into_iter()
                        .chain(component.contracts[2].clone())
                        .collect();

                    // create an entrypoint for each component
                    let (entrypoint, params) = create_entrypoint(
                        get_reserves_resolver(block.number).to_vec(),
                        "getPoolReservesAdjusted(address)".to_string(),
                        component.id.clone(),
                        TraceData::Rpc(RpcTraceData { caller: None, calldata }),
                    );
                    builder.add_entrypoint(&entrypoint);
                    builder.add_entrypoint_params(&params);
                    builder.add_entity_change(&EntityChanges {
                        component_id: component.id.clone(),
                        attributes: default_attributes.clone(),
                    })
                });
        });

    // Extract and insert any storage changes that happened for any of the components.
    extract_contract_changes_builder(
        &block,
        |addr| {
            contracts_store
                .get_last(hex::encode(addr))
                .is_some()
        },
        &mut transaction_changes,
    );

    // handle attributes if a pool was paused or unpaused
    add_paused_attributes(contracts_store, &mut transaction_changes, &block);

    let block_storage_changes = get_block_storage_changes(&block);

    // Process all `transaction_changes` for final output in the `BlockChanges`,
    //  sorted by transaction index (the key).
    Ok(BlockChanges {
        block: Some((&block).into()),
        changes: transaction_changes
            .drain()
            .sorted_unstable_by_key(|(index, _)| *index)
            .filter_map(|(_, builder)| builder.build())
            .collect::<Vec<_>>(),
        storage_changes: block_storage_changes,
    })
}
