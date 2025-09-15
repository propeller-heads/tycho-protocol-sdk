use substreams::prelude::{StoreAddBigInt, StoreNew};
use tycho_substreams::models::BlockBalanceDeltas;

/// It's significant to include both the `pool_id` and the `token_id` for each balance delta as the
///  store key to ensure that there's a unique balance being tallied for each.
#[substreams::handlers::store]
pub fn store_balances(deltas: BlockBalanceDeltas, store: StoreAddBigInt) {
    tycho_substreams::balances::store_balance_changes(deltas, store);
}
