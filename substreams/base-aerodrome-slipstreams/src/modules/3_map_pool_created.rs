use crate::{abi::factory::events::PoolCreated, modules::utils::Params};
use ethabi::ethereum_types::Address;
use prost::Message;
use std::str::FromStr;
use substreams::store::{StoreGet, StoreGetInt64};
use substreams_ethereum::pb::eth::v2::{self as eth};
use substreams_helper::{event_handler::EventHandler, hex::Hexable};
use tycho_substreams::prelude::*;

#[substreams::handlers::map]
pub fn map_pools_created(
    params: String,
    block: eth::Block,
    tick_spacing_to_fee_store: StoreGetInt64,
) -> Result<BlockTransactionProtocolComponents, substreams::errors::Error> {
    let mut new_pools: Vec<TransactionProtocolComponents> = vec![];
    let params = Params::parse_from_query(&params)?;
    get_new_pools(&block, &mut new_pools, params.factory.as_str(), tick_spacing_to_fee_store);

    Ok(BlockTransactionProtocolComponents { tx_components: new_pools })
}

// Extract new pools from PoolCreated events
fn get_new_pools(
    block: &eth::Block,
    new_pools: &mut Vec<TransactionProtocolComponents>,
    factory_address: &str,
    tick_spacing_to_fee_store: StoreGetInt64,
) {
    // Extract new pools from PoolCreated events
    let mut on_pool_created = |event: PoolCreated, _tx: &eth::TransactionTrace, _log: &eth::Log| {
        let tycho_tx: Transaction = _tx.into();
        // Get default fee for tick spacing
        let default_fee = tick_spacing_to_fee_store
            .get_last(event.tick_spacing.to_string())
            .unwrap_or_default(); // todo default fee for test
        new_pools.push(TransactionProtocolComponents {
            tx: Some(tycho_tx.clone()),
            components: vec![ProtocolComponent {
                id: event.pool.to_hex(),
                tokens: vec![event.token0, event.token1],
                contracts: vec![],
                static_att: vec![
                    Attribute {
                        name: "default_fee".to_string(),
                        value: default_fee.encode_to_vec(),
                        change: ChangeType::Creation.into(),
                    },
                    Attribute {
                        name: "tick_spacing".to_string(),
                        value: event.tick_spacing.to_signed_bytes_be(),
                        change: ChangeType::Creation.into(),
                    },
                    Attribute {
                        name: "pool_address".to_string(),
                        value: event.pool,
                        change: ChangeType::Creation.into(),
                    },
                ],
                change: i32::from(ChangeType::Creation),
                protocol_type: Option::from(ProtocolType {
                    name: "aerodrome_slipstreams_pool".to_string(),
                    financial_type: FinancialType::Swap.into(),
                    attribute_schema: vec![],
                    implementation_type: ImplementationType::Custom.into(),
                }),
            }],
        })
    };

    let mut eh = EventHandler::new(block);

    eh.filter_by_address(vec![Address::from_str(factory_address).unwrap()]);

    eh.on::<PoolCreated, _>(&mut on_pool_created);
    eh.handle_events();
}
