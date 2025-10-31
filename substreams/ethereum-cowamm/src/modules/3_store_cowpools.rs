use substreams::{
    prelude::{StoreSet, StoreSetProto},
    store::StoreNew,
};
use crate::pb::cowamm::{CowPool, CowPools};
use substreams_helper::{hex::Hexable};

#[substreams::handlers::store]
pub fn store_cowpools(pools_output: CowPools, store: StoreSetProto<CowPool>) {
    for pool in pools_output.pools.iter() {
        store.set(0, format!("Pool:0x{}", hex::encode(&pool.address)), pool);
    }
}
