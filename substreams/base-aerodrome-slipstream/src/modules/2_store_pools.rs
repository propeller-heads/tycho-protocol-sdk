use std::str;

use substreams::store::{StoreNew, StoreSetIfNotExists, StoreSetIfNotExistsProto};
use tycho_substreams::models::BlockTransactionProtocolComponents;
use crate::pb::aerodrome::slipstream::Pool;

#[substreams::handlers::store]
pub fn store_pools(pools_created: BlockTransactionProtocolComponents, store: StoreSetIfNotExistsProto<Pool>) {
    for tx_pc in pools_created.tx_components {
        for component_change in &tx_pc.components {
            let pool_address: &str = &component_change.id;
            let pool: Pool = Pool {
                address: hex::decode(pool_address.trim_start_matches("0x")).unwrap(),
                token0: component_change.tokens[0].clone(),
                token1: component_change.tokens[1].clone(),
                created_tx_hash: tx_pc.tx.as_ref().unwrap().hash.clone(),
            };
            store.set_if_not_exists(0, format!("{}:{}", "Pool", pool_address), &pool);
        }
    }
}
