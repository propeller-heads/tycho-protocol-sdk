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
use crate::modules::utils::extract_address;

fn create_component(factory_address: &[u8], pool: CowPool) -> Option<CowProtocolComponent> {
         Some(CowProtocolComponent {
        id: pool.address.to_hex(),                      
        tokens: vec![
            pool.token_a.to_vec(),
            pool.token_b.to_vec(),
            pool.lp_token.to_vec(),
        ],
        contracts: vec![],
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
    const COW_PROTOCOL_GPV2_SETTLEMENT_ADDRESS: &str = "0x9008d19f58aabd9ed0d60971565aa8510560ab41";
    const COW_PROTOCOL_GPV2_TOPIC: &str = "0xa07a543ab8a018198e99ca0184c93fe9050a79400a0a723441f84de1d972cc17";

    let factory_address = params.decode_addresses().expect("failed to get factory address");
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
                
                for bind in parsed_binds.iter() {
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
                //this case extract any balance deltas from this log that is CowAMM related for the particular pool
            } else if let Some(pool) = store.get_last(format!("Pool:{}", &log.address.to_hex())) {
                tx_deltas.extend(get_log_changed_balances(&tx.into(), log, &pool));
            } else if log.address.to_hex() == COW_PROTOCOL_GPV2_SETTLEMENT_ADDRESS {
                //when a trade is settled on the CowAMM via the cowprotocol a delta also occurs but the log.address 
                //will be the GPV2_SETTLEMENT address, we just have to check the owner if its this pool

                //https://etherscan.io/tx/0x530416d2f894e7d029a42854fc7656a1605a4bddf711707e41e4c8997becbac5#eventlog#504 example
                if log.topics.get(0).map(|t| t.to_hex()) == Some(COW_PROTOCOL_GPV2_TOPIC.to_string()) {
                    if let Some(pool_address) = log.topics.get(1).map(|t| t.to_hex()) {
                        //24 + 40 chars 
                        //pool is address is left padded with 24 '0's so we remove that
                        //0x0000000000000000000000009bd702e05b9c97e4a4a3e47df1e0fe7a0c26d2f1 left padded to 44 bytes
                        let address = extract_address(&pool_address, 40);
                        if let Some(pool) = store.get_last(format!("Pool:{}", &address)) {
                            tx_deltas.extend(get_log_changed_balances(&tx.into(), log, &pool));
                        }
                    }
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
        tx_protocol_components: Some(BlockTransactionProtocolComponents { 
            tx_components: tx_protocol_components
        }),
        block_balance_deltas: Some(BlockBalanceDeltas { balance_deltas: final_deltas })
    })
}
