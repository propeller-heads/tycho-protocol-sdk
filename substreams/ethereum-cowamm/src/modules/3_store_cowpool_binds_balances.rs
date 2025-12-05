use substreams::store::{StoreAddBigInt, StoreNew};
use tycho_substreams::prelude::*;

#[substreams::handlers::store] 
//its one bind at a time so we don't even need StoreAddBigInt 
pub fn store_cowpool_binds_balances(binds_balances_deltas: BlockBalanceDeltas, store: StoreAddBigInt) {
    tycho_substreams::balances::store_balance_changes(binds_balances_deltas, store);
}
