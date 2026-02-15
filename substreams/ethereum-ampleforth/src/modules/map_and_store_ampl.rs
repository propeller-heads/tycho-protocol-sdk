use anyhow::Result;
use std::str::FromStr;
use substreams::{
    scalar::BigInt,
    store::{
        StoreAdd, StoreAddBigInt, StoreGet, StoreGetBigInt, StoreNew, StoreSet, StoreSetBigInt,
    },
};
use substreams_ethereum::{
    pb::eth::{self},
    Event,
};

use crate::abi;
use crate::pb::ampleforth::{AmplGonBalanceDelta, AmplGonBalanceDeltas, AmplRebases, RebaseEvent};

// --------------
// Constants
// --------------
// NOTE: These pools are singletons and will only exist on Ethreum mainnet.
// We thus hardcode all addresses.

// AMPL is a rebasing token. It's balance scales up and down based on a daily "rebase" operation.
// The contract stores balances using an internal representation called "gons".
// The user's external AMPL balance is calculated by multiplying "gons", by a scalar stored in the contract.
// The scalar value in the contract's storage is updated on every rebase.
//
// We keep track of all the internal gon balance of all the interested pools, and the scalar
// to then be able to compute it's actual AMPL balance in any given block.

pub const AMPL_ADDRESS_HEX: &str = "d46ba6d942050d489dbd938a2c909a5d5039a161";
pub const ZERO_ADDRESS_HEX: &str = "0000000000000000000000000000000000000000";
pub const WAMPL_ADDRESS_HEX: &str = "edb171c18ce90b633db442f2a6f72874093b49ef";

pub const INTERESTED_POOLS: [&str; 1] = [WAMPL_ADDRESS_HEX];
pub const INTERESTED_TOKENS: [&str; 2] = [AMPL_ADDRESS_HEX, WAMPL_ADDRESS_HEX];

pub const AMPL_STORE_REF: &str = "ampl";
pub const AMPL_GON_SUPPLY: &str =
    "115792089237316195423570985008687907853269984665640564039457550000000000000000";
pub const AMPL_INIT_SUPPLY: &str = "129716368304480471"; // AMPL supply when WAMPL pool was initialized..

// --------------
// Keep track of AMPL rebases
// --------------
#[substreams::handlers::map]
pub fn map_ampl_rebases(block: eth::v2::Block) -> Result<AmplRebases> {
    let mut rebase_events = vec![];
    for tx_log in block.logs() {
        let pool_addr = tx_log.address();
        let pool_addr_hex = hex::encode(pool_addr);
        if pool_addr_hex != AMPL_ADDRESS_HEX {
            continue;
        }
        if let Some(ev) = abi::ampl_contract::events::LogRebase::match_and_decode(tx_log.log) {
            let epoch_str = ev.epoch.to_string();
            let total_supply_str = ev.total_supply.to_string();
            substreams::log::debug!("rebase:{}:{}", epoch_str, total_supply_str,);
            rebase_events.push(RebaseEvent {
                ordinal: tx_log.ordinal(),
                epoch: epoch_str,
                total_supply: total_supply_str,
            });
        }
    }
    Ok(AmplRebases { rebase_events })
}

#[substreams::handlers::store]
pub fn store_ampl_supply(rebases: AmplRebases, ampl_supply_store: StoreSetBigInt) {
    for rebase_event in rebases.rebase_events {
        let epoch = BigInt::from_str(&rebase_event.epoch).unwrap_or_else(|_| BigInt::zero());
        let total_supply =
            BigInt::from_str(&rebase_event.total_supply).unwrap_or_else(|_| BigInt::zero());
        ampl_supply_store.set(0, format!("{}:epoch", AMPL_STORE_REF), &epoch);
        ampl_supply_store.set(0, format!("{}:total_supply", AMPL_STORE_REF), &total_supply);
        ampl_supply_store.set(
            0,
            format!("{}:{}:total_supply", AMPL_STORE_REF, rebase_event.epoch),
            &total_supply,
        );
    }
}

// --------------
// Keep track of internal AMPL balances (gons) of interested pools
// --------------
#[substreams::handlers::map]
pub fn map_ampl_gon_balances(
    block: eth::v2::Block,
    ampl_supply_store: StoreGetBigInt,
) -> Result<AmplGonBalanceDeltas> {
    let mut balance_deltas = Vec::new();
    for tx_log in block.logs() {
        if let Some(ev) = abi::ampl_contract::events::Transfer::match_and_decode(tx_log.log) {
            let token_addr = tx_log.address();
            let token_hex = hex::encode(token_addr);
            let from_hex = hex::encode(ev.from);
            let to_hex = hex::encode(ev.to);

            let is_ampl_tx = token_hex == AMPL_ADDRESS_HEX;
            let is_token_out = INTERESTED_POOLS.contains(&from_hex.as_str());
            let is_token_in = INTERESTED_POOLS.contains(&to_hex.as_str());

            let ampl_supply = fetch_current_ampl_supply(&ampl_supply_store);
            let delta_value = compute_gon_balance(&ev.value, &ampl_supply);

            if is_ampl_tx && is_token_out {
                balance_deltas.push(AmplGonBalanceDelta {
                    ordinal: tx_log.ordinal(),
                    delta: delta_value.clone().neg().to_string(),
                    component_id: from_hex.as_bytes().to_vec(),
                });
            }

            if is_ampl_tx && is_token_in {
                balance_deltas.push(AmplGonBalanceDelta {
                    ordinal: tx_log.ordinal(),
                    delta: delta_value.to_string(),
                    component_id: to_hex.as_bytes().to_vec(),
                });
            }
        }
    }
    Ok(AmplGonBalanceDeltas { balance_deltas })
}

