use substreams::{
    scalar::BigInt,
    store::{StoreSetSum, StoreSetSumBigInt},
};
use substreams_helper::hex::Hexable;

use crate::pb::ekubo::{ChangeType, SaleRateChanges};

#[substreams::handlers::store]
pub fn store_active_sale_rates(sale_rate_changes: SaleRateChanges, store: StoreSetSumBigInt) {
    sale_rate_changes
        .changes
        .into_iter()
        .for_each(|changes| {
            let pool_id = changes.pool_id.to_hex();

            match changes.change_type() {
                ChangeType::Delta => {
                    store.sum(
                        changes.ordinal,
                        format!("pool:{}:token0", pool_id),
                        BigInt::from_signed_bytes_be(&changes.token0_value),
                    );
                    store.sum(
                        changes.ordinal,
                        format!("pool:{}:token1", pool_id),
                        BigInt::from_signed_bytes_be(&changes.token1_value),
                    );
                }
                ChangeType::Absolute => {
                    store.set(
                        changes.ordinal,
                        format!("pool:{}:token0", pool_id),
                        BigInt::from_unsigned_bytes_be(&changes.token0_value),
                    );
                    store.set(
                        changes.ordinal,
                        format!("pool:{}:token1", pool_id),
                        BigInt::from_unsigned_bytes_be(&changes.token1_value),
                    );
                }
            }
        });
}
