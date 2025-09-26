use crate::pb::cowamm::CowPoolBinds;
use serde_json::json;
use substreams::store::{Appender, StoreAppend, StoreNew};

#[substreams::handlers::store]
pub fn store_cowpool_binds(binds: CowPoolBinds, store: StoreAppend<String>) {
    for bind in binds.binds.iter() {
        let pool_key = hex::encode(&bind.address);
        // Format the bind as a JSON string
        let bind_string = serde_json::json!({
            "address": hex::encode(&bind.address),
            "token": hex::encode(&bind.token),
            "weight": hex::encode(&bind.weight),
        })
        .to_string();
        store.append(0, pool_key, bind_string);
    }
}