#[substreams::handlers::store]
pub fn store_ampl_gon_balances(
    balance_deltas: AmplGonBalanceDeltas,
    ampl_gon_balance_store: StoreAddBigInt,
) {
    for delta in balance_deltas.balance_deltas {
        let delta_value = BigInt::from_str(&delta.delta).unwrap_or_else(|_| BigInt::zero());
        let gon_balance_key = format!(
            "{}:{}",
            AMPL_STORE_REF,
            String::from_utf8(delta.component_id.clone())
                .expect("delta.component_id is not valid utf-8!")
        );
        ampl_gon_balance_store.add(0, gon_balance_key, &delta_value);
    }
}

// --------------
// Store query methods
// --------------
pub fn fetch_balance_delta_after_rebase(
    ampl_supply_store: &StoreGetBigInt,
    ampl_gon_balance_store: &StoreGetBigInt,
    addr: &str,
) -> BigInt {
    let curr_balance = fetch_current_balance(ampl_supply_store, ampl_gon_balance_store, addr);
    let prev_balance = fetch_prev_balance(ampl_supply_store, ampl_gon_balance_store, addr);
    substreams::log::debug!(
        "rebase scaling: curr_balance={}, prev_balance={}",
        curr_balance,
        prev_balance
    );
    curr_balance - prev_balance
}

fn fetch_current_balance(
    ampl_supply_store: &StoreGetBigInt,
    ampl_gon_balance_store: &StoreGetBigInt,
    addr: &str,
) -> BigInt {
    let curr_supply = fetch_current_ampl_supply(ampl_supply_store);
    let gon_balance = fetch_gon_balance(ampl_gon_balance_store, addr);
    compute_ampl_balance(&gon_balance, &curr_supply)
}

fn fetch_prev_balance(
    ampl_supply_store: &StoreGetBigInt,
    ampl_gon_balance_store: &StoreGetBigInt,
    addr: &str,
) -> BigInt {
    let prev_supply = fetch_prev_ampl_supply(ampl_supply_store);
    let gon_balance = fetch_gon_balance(ampl_gon_balance_store, addr);
    compute_ampl_balance(&gon_balance, &prev_supply)
}

fn compute_ampl_balance(gon_balance: &BigInt, ampl_supply: &BigInt) -> BigInt {
    let gon_supply =
        BigInt::from_str(AMPL_GON_SUPPLY).expect("AMPL_GON_SUPPLY is not a valid BigInt");
    let gons_per_ampl = gon_supply / ampl_supply;
    let ampl_balance = gon_balance / gons_per_ampl;
    ampl_balance
}

fn compute_gon_balance(ampl_balance: &BigInt, ampl_supply: &BigInt) -> BigInt {
    let gon_supply =
        BigInt::from_str(AMPL_GON_SUPPLY).expect("AMPL_GON_SUPPLY is not a valid BigInt");
    let gons_per_ampl = gon_supply / ampl_supply;
    let gon_balance = ampl_balance * gons_per_ampl;
    gon_balance
}

fn fetch_gon_balance(ampl_gon_balance_store: &StoreGetBigInt, addr: &str) -> BigInt {
    let component_id = addr.as_bytes().to_vec();
    let gon_balance_key = format!(
        "{}:{}",
        AMPL_STORE_REF,
        String::from_utf8(component_id.clone()).expect("delta.component_id is not valid utf-8!")
    );
    let gon_balance = ampl_gon_balance_store
        .get_last(gon_balance_key)
        .unwrap_or_else(BigInt::zero);
    gon_balance
}

fn fetch_current_ampl_supply(ampl_supply_store: &StoreGetBigInt) -> BigInt {
    let mut ampl_supply = ampl_supply_store
        .get_last(format!("{}:total_supply", AMPL_STORE_REF))
        .unwrap_or_default();
    if ampl_supply.is_zero() {
        ampl_supply = BigInt::from_str(AMPL_INIT_SUPPLY).expect("Unable to parse default");
    }
    substreams::log::debug!("curr_supply={}", ampl_supply);
    ampl_supply
}

fn fetch_prev_ampl_supply(ampl_supply_store: &StoreGetBigInt) -> BigInt {
    let epoch = ampl_supply_store
        .get_last(format!("{}:epoch", AMPL_STORE_REF))
        .unwrap_or_default();
    let mut ampl_supply = ampl_supply_store
        .get_last(format!("{}:{}:total_supply", AMPL_STORE_REF, epoch - 1))
        .unwrap_or_default();
    if ampl_supply.is_zero() {
        ampl_supply = BigInt::from_str(AMPL_INIT_SUPPLY).expect("Unable to parse default");
    }
    substreams::log::debug!("prev_supply={}", ampl_supply);
    ampl_supply
}
