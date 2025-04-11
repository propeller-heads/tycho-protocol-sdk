use crate::{
    pb::ekubo::{
        block_transaction_events::transaction_events::{pool_log::Event, PoolLog},
        BlockTransactionEvents, ChangeType, SaleRateChange, SaleRateChanges,
    },
    twamm::sale_rate_deltas_from_order_update,
};

#[substreams::handlers::map]
pub fn map_sale_rate_changes(block_tx_events: BlockTransactionEvents) -> SaleRateChanges {
    SaleRateChanges {
        changes: block_tx_events
            .block_transaction_events
            .into_iter()
            .flat_map(|tx_events| {
                tx_events
                    .pool_logs
                    .into_iter()
                    .filter_map(move |log| {
                        maybe_sale_rate_change(&log, block_tx_events.timestamp).map(|partial| {
                            SaleRateChange {
                                change_type: partial.change_type.into(),
                                pool_id: log.pool_id,
                                token0_value: partial.token0_value,
                                token1_value: partial.token1_value,
                                ordinal: log.ordinal,
                                transaction: tx_events.transaction.clone(),
                            }
                        })
                    })
            })
            .collect(),
    }
}

struct PartialSaleRateChange {
    token0_value: Vec<u8>,
    token1_value: Vec<u8>,
    change_type: ChangeType,
}

fn maybe_sale_rate_change(log: &PoolLog, timestamp: u64) -> Option<PartialSaleRateChange> {
    match log.event.as_ref().unwrap() {
        Event::VirtualOrdersExecuted(ev) => Some(PartialSaleRateChange {
            change_type: ChangeType::Absolute,
            token0_value: ev.token0_sale_rate.clone(),
            token1_value: ev.token1_sale_rate.clone(),
        }),
        Event::OrderUpdated(ev) => {
            // A virtual order execution always happens before an order update
            let last_execution_time = timestamp;

            let (token0_sale_rate_delta, token1_sale_rate_delta) =
                sale_rate_deltas_from_order_update(ev);
            let key = ev.order_key.as_ref().unwrap();

            (last_execution_time >= key.start_time && last_execution_time < key.end_time).then_some(
                PartialSaleRateChange {
                    change_type: ChangeType::Delta,
                    token0_value: token0_sale_rate_delta,
                    token1_value: token1_sale_rate_delta,
                },
            )
        }
        _ => None,
    }
}
