use std::str::FromStr;

use ethabi::ethereum_types::Address;
use serde::Deserialize;
use substreams_ethereum::pb::eth::v2::{self as eth};
use substreams_ethereum::Event;

use crate::abi;

use tycho_substreams::prelude::*;

#[derive(Debug, Deserialize)]
struct Params {
    factory_address: String,
}

#[substreams::handlers::map]
pub fn map_pools_created(
    params: String,
    block: eth::Block,
) -> Result<BlockChanges, substreams::errors::Error> {
    let mut new_pools: Vec<TransactionChanges> = vec![];

    let query_params: Params = serde_qs::from_str(params.as_str()).expect("Unable to deserialize params");

    get_pools(&block, &mut new_pools, &query_params.factory_address);

    let tycho_block: Block = (&block).into();

    Ok(BlockChanges { block: Some(tycho_block), changes: new_pools })
}

fn get_pools(block: &eth::Block, new_pools: &mut Vec<TransactionChanges>, factory_address: &str) {
    let factory_addr = Address::from_str(factory_address).unwrap();

    for trx in block.transactions() {
        let tycho_tx: Transaction = trx.into();
        if let Some(receipt) = trx.receipt.as_ref() {
            for log in &receipt.logs {
                if log.address != factory_addr.as_bytes() {
                    continue;
                }

            if let Some(event) = abi::algebrafactory::events::Pool::match_and_decode(log) {
                substreams::log::info!(
                    "Pool Created: address=0x{} token0=0x{} token1=0x{}",
                    substreams::Hex(&event.pool),
                    substreams::Hex(&event.token0),
                    substreams::Hex(&event.token1)
                );
                let pool_id = format!("{}", substreams::Hex(&event.pool)).to_lowercase();
                let factory_id = format!("{}", substreams::Hex(&factory_addr)).to_lowercase();

                new_pools.push(TransactionChanges {
                    tx: Some(tycho_tx.clone()),
                    contract_changes: vec![ContractChange {
                        address: event.pool.clone(),
                        change: ChangeType::Creation.into(),
                        ..Default::default()
                    }],
                    entity_changes: vec![],
                    component_changes: vec![ProtocolComponent {
                        id: pool_id.clone(),
                        tokens: vec![event.token0.clone(), event.token1.clone()],
                        contracts: vec![event.pool.clone()],
                        static_att: vec![
                            Attribute {
                                name: "token0".to_string(),
                                value: event.token0.clone(),
                                change: ChangeType::Creation.into(),
                            },
                            Attribute {
                                name: "token1".to_string(),
                                value: event.token1.clone(),
                                change: ChangeType::Creation.into(),
                            },
                            Attribute {
                                name: "deployer".to_string(),
                                value: factory_id.as_bytes().to_vec(),
                                change: ChangeType::Creation.into(),
                            },
                        ],
                        change: i32::from(ChangeType::Creation),
                        protocol_type: Some(ProtocolType {
                            name: "supernova_algebra_pool".to_string(),
                            financial_type: FinancialType::Swap.into(),
                            attribute_schema: vec![],
                            implementation_type: ImplementationType::Vm.into(),
                        }),
                        tx: Some(tycho_tx.clone()),
                    }],
                    ..Default::default()
                });
            } else if let Some(event) = abi::algebrafactory::events::CustomPool::match_and_decode(log) {
                substreams::log::info!(
                    "CustomPool Created: address={} token0={} token1={}",
                    substreams::Hex(&event.pool),
                    substreams::Hex(&event.token0),
                    substreams::Hex(&event.token1)
                );
                let pool_id = format!("{}", substreams::Hex(&event.pool)).to_lowercase();
                let factory_id = format!("{}", substreams::Hex(&factory_addr)).to_lowercase();

                new_pools.push(TransactionChanges {
                    tx: Some(tycho_tx.clone()),
                    contract_changes: vec![ContractChange {
                        address: event.pool.clone(),
                        change: ChangeType::Creation.into(),
                        ..Default::default()
                    }],
                    entity_changes: vec![],
                    component_changes: vec![ProtocolComponent {
                        id: pool_id.clone(),
                        tokens: vec![event.token0.clone(), event.token1.clone()],
                        contracts: vec![event.pool.clone()],
                        static_att: vec![
                            Attribute {
                                name: "token0".to_string(),
                                value: event.token0.clone(),
                                change: ChangeType::Creation.into(),
                            },
                            Attribute {
                                name: "token1".to_string(),
                                value: event.token1.clone(),
                                change: ChangeType::Creation.into(),
                            },
                            Attribute {
                                name: "deployer".to_string(),
                                value: factory_id.as_bytes().to_vec(),
                                change: ChangeType::Creation.into(),
                            },
                        ],
                        change: i32::from(ChangeType::Creation),
                        protocol_type: Some(ProtocolType {
                            name: "supernova_algebra_pool".to_string(),
                            financial_type: FinancialType::Swap.into(),
                            attribute_schema: vec![],
                            implementation_type: ImplementationType::Vm.into(),
                        }),
                        tx: Some(tycho_tx.clone()),
                    }],
                    ..Default::default()
                });
            }
        }
    }
    }
}
