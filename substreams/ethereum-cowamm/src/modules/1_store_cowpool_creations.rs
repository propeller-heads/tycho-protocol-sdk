use substreams::{
    scalar::BigInt,
    prelude::{StoreSetIfNotExists, StoreSetIfNotExistsProto},
    store::StoreNew,
};
use crate::pb::cowamm::{CowPoolCreation, CowPoolCreations};

#[substreams::handlers::store]
pub fn store_cowpool_creations(creations: CowPoolCreations, store: StoreSetIfNotExistsProto<CowPoolCreation>) {
    for creation in creations.pools.iter() {
        store.set_if_not_exists(0, hex::encode(&creation.address), creation);
    }
}
