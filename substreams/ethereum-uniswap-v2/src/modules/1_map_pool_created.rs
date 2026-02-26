use std::{collections::HashMap, str::FromStr};

use ethabi::ethereum_types::Address;
use serde::Deserialize;
use substreams::prelude::BigInt;
use substreams_ethereum::pb::eth::v2::{self as eth};
use substreams_helper::{event_handler::EventHandler, hex::Hexable};

use crate::abi::factory::events::PairCreated;

use tycho_substreams::prelude::*;

fn default_fee() -> u32 {
    30
}

#[derive(Debug, Deserialize)]
struct Params {
    factory_address: String,
    protocol_type_name: String,
    #[serde(default = "default_fee")]
    fee: u32,
}

/// Parse the factory_address field into addresses and per-factory fee overrides.
///
/// Supports two formats:
/// - Single address: `"abcdef1234..."`
/// - Comma-separated with optional `:fee` suffix: `"addr1,addr2:25,addr3:20"`
///
/// Addresses without a `:fee` suffix use `default_fee`.
fn parse_factories(
    factory_address: &str,
    default_fee: u32,
) -> (Vec<Address>, HashMap<[u8; 20], u32>) {
    let mut addresses = Vec::new();
    let mut fee_overrides = HashMap::new();

    for entry in factory_address.split(',') {
        let entry = entry.trim();
        if entry.is_empty() {
            continue;
        }
        let (addr_str, fee) = match entry.split_once(':') {
            Some((addr, fee_str)) => {
                let fee: u32 = fee_str
                    .parse()
                    .unwrap_or_else(|_| panic!("invalid fee '{fee_str}' in factory_address"));
                assert!(fee <= 10000, "fee must be <= 10000 bps, got {fee}");
                (addr, fee)
            }
            None => (entry, default_fee),
        };
        let address =
            Address::from_str(addr_str).unwrap_or_else(|_| panic!("invalid address '{addr_str}'"));
        if fee != default_fee {
            fee_overrides.insert(address.0, fee);
        }
        addresses.push(address);
    }

    (addresses, fee_overrides)
}

#[substreams::handlers::map]
pub fn map_pools_created(
    params: String,
    block: eth::Block,
) -> Result<BlockChanges, substreams::errors::Error> {
    let mut new_pools: Vec<TransactionChanges> = vec![];

    let params: Params =
        serde_qs::from_str(params.as_str()).expect("Unable to deserialize params");

    assert!(
        params.fee <= 10000,
        "fee must be <= 10000 bps, got {}",
        params.fee
    );

    let (factory_addresses, fee_overrides) =
        parse_factories(&params.factory_address, params.fee);

    get_pools(
        &block,
        &mut new_pools,
        &factory_addresses,
        &fee_overrides,
        params.fee,
        &params.protocol_type_name,
    );

    let tycho_block: Block = (&block).into();

    Ok(BlockChanges { block: Some(tycho_block), changes: new_pools })
}

fn get_pools(
    block: &eth::Block,
    new_pools: &mut Vec<TransactionChanges>,
    factory_addresses: &[Address],
    fee_overrides: &HashMap<[u8; 20], u32>,
    default_fee: u32,
    protocol_type_name: &str,
) {
    let fee_overrides = fee_overrides.clone();
    let protocol_type_name = protocol_type_name.to_string();

    let mut on_pair_created =
        |event: PairCreated, _tx: &eth::TransactionTrace, _log: &eth::Log| {
            // Look up per-factory fee, fall back to default
            let mut log_addr = [0u8; 20];
            log_addr.copy_from_slice(&_log.address);
            let fee = fee_overrides
                .get(&log_addr)
                .copied()
                .unwrap_or(default_fee);

            let tycho_tx: Transaction = _tx.into();

            new_pools.push(TransactionChanges {
                tx: Some(tycho_tx.clone()),
                contract_changes: vec![],
                entity_changes: vec![EntityChanges {
                    component_id: event.pair.to_hex(),
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
                    id: event.pair.to_hex(),
                    tokens: vec![event.token0.clone(), event.token1.clone()],
                    contracts: vec![],
                    static_att: vec![
                        Attribute {
                            name: "fee".to_string(),
                            value: BigInt::from(fee).to_signed_bytes_be(),
                            change: ChangeType::Creation.into(),
                        },
                        Attribute {
                            name: "pool_address".to_string(),
                            value: event.pair.clone(),
                            change: ChangeType::Creation.into(),
                        },
                    ],
                    change: i32::from(ChangeType::Creation),
                    protocol_type: Some(ProtocolType {
                        name: protocol_type_name.clone(),
                        financial_type: FinancialType::Swap.into(),
                        attribute_schema: vec![],
                        implementation_type: ImplementationType::Custom.into(),
                    }),
                    tx: Some(tycho_tx),
                }],
                balance_changes: vec![
                    BalanceChange {
                        token: event.token0,
                        balance: BigInt::from(0).to_signed_bytes_be(),
                        component_id: event.pair.to_hex().as_bytes().to_vec(),
                    },
                    BalanceChange {
                        token: event.token1,
                        balance: BigInt::from(0).to_signed_bytes_be(),
                        component_id: event.pair.to_hex().as_bytes().to_vec(),
                    },
                ],
            })
        };

    let mut eh = EventHandler::new(block);
    eh.filter_by_address(factory_addresses.to_vec());
    eh.on::<PairCreated, _>(&mut on_pair_created);
    eh.handle_events();
}
