use substreams::prelude::{StoreSetIfNotExists, StoreSetIfNotExistsProto};
use substreams::store::{StoreNew};
use tycho_substreams::prelude::*;
use crate::pb::maverick::v2::Pool;
#[substreams::handlers::store]
pub fn store_components(map: BlockTransactionProtocolComponents, store: StoreSetIfNotExistsProto<Pool>) {
    for tx_pc in map.tx_components{
        for pc in tx_pc.components{
            let pool_address = &pc.id;
            let pool = Pool {
                address: hex::decode(pool_address.trim_start_matches("0x")).unwrap(),
                token_a: pc.tokens[0].clone(),
                token_b: pc.tokens[1].clone(),
            };
            store.set_if_not_exists(0, format!("{}:{}", "Pool", pool_address), &pool);
        }
    }
}