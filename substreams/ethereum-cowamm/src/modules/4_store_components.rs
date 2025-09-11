use crate::pb::cowamm::CowPool;
use substreams::{
    prelude::{StoreSetIfNotExists, StoreSetIfNotExistsProto},
    scalar::BigInt,
    store::StoreNew,
};
use tycho_substreams::prelude::*;

#[substreams::handlers::store]
pub fn store_components(
    map: BlockTransactionProtocolComponents,
    store: StoreSetIfNotExistsProto<CowPool>,
) {
    for tx_pc in map.tx_components {
        for pc in tx_pc.components {
            let pool_address = &pc.id;
            let pool = CowPool {
                address: hex::decode(pool_address.trim_start_matches("0x"))
                    .expect("failed to decode pool address"),
                token_a: pc.tokens[0].clone(),
                token_b: pc.tokens[1].clone(),
                liquidity_a: pc
                    .get_attribute_value("liquidity_a")
                    .expect("every cow pool should have liquidity_a as static attribute"),
                liquidity_b: pc
                    .get_attribute_value("liquidity_b")
                    .expect("every cow pool should have liquidity_b as static attribute"),
                lp_token: pc.tokens[2].clone(),
                lp_token_supply: pc
                    .get_attribute_value("lp_token_supply")
                    .expect("every cow pool should have lp_token_supply as static attribute"),
                weight_a: pc
                    .get_attribute_value("weight_a")
                    .expect("every cow pool should have weight_a as static attribute"),
                weight_b: pc
                    .get_attribute_value("weight_b")
                    .expect("every cow pool should have weight_b as static attribute"),
                fee: 0 as u64,
                created_tx_hash: tx_pc.tx.as_ref().unwrap().hash.clone(),
            };
            store.set_if_not_exists(0, format!("Pool:{}", pool_address), &pool);
        }
    }
}
