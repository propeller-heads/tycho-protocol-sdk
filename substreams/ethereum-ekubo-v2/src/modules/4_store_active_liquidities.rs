use substreams::{
    scalar::BigInt,
    store::{StoreSetSum, StoreSetSumBigInt},
};
use substreams_helper::hex::Hexable;

use crate::pb::ekubo::{ChangeType, LiquidityChanges};

#[substreams::handlers::store]
pub fn store_active_liquidities(liquidity_changes: LiquidityChanges, store: StoreSetSumBigInt) {
    liquidity_changes
        .changes
        .into_iter()
        .for_each(|changes| match changes.change_type() {
            ChangeType::Delta => {
                store.sum(
                    changes.ordinal,
                    format!("pool:{}", changes.pool_id.to_hex()),
                    BigInt::from_signed_bytes_be(&changes.value),
                );
            }
            ChangeType::Absolute => {
                store.set(
                    changes.ordinal,
                    format!("pool:{}", changes.pool_id.to_hex()),
                    BigInt::from_unsigned_bytes_be(&changes.value),
                );
            }
        });
}
