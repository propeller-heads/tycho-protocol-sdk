use tycho_substreams::prelude::*;
use substreams::store::{StoreNew, StoreAddBigInt};

#[substreams::handlers::store]
pub fn store_balances(_deltas: BlockBalanceDeltas, _store: StoreAddBigInt) {
    // tycho_substreams::balances::store_balance_changes(deltas, store);
}
