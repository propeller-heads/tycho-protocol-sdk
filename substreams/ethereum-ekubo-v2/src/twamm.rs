use substreams::scalar::BigInt;

use crate::pb::ekubo::block_transaction_events::transaction_events::pool_log::OrderUpdated;

pub fn sale_rate_deltas_from_order_update(ev: &OrderUpdated) -> (Vec<u8>, Vec<u8>) {
    let key = ev.order_key.as_ref().unwrap();
    let zero = BigInt::zero().to_bytes_be().1;

    if key.sell_token > key.buy_token {
        (zero, ev.sale_rate_delta.clone())
    } else {
        (ev.sale_rate_delta.clone(), zero)
    }
}
