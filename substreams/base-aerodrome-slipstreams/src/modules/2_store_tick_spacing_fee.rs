use crate::pb::tycho::evm::aerodrome::TickSpacingFees;
use substreams::store::{StoreNew, StoreSetIfNotExists, StoreSetIfNotExistsInt64};

#[substreams::handlers::store]
pub fn store_tick_spacing_fee(tick_spacing_fees: TickSpacingFees, store: StoreSetIfNotExistsInt64) {
    for fee in tick_spacing_fees
        .tick_spacing_fees
        .iter()
    {
        store.set_if_not_exists(0, fee.tick_spacing.to_string(), &(fee.fee as i64));
    }
}
