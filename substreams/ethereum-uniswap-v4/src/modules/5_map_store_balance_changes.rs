use std::str::FromStr;

use anyhow::Ok;
use tycho_substreams::models::{BalanceDelta, BlockBalanceDeltas};

use crate::{
    modules::uni_math::calculate_token_amounts,
    pb::uniswap::v4::{
        events::{pool_event, PoolEvent},
        Events,
    },
};
use substreams::{
    prelude::{StoreGet, StoreGetInt64},
    scalar::BigInt,
    store::{StoreAddBigInt, StoreNew},
};

#[substreams::handlers::map]
pub fn map_balance_changes(
    events: Events,
    pools_current_tick_store: StoreGetInt64,
) -> Result<BlockBalanceDeltas, anyhow::Error> {
    let balance_deltas = events
        .pool_events
        .into_iter()
        .filter(PoolEvent::can_introduce_balance_changes)
        .map(|e| {
            (
                pools_current_tick_store
                    .get_at(e.log_ordinal, format!("pool:{0}", &e.pool_id))
                    .unwrap_or(0),
                e,
            )
        })
        .filter_map(|(current_tick, event)| event_to_balance_deltas(current_tick, event))
        .flatten()
        .collect();

    Ok(BlockBalanceDeltas { balance_deltas })
}

#[substreams::handlers::store]
pub fn store_pools_balances(balances_deltas: BlockBalanceDeltas, store: StoreAddBigInt) {
    tycho_substreams::balances::store_balance_changes(balances_deltas, store);
}

fn event_to_balance_deltas(current_tick: i64, event: PoolEvent) -> Option<Vec<BalanceDelta>> {
    let address = event.pool_id.as_bytes().to_vec();
    match event.r#type.unwrap() {
        pool_event::Type::ModifyLiquidity(e) => {
            let (delta0, delta1) =
                get_amount_delta(current_tick, e.tick_lower, e.tick_upper, e.liquidity_delta);
            Some(vec![
                BalanceDelta {
                    token: hex::decode(
                        event
                            .currency0
                            .clone()
                            .trim_start_matches("0x"),
                    )
                    .unwrap(),
                    delta: delta0.to_signed_bytes_be(),
                    component_id: address.clone(),
                    ord: event.log_ordinal,
                    tx: event
                        .transaction
                        .clone()
                        .map(Into::into),
                },
                BalanceDelta {
                    token: hex::decode(
                        event
                            .currency1
                            .clone()
                            .trim_start_matches("0x"),
                    )
                    .unwrap(),
                    delta: delta1.to_signed_bytes_be(),
                    component_id: address,
                    ord: event.log_ordinal,
                    tx: event.transaction.map(Into::into),
                },
            ])
        }

        pool_event::Type::Swap(e) => Some(vec![
            BalanceDelta {
                token: hex::decode(
                    event
                        .currency0
                        .clone()
                        .trim_start_matches("0x"),
                )
                .unwrap(),
                delta: BigInt::from_str(&e.amount0)
                    .unwrap()
                    .to_signed_bytes_be(),
                component_id: address.clone(),
                ord: event.log_ordinal,
                tx: event
                    .transaction
                    .clone()
                    .map(Into::into),
            },
            BalanceDelta {
                token: hex::decode(
                    event
                        .currency1
                        .clone()
                        .trim_start_matches("0x"),
                )
                .unwrap(),
                delta: BigInt::from_str(&e.amount1)
                    .unwrap()
                    .to_signed_bytes_be(),
                component_id: address.clone(),
                ord: event.log_ordinal,
                tx: event.transaction.map(Into::into),
            },
        ]),
        _ => None,
    }
}

impl PoolEvent {
    fn can_introduce_balance_changes(&self) -> bool {
        matches!(
            self.r#type.as_ref().unwrap(),
            pool_event::Type::ModifyLiquidity(_) | pool_event::Type::Swap(_)
        )
    }
}

fn get_amount_delta(
    current_tick: i64,
    tick_lower: i32,
    tick_upper: i32,
    liquidity_delta: String,
) -> (BigInt, BigInt) {
    let liquidity_delta: i128 = liquidity_delta
        .parse()
        .expect(" Failed to parse liquidity delta");
    let current_tick =
        TryInto::<i32>::try_into(current_tick).expect("Failed to convert current tick to i32");

    let (amount0, amount1) =
        calculate_token_amounts(current_tick, tick_lower, tick_upper, liquidity_delta)
            .expect("Failed to calculate token amounts from liquidity delta");
    (BigInt::from(amount0), BigInt::from(amount1))
}
