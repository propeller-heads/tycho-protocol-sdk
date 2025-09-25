use std::str::FromStr;

use crate::abi::factory::events::PoolCreated;
use ethabi::ethereum_types::Address;
use serde::Deserialize;
use substreams::prelude::BigInt;
use substreams_ethereum::pb::eth::v2::{self as eth};
use substreams_helper::{event_handler::EventHandler, hex::Hexable};
use tycho_substreams::entrypoint::create_entrypoint;

use tycho_substreams::prelude::{entry_point_params::TraceData, *};

#[derive(Debug, Deserialize)]
struct Params {
    factory_address: String,
    protocol_type_name: String,
}

#[substreams::handlers::map]
pub fn map_pools_created(
    params: String,
    block: eth::Block,
) -> Result<BlockChanges, substreams::errors::Error> {
    let mut new_pools: Vec<TransactionChanges> = vec![];

    let params: Params = serde_qs::from_str(params.as_str()).expect("Unable to deserialize params");

    get_pools(&block, &mut new_pools, &params);

    let tycho_block: Block = (&block).into();

    Ok(BlockChanges { block: Some(tycho_block), changes: new_pools, storage_changes: vec![] })
}

fn get_pools(block: &eth::Block, new_pools: &mut Vec<TransactionChanges>, params: &Params) {
    // Extract new pools from PairCreated events
    let mut on_pair_created = |event: PoolCreated, tx: &eth::TransactionTrace, log: &eth::Log| {
        let tycho_tx: Transaction = tx.into();
        let fee_trace_data = TraceData::Rpc(RpcTraceData {
            caller: None,
            calldata: [
                &hex::decode("cc56b2c5").unwrap()[..], // selector
                &[0u8; 12],                            // padding for address
                &event.pool[..],                       // 20-byte address
                &{
                    let mut b = [0u8; 32];
                    b[31] = if event.stable { 1 } else { 0 }; // bool param
                    b
                }[..],
            ]
            .concat(),
        });
        let (entrypoint, entrypoint_param) = create_entrypoint(
            hex::decode(params.factory_address).unwrap(),
            "getFee(address,bool)".to_string(),
            event.pool.to_hex(),
            fee_trace_data,
        );
        new_pools.push(TransactionChanges {
            tx: Some(tycho_tx.clone()),
            contract_changes: vec![],
            entity_changes: vec![EntityChanges {
                component_id: event.pool.to_hex(),
                attributes: vec![
                    Attribute {
                        name: "reserve0".to_string(),
                        value: BigInt::from(0).to_signed_bytes_be(),
                        change: ChangeType::Creation.into(),
                    },
                    Attribute {
                        name: "reserve1".to_string(),
                        value: BigInt::from(0).to_signed_bytes_be(),
                        change: ChangeType::Creation.into(),
                    },
                ],
            }],
            component_changes: vec![ProtocolComponent {
                id: event.pool.to_hex(),
                tokens: vec![event.token0.clone(), event.token1.clone()],
                contracts: vec![],
                static_att: vec![
                    Attribute {
                        name: "pool_address".to_string(),
                        value: event.pool.clone(),
                        change: ChangeType::Creation.into(),
                    },
                    Attribute {
                        name: "stable".to_string(),
                        value: vec![event.stable as u8],
                        change: ChangeType::Creation.into(),
                    },
                ],
                change: i32::from(ChangeType::Creation),
                protocol_type: Some(ProtocolType {
                    name: params.protocol_type_name.to_string(),
                    financial_type: FinancialType::Swap.into(),
                    attribute_schema: vec![],
                    implementation_type: ImplementationType::Custom.into(),
                }),
            }],
            balance_changes: vec![
                BalanceChange {
                    token: event.token0,
                    balance: BigInt::from(0).to_signed_bytes_be(),
                    component_id: event.pool.to_hex().as_bytes().to_vec(),
                },
                BalanceChange {
                    token: event.token1,
                    balance: BigInt::from(0).to_signed_bytes_be(),
                    component_id: event.pool.to_hex().as_bytes().to_vec(),
                },
            ],
            entrypoints: vec![entrypoint],
            entrypoint_params: vec![entrypoint_param],
        })
    };

    let mut eh = EventHandler::new(block);

    eh.filter_by_address(vec![Address::from_str(&params.factory_address).unwrap()]);

    eh.on::<PoolCreated, _>(&mut on_pair_created);

    eh.handle_events();
}
