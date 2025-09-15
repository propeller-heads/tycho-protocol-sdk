use substreams::{
    prelude::StoreSet,
    store::{StoreNew, StoreSetString},
};
use tycho_substreams::prelude::*;
#[substreams::handlers::store]
pub fn store_token_mapping(map: BlockTransactionProtocolComponents, store: StoreSetString) {
    map.tx_components
        .into_iter()
        .for_each(|tx_pc| {
            tx_pc
                .components
                .into_iter()
                .for_each(|pc| {
                    if let Some(pool_type) = pc.get_attribute_value("pool_type") {
                        if pool_type == "LiquidityBuffer".as_bytes() && pc.tokens.len() >= 2 {
                            let wrapped_token = hex::encode(&pc.tokens[0]);
                            let underlying_token = hex::encode(&pc.tokens[1]);
                            let mapping_key = format!("buffer_mapping_{}", wrapped_token);
                            store.set(0, mapping_key, &underlying_token);
                        }
                    }
                })
        });
}
