use crate::modules::utils::Params;
use crate::{
    pb::cowamm::{
        CowPool, BlockPoolChanges, CowBalanceDelta, CowProtocolComponent, Attribute, ProtocolType, Transaction as CowTransaction,
        BlockBalanceDeltas, BlockTransactionProtocolComponents, TransactionProtocolComponents
    },
    events::get_log_changed_balances, 
    modules::map_cowpools::parse_binds
};
use anyhow::{Ok, Result};
use ethabi::ethereum_types::Address;
use substreams::prelude::{BigInt, StoreGetString};
use substreams::store::{StoreGet, StoreGetProto};
use substreams_ethereum::pb::eth::v2::Block;
use substreams_ethereum::Event;
use substreams_helper::hex::Hexable;
use tycho_substreams::prelude::*;

fn create_component(factory_address: &[u8], pool: CowPool) -> Option<CowProtocolComponent> {
         Some(CowProtocolComponent {
        id: pool.address.to_hex(),                      
        tokens: vec![
            pool.token_a.to_vec(),
            pool.token_b.to_vec(),
            pool.lp_token.to_vec(),
        ],
        contracts: vec![
            factory_address.to_vec(),
            pool.address.to_vec(),
        ],
        static_att: vec![
            Attribute {
                name: "token_a".to_string(),
                value: pool.token_a.to_vec(),
                change: ChangeType::Creation.into(),
            },
            Attribute {
                name: "token_b".to_string(),
                value: pool.token_b.to_vec(),
                change: ChangeType::Creation.into(),
            },
            Attribute {
                name: "lp_token".to_string(),
                value: pool.lp_token.to_vec(),
                change: ChangeType::Creation.into(),
            },
            Attribute {
                name: "fee".to_string(),
                value: BigInt::from(0).to_signed_bytes_be(),
                change: ChangeType::Creation.into(),
            },
            Attribute {
                name: "weight_a".to_string(),
                value: pool.weight_a.to_vec(),
                change: ChangeType::Creation.into(),
            },
            Attribute {
                name: "weight_b".to_string(),
                value: pool.weight_b.to_vec(),
                change: ChangeType::Creation.into(),
            },
        ],
        change: ChangeType::Creation.into(),
        protocol_type: Some(ProtocolType {
            name: "cowamm_pool".to_string(),
            financial_type: FinancialType::Swap.into(),
            attribute_schema: vec![],
            implementation_type: ImplementationType::Vm.into(),
        }),
    })
}

#[substreams::handlers::map]
pub fn map_components_with_balances(
    params: String,
    block: Block,
    store: StoreGetProto<CowPool>,
    binds: StoreGetString
) -> Result<BlockPoolChanges, substreams::errors::Error> {
    let params = Params::parse_from_query(&params)?;
    const COWAMM_POOL_CREATED_TOPIC: &str =
        "0x0d03834d0d86c7f57e877af40e26f176dc31bd637535d4ba153d1ac9de88a7ea";
    let factory_address = params.decode_addresses().unwrap();
    let store = &store;
    let mut tx_deltas = Vec::new();
    let mut tx_protocol_components = Vec::new();

    for tx in block.transactions() {
        let mut tx_components = Vec::new();
        for (log, call) in tx.logs_with_calls() {
            // Skip reverted calls
            if call.call.state_reverted {
                continue;
            }
            let is_pool_creation = 
                log.address == factory_address
                && log.topics
                    .get(0)
                    .map(|t| t.to_hex()) == Some(COWAMM_POOL_CREATED_TOPIC.to_string());
            
            if is_pool_creation {
                // Handle pool creation
                let pool_address_topic = match log.topics.get(1) {
                    Some(topic) => topic.as_slice()[12..].to_vec(),
                    None => continue,
                };
                
                let pool_address_hex = hex::encode(&pool_address_topic);
                let pool_key = format!("Pool:0x{}", pool_address_hex);

                let bind_data = match binds.get_first(&pool_address_hex) {
                    Some(data) => data,
                    None => continue,
                };
                
                let parsed_binds = match parse_binds(&bind_data) {
                    Some(binds) if !binds.is_empty() => binds,
                    _ => continue,
                };
                
                // substreams::log::info!("THIS IS THE PARSED BINDS:{:?}", parsed_binds);
                // substreams::log::info!("THIS IS THE PARSED BINDS LENGTH:{:?}", parsed_binds.len());
                // substreams::log::info!("THIS IS THE PARSED BINDS ADDRESS:{:?}", parsed_binds.len());
                
                // Create deltas for each bind, since the bind happens before the actual CowAMMPoolCreated event 
                //we'll add the deltas in the block where the pool was created
                
                for bind in parsed_binds.iter() {
                    substreams::log::info!("We are in bind right now {}", hex::encode(&log.address));
                    substreams::log::info!("We are in bind right now {:?}", bind);
                    
                    let bind_tx = bind.tx.as_ref().unwrap();
                    let delta = BalanceDelta {
                        ord: bind.ordinal,
                        tx: Some(Transaction {
                            from: bind_tx.from.clone(),
                            to: bind_tx.to.clone(),
                            hash: bind_tx.hash.clone(),
                            index: bind_tx.index.clone(),
                        }),
                        token: bind.token.clone(),
                        delta: BigInt::from_unsigned_bytes_be(&bind.amount).to_signed_bytes_be(),
                        component_id: bind.address.clone().to_hex().as_bytes().to_vec(), 
                    };
                    tx_deltas.push(delta);
                }
                let pool = store
                        .get_last(pool_key)
                        .expect("failed to get pool from store");
                // Create the component
                if let Some(component) = create_component(&factory_address, pool.clone()) {
                    tx_components.push(component);
                }
            } else {
                //its possible that we encounter logs from a CowAMMPool buts its not creation related
                //we extract any balance deltas from this log
                if let Some(pool) = store.get_last(format!("Pool:{}", &log.address.to_hex())) {
                    tx_deltas.extend(get_log_changed_balances(&tx.into(), log, &pool));
                } else {
                    continue;
                }
            }
            if !tx_components.is_empty() {
                tx_protocol_components.push(
                    TransactionProtocolComponents { tx: Some(tx.into()), components: tx_components.clone()}
                )
            }
        }
    }

    //convert normal balance deltas to cow balance deltas 
    let final_deltas = tx_deltas.iter().map(|delta| {
        delta.into()
    }).collect::<Vec<CowBalanceDelta>>();

    Ok(BlockPoolChanges {
        //we need the tx object 
        tx_protocol_components: Some(BlockTransactionProtocolComponents { 
            tx_components: tx_protocol_components
        }),
        // dont want to make it inconsistne by using a non Block-type
        block_balance_deltas: Some(BlockBalanceDeltas { balance_deltas: final_deltas })
    })
}
