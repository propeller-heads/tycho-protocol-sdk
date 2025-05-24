use crate::pb::cowamm::v2::BCowPool;
use substreams::{
    prelude::{StoreSetIfNotExists, StoreSetIfNotExistsProto},
    store::StoreNew,
};
use tycho_substreams::prelude::*;

#[substreams::handlers::store]
pub fn store_components(
    map: BlockTransactionProtocolComponents,
    store: StoreSetIfNotExistsProto<Pool>,
) {
    for tx_pc in map.tx_components { //tx_components here is new_pools
        for pc in tx_pc.components {
            // let pool_address = &pc.; //tx_pc.components here is TransactionProtocolComponents
            let pool = CowPool {
                address: hex::decode(pool_address.trim_start_matches("0x")).unwrap(),
                token_a: pc.tokens[0].clone(),
                token_b: pc.tokens[1].clone(),
                weight_a:pc.denorm.clone(),
                weight_b:pc.denorm.clone(),
                fee: BigInt::from_signed_bytes_be(
                    &pc
                        .static_att
                        .iter()
                        .find(|attr| attr.name == "swap_fee")
                        .expect("every pool should have swap_fee as static attribute")
                        .value,
                )
                .to_u64(),
                created_tx_hash: tx_pc.tx.as_ref().unwrap().hash.clone(),
            };
            store.set_if_not_exists(0, format!("Pool:{}", pool_address), &pool);
        }
    }
}
