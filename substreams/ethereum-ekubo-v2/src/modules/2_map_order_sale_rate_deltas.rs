use substreams::scalar::BigInt;

use crate::{
    pb::ekubo::{
        block_transaction_events::transaction_events::pool_log::Event, BlockTransactionEvents,
        OrderSaleRateDelta, OrderSaleRateDeltas,
    },
    twamm::sale_rate_deltas_from_order_update,
};

#[substreams::handlers::map]
pub fn map_order_sale_rate_deltas(block_tx_events: BlockTransactionEvents) -> OrderSaleRateDeltas {
    OrderSaleRateDeltas {
        deltas: block_tx_events
            .block_transaction_events
            .into_iter()
            .flat_map(|tx_events| {
                let tx = tx_events.transaction;

                tx_events
                    .pool_logs
                    .into_iter()
                    .flat_map(move |log| {
                        let tx = tx.clone();

                        order_sale_rate_deltas(log.event.unwrap())
                            .into_iter()
                            .map(move |partial| OrderSaleRateDelta {
                                pool_id: log.pool_id.clone(),
                                time: partial.time,
                                sale_rate_delta0: partial.sale_rate_delta0,
                                sale_rate_delta1: partial.sale_rate_delta1,
                                ordinal: log.ordinal,
                                transaction: tx.clone(),
                            })
                    })
            })
            .collect(),
    }
}

struct PartialOrderSaleRateDelta {
    time: u64,
    sale_rate_delta0: Vec<u8>,
    sale_rate_delta1: Vec<u8>,
}

fn order_sale_rate_deltas(ev: Event) -> Vec<PartialOrderSaleRateDelta> {
    match ev {
        Event::OrderUpdated(ev) => {
            let (sale_rate_delta0, sale_rate_delta1) = sale_rate_deltas_from_order_update(&ev);

            let (start_time, end_time) = {
                let key = ev.order_key.unwrap();
                (key.start_time, key.end_time)
            };

            vec![
                PartialOrderSaleRateDelta {
                    time: end_time,
                    sale_rate_delta0: BigInt::from_signed_bytes_be(&sale_rate_delta0)
                        .neg()
                        .to_signed_bytes_be(),
                    sale_rate_delta1: BigInt::from_signed_bytes_be(&sale_rate_delta1)
                        .neg()
                        .to_signed_bytes_be(),
                },
                PartialOrderSaleRateDelta { time: start_time, sale_rate_delta0, sale_rate_delta1 },
            ]
        }
        _ => vec![],
    }
}
