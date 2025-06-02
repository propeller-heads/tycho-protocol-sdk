use std::collections::HashMap;
use crate::{modules::utils::Params};
use anyhow::{Ok, Result};
use ethabi::ethereum_types::Address;
use substreams::{prelude::BigInt};
use hex_literal::hex;
use crate::pb::cowamm::{CowPool, CowPools};
use substreams_ethereum::pb::eth::v2::{Block};
use substreams_ethereum::Event;
use tycho_substreams::prelude::*;
use crate::abi::{b_cow_factory::events::CowammPoolCreated, b_cow_pool::functions::Bind};
use substreams_helper::{event_handler::EventHandler, hex::Hexable};
use substreams::{
    pb::substreams::StoreDeltas,
    store::{StoreGet, StoreGetProto},
};
use substreams::log::info;

fn create_component(
    factory_address: &[u8],
    pool: CowPool,
) -> Option<ProtocolComponent> {

    Some(
        ProtocolComponent::new(&pool.address.to_hex())
        .with_tokens(&[pool.token_a.as_slice(), pool.token_a.as_slice()])  //remember the address of the contract is the address of the lp token  
        .with_contracts(&[
            factory_address,
            pool.address.as_slice()
        ])
        .with_attributes(&[  
            ("fee", BigInt::from(0).to_signed_bytes_be()),
            ("lp_token", pool.lp_token),
            ("normalized_weight_a", BigInt::from(pool.weight_a).to_signed_bytes_be()), 
            ("normalized_weight_b", BigInt::from(pool.weight_b).to_signed_bytes_be()),
        ])
        .as_swap_type("cowamm_pool", ImplementationType::Vm)
    )
}

#[substreams::handlers::map]
pub fn map_components(params: String, block: Block, store: StoreGetProto<CowPool>) -> Result<BlockTransactionProtocolComponents> {
    let params = Params::parse_from_query(&params)?;
    let factory_address = params.decode_addresses().unwrap();

    let store = &store;
        Ok(BlockTransactionProtocolComponents {
            tx_components: 
              block
                .transactions() 
                .filter_map(|tx| {
                    let components = tx
                        .logs_with_calls()
                        .filter(|(log, _)| log.address == factory_address)
                        .filter_map(|(log, call)| {
                            let tx_hash = hex::encode(&tx.hash);
                            let pool = store.get_last(tx_hash)?;
                            create_component(&factory_address, pool.clone())
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
