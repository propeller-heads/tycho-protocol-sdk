use crate::pb::cowamm::CowPool;
use substreams::{
    scalar::BigInt,
    prelude::{StoreSetIfNotExists, StoreSetIfNotExistsProto},
    store::StoreNew,
};
use tycho_substreams::prelude::*;

#[substreams::handlers::store]
pub fn store_components(
    map: BlockTransactionProtocolComponents,
    store: StoreSetIfNotExistsProto<CowPool>,
) {
    for tx_pc in map.tx_components { //tx_components here is new_pools
        for pc in tx_pc.components {
            let pool_address = &pc.id; //tx_pc.components here is TransactionProtocolComponents
            let pool = CowPool {
                address: hex::decode(pool_address.trim_start_matches("0x")).unwrap(),
                token_a: pc.tokens[0].clone(),
                token_b: pc.tokens[1].clone(),
                lp_token: pc.static_att
                    .iter()
                    .find(|attr| attr.name == "lp_token")
                    .expect("every cow pool should have lp_token as static attribute")
                    .value.clone(), //look at this later
                weight_a: BigInt::from_signed_bytes_be(&pc
                    .static_att
                    .iter()
                    .find(|attr| attr.name == "denormalized_weight_a")
                    .expect("every cow pool should have denormalized_weight_a as static attribute")
                    .value).to_u64(),
                weight_b: BigInt::from_signed_bytes_be(&pc    
                    .static_att
                    .iter()
                    .find(|attr| attr.name == "denormalized_weight_b")
                    .expect("every cow pool should have denormalized_weight_b as static attribute")
                    .value).to_u64(),
                fee: 0 as u64,
                created_tx_hash: tx_pc.tx.as_ref().unwrap().hash.clone(),
            };
            store.set_if_not_exists(0, format!("Pool:{}", pool_address), &pool);
        }
    }
}
