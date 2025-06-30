use crate::{modules::utils::Params};
use anyhow::{Ok, Result};
use ethabi::ethereum_types::Address;
use substreams::{prelude::BigInt};
use crate::pb::cowamm::{CowPool};
use substreams_ethereum::pb::eth::v2::{Block};
use substreams_ethereum::Event;
use tycho_substreams::prelude::*;
use substreams_helper::{hex::Hexable};
use substreams::{
    store::{StoreGet, StoreGetProto},
};

fn create_component(
    factory_address: &[u8],
    pool: CowPool,
) -> Option<ProtocolComponent> {

    Some(
        ProtocolComponent::new(&pool.address.to_hex())
        .with_tokens(&[pool.token_a.as_slice(), pool.token_b.as_slice(), pool.lp_token.as_slice()]) 
        .with_contracts(&[
            factory_address,
            pool.address.as_slice()
        ])
        .with_attributes(&[
            ("lp_token", pool.lp_token.as_slice()),
            ("token_a", pool.token_a.as_slice()),
            ("token_b", pool.token_b.as_slice()),
            ("fee", BigInt::from(0).to_signed_bytes_be()),
            ("weight_a", pool.weight_a.as_slice()), 
            ("weight_b", pool.weight_b.as_slice()),
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
                        .filter_map(|(log, _)| {
                            let pool_address = &log
                                .topics
                                .get(1)
                                .map(|topic| topic.as_slice()[12..].to_vec())?; 
                            let pool_key = format!("Pool:0x{}",hex::encode(&pool_address));
                            let pool = store.get_last(pool_key).expect("failed to get pool from store");
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
