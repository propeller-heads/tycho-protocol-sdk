use substreams::{
    scalar::BigInt,
    prelude::{StoreSet, StoreSetProto},
    store::StoreNew,
};
use crate::pb::cowamm::{CowPool, CowPools};

#[substreams::handlers::store]
pub fn store_cowpools(pools_output: CowPools, store: StoreSetProto<CowPool>) {
    for pool in pools_output.pools.iter() {
        store.set(0, hex::encode(&pool.created_tx_hash), pool);
    }
}

