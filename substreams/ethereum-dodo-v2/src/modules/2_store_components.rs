use crate::pb::dodo::v2::Pool;
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
    for tx_pc in map.tx_components {
        for pc in tx_pc.components {
            let pool_address = &pc.id;
            let pool = Pool {
                address: hex::decode(pool_address.trim_start_matches("0x")).unwrap(),
                base_token: pc.tokens[0].clone(),
                quote_token: pc.tokens[1].clone(),
                created_tx_hash: tx_pc.tx.as_ref().unwrap().hash.clone(),
                pool_type: pc
                    .get_attribute_value("pool_type")
                    .unwrap_or_default(),
            };
            store.set_if_not_exists(0, format!("Pool:{pool_address}"), &pool);
        }
    }
}
