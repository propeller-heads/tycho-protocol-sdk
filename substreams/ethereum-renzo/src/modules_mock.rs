use crate::abi::{self, restake_manager_contract::events::Deposit};
use anyhow::{Ok, Result};
use std::collections::HashMap;
use substreams::{
    hex,
    pb::substreams::StoreDeltas,
    scalar::BigInt as ScalarBigInt,
    store::{StoreAdd, StoreAddBigInt, StoreAddInt64, StoreGet, StoreGetInt64, StoreNew},
};
use substreams_ethereum::{
    pb::eth::{self},
    Event,
};
use tycho_substreams::{
    balances::aggregate_balances_changes, contract::extract_contract_changes, prelude::*,
};

// hex!("ae7ab96520DE3A18E5e111B5EaAb095312D7fE84") stETH
// hex!("a2E3356610840701BDf5611a53974510Ae27E2e1") wBETH
// hex!("0000000000000000000000000000000000000000") ETH
// hex!("bf5495Efe5DB9ce00f80364C8B423567e58d2110") ezETH

/// Ethereum native token address representation
pub const ETH_ADDRESS: [u8; 20] = hex!("eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee");

/// First deposit signature (without referralId)
/// Deployed at block 18727859
/// Example TX: 0x3c8188f4d87fcb79901082f88f4818fc6d5449728b44c7872e9381cd5f99d38b
pub const FIRST_DEPOSIT_SIG: [u8; 32] =
    hex!("dcbc1c05240f31ff3ad067ef1ee35ce4997762752e3a095284754544f4c709d7");

// Domenico Shadowy, [26 Nov 2024 at 14:48:06]:
// 0xdcbc1c05240f31ff3ad067ef1ee35ce4997762752e3a095284754544f4c709d7
// 0x4e2ca0515ed1aef1395f66b5303bb5d6f1bf9d61a353fa53f73f8ac9973fa9f6

/// Second deposit signature (with referralId)
/// Deployed at block 19164302
/// Example TX: 0x7d40c574c44e441a0c851d8d3a5fb186127fa5675405629bef91983fcccfeb01
pub const SECOND_DEPOSIT_SIG: [u8; 32] =
    hex!("4e2ca0515ed1aef1395f66b5303bb5d6f1bf9d61a353fa53f73f8ac9973fa9f6");

/// Map to extract protocol components for transactions in a block
#[substreams::handlers::map]
pub fn map_components(
    params: String,
    block: eth::v2::Block,
) -> Result<BlockTransactionProtocolComponents, anyhow::Error> {
    let component_address =
        hex::decode(params).map_err(|e| anyhow::anyhow!("Failed to decode params: {}", e))?;

    let component_token = find_deployed_underlying_address(&component_address)
        .ok_or_else(|| anyhow::anyhow!("Failed to find deployed underlying address"))?;

    // We store these as a hashmap by tx hash since we need to agg by tx hash later
    Ok(BlockTransactionProtocolComponents {
        tx_components: block
            .transactions()
            .filter_map(|tx| {
                let components = tx
                    .calls()
                    .filter(|call| !call.call.state_reverted)
                    .filter_map(|call| {
                        // address doesn't exist before contract deployment, hence the first tx with
                        // a log.address = component_address is the deployment tx
                        if is_deployment_call(call.call, &component_address) {
                            Some(
                                ProtocolComponent::at_contract(&component_address, &tx.into())
                                    .with_tokens(&[
                                        component_token.as_slice(),
                                        ETH_ADDRESS.as_slice(),
                                    ])
                                    .as_swap_type("restake_manager", ImplementationType::Vm),
                            )
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();

                if !components.is_empty() {
                    Some(TransactionProtocolComponents { tx: Some(tx.into()), components })
                } else {
                    None
                }
            })
            .collect::<Vec<_>>(),
    })
}

/// Store protocol components and associated tokens
#[substreams::handlers::store]
pub fn store_components(map: BlockTransactionProtocolComponents, store: StoreAddInt64) {
    let components: Vec<String> = map
        .tx_components
        .iter()
        .flat_map(|tx_components| &tx_components.components)
        .map(|component| format!("pool:{0}", component.id))
        .collect();

    if !components.is_empty() {
        store.add_many(
            map.tx_components
                .first()
                .and_then(|tx| tx.tx.as_ref())
                .map(|tx| tx.index)
                .unwrap_or(0),
            &components,
            1,
        );
    }
}

/// Map balance changes caused by deposit and withdrawal events
#[substreams::handlers::map]
pub fn map_relative_balances(
    block: eth::v2::Block,
    store: StoreGetInt64,
) -> Result<BlockBalanceDeltas, anyhow::Error> {
    let mut balance_deltas = Vec::new();

    // Log block processing start
    let logs: Vec<_> = block.logs().collect();

    for log in logs {

        // Try to decode as deposit
        // if let Some(ev) = decode_renzo_deposit(log.log) {
        if 1 == 1 {
            let address_hex = format!("0x{}", hex::encode(log.log.address.clone()));

            // Check if this is a tracked component
            let store_key = format!("pool:{0}", address_hex);

            if let Some(component) = store.get_last(&store_key) {
                let ez_eth_address = find_deployed_underlying_address(&log.log.address).unwrap();

                let deposit_mock: Deposit = Deposit {
                    depositor: log.log.address.clone(),
                    token: ez_eth_address.clone().to_vec(),
                    amount: ScalarBigInt::from(333),
                    ez_eth_minted: ScalarBigInt::from(333),
                    referral_id: ScalarBigInt::from(333),
                };

                substreams::log::debug!(
                    "Pushing token:{} to:{} value:{}", hex::encode(deposit_mock.token.clone()), address_hex,deposit_mock.amount
                );

                balance_deltas.push(BalanceDelta {
                    ord: log.ordinal(),
                    tx: Some(log.receipt.transaction.into()),
                    token: deposit_mock.token.to_vec(),
                    delta: deposit_mock.amount.to_signed_bytes_be(),
                    component_id: address_hex.as_bytes().to_vec(),
                });
            }
        } else if let Some(ev) =
            abi::restake_manager_contract::events::UserWithdrawCompleted::match_and_decode(log.log)
        {
            let address_hex = format!("0x{}", hex::encode(log.log.address.clone()));

            substreams::log::info!(
                "Found withdrawal event: address={}, token={}, amount={}, ez_eth_burned={}",
                address_hex,
                hex::encode(&ev.token),
                ev.amount,
                ev.ez_eth_burned
            );

            if let Some(_component_key) = store.get_last(format!("pool:{0}", address_hex)) {
                let ez_eth_address = hex!("bf5495Efe5DB9ce00f80364C8B423567e58d2110").to_vec();

                // Log withdrawal delta creation
                substreams::log::info!(
                    "Creating withdrawal delta: token={}, amount=-{}, component_id={}",
                    hex::encode(&ev.token),
                    ev.amount,
                    hex::encode(&ev.token)
                );

                // Handle the balance delta for withdrawn token
                balance_deltas.push(BalanceDelta {
                    ord: log.log.ordinal,
                    tx: Some(log.receipt.transaction.into()),
                    token: ev.token.clone(),
                    delta: ev.amount.neg().to_signed_bytes_be(),
                    component_id: ev.token.to_vec(),
                });

                // Log ezETH burning delta creation
                substreams::log::info!(
                    "Creating ezETH burn delta: token={}, amount=-{}, component_id={}",
                    hex::encode(&ez_eth_address),
                    ev.ez_eth_burned,
                    hex::encode(&ez_eth_address)
                );

                // Handle the balance delta for burned ezETH
                balance_deltas.push(BalanceDelta {
                    ord: log.ordinal(),
                    tx: Some(log.receipt.transaction.into()),
                    token: ez_eth_address.clone(),
                    delta: ev
                        .ez_eth_burned
                        .neg()
                        .to_signed_bytes_be(),
                    component_id: ez_eth_address,
                });
            }
        }
    }

    // Log summary of processed deltas
    substreams::log::info!(
        "Block {} processing complete. Created {} balance deltas",
        block.number,
        balance_deltas.len()
    );

    Ok(BlockBalanceDeltas { balance_deltas })
}

/// Store aggregated balance changes
#[substreams::handlers::store]
pub fn store_balances(deltas: BlockBalanceDeltas, store: StoreAddBigInt) {
    tycho_substreams::balances::store_balance_changes(deltas, store);
}

/// Map protocol changes across transactions and balances
#[substreams::handlers::map]
pub fn map_protocol_changes(
    block: eth::v2::Block,
    grouped_components: BlockTransactionProtocolComponents,
    deltas: BlockBalanceDeltas,
    components_store: StoreGetInt64,
    balance_store: StoreDeltas,
) -> Result<BlockChanges, anyhow::Error> {
    let mut transaction_contract = HashMap::new();

    for tx_component in &grouped_components.tx_components {
        let tx = tx_component.tx.as_ref().unwrap();
        transaction_contract
            .entry(tx.index)
            .or_insert_with(|| TransactionChanges::new(tx))
            .component_changes
            .extend(tx_component.components.clone());
    }

    aggregate_balances_changes(balance_store, deltas)
        .into_iter()
        .for_each(|(_, (tx, balances))| {
            let tx_change = transaction_contract
                .entry(tx.index)
                .or_insert_with(|| TransactionChanges::new(&tx));

            tx_change.balance_changes.extend(
                balances
                    .into_values()
                    .flat_map(|map| map.into_values()),
            );
        });

    extract_contract_changes(
        &block,
        |addr| {
            components_store
                .get_last(format!("pool:0x{0}", hex::encode(addr)))
                .is_some()
        },
        &mut transaction_contract,
    );

    Ok(BlockChanges {
        block: Some((&block).into()),
        changes: transaction_contract
            .into_values()
            .filter(|change| !change.component_changes.is_empty())
            .collect(),
    })
}

/// Determine if a transaction deploys the Restake Manager
fn is_deployment_call(call: &eth::v2::Call, component_address: &[u8]) -> bool {
    call.account_creations
        .iter()
        .any(|ac| ac.account.as_slice() == component_address)
}

fn find_deployed_underlying_address(component_address: &[u8]) -> Option<[u8; 20]> {
    let result = match component_address {
        hex!("74a09653A083691711cF8215a6ab074BB4e99ef5") => {
            Some(hex!("bf5495Efe5DB9ce00f80364C8B423567e58d2110"))
        }
        _ => {
            substreams::log::info!(
                "Unknown component address: 0x{}",
                hex::encode(component_address)
            );
            None
        }
    };
    result
}

// Deposit signatures
// FIRST_DEPOSIT_SIG
// 0xdcbc1c05240f31ff3ad067ef1ee35ce4997762752e3a095284754544f4c709d7
// block: 18727859
// tx hash: 0x3c8188f4d87fcb79901082f88f4818fc6d5449728b44c7872e9381cd5f99d38b
// Asset: ETH

// SECOND_DEPOSIT_SIG
// 0x4e2ca0515ed1aef1395f66b5303bb5d6f1bf9d61a353fa53f73f8ac9973fa9f6
// 0x4e2ca0515ed1aef1395f66b5303bb5d6f1bf9d61a353fa53f73f8ac9973fa9f6
// block: 19164302
// tx hash: 0x7d40c574c44e441a0c851d8d3a5fb186127fa5675405629bef91983fcccfeb01
// Asset: wBETH

// Determine if the log is a deposit of old Renzo's Deposit signature excl. referralId
pub fn is_deposit_type_1(log: &eth::v2::Log) -> bool {
    log.topics
        .first()
        .map(|topic| *topic == FIRST_DEPOSIT_SIG)
        .unwrap_or(false)
}

// Determine if the log is a deposit of new Renzo's Deposit signature incl. referralId
pub fn is_deposit_type_2(log: &eth::v2::Log) -> bool {
    log.topics
        .first()
        .map(|topic| *topic == SECOND_DEPOSIT_SIG)
        .unwrap_or(false)
}

/// Decodes a Renzo deposit event from an Ethereum log
///
/// # Arguments
/// * `log` - The Ethereum log to decode
///
/// # Returns
/// * `Option<Deposit>` - The decoded deposit event, or None if the log is not a valid deposit
///
/// # Type 1 Format (Old)
/// * topics[0]: FIRST_DEPOSIT_SIG
/// * topics[1]: depositor address
/// * topics[2]: token address
/// * topics[3]: amount
/// * topics[4]: ez_eth_minted
///
/// # Type 2 Format (New)
/// * topics[0]: SECOND_DEPOSIT_SIG
/// * topics[1]: depositor address
/// * topics[2]: token address
/// * topics[3]: amount
/// * topics[4]: ez_eth_minted
/// * topics[5]: referral_id
pub fn decode_renzo_deposit(log: &eth::v2::Log) -> Option<Deposit> {
    // Validate minimum topic length
    if log.topics.len() < 3 {
        return None;
    }

    let is_type_1 = is_deposit_type_1(log);
    let is_type_2 = is_deposit_type_2(log);

    if !is_type_1 && !is_type_2 {
        return None;
    }

    // Validate required topic length for each type
    // if (is_type_1 && log.topics.len() < 5) || (is_type_2 && log.topics.len() < 6) {
    //     substreams::log::info!(
    //         "Invalid number of topics for deposit type: {} topics, type1: {}, type2: {}",
    //         log.topics.len(),
    //         is_type_1,
    //         is_type_2
    //     );
    //     return None;
    // }

    // Common fields for both types
    let depositor = log
        .topics
        .get(1)
        .cloned()
        .unwrap_or_default();
    let token = log
        .topics
        .get(2)
        .cloned()
        .unwrap_or_default();

    if is_type_1 {
        // Type 1 has amount and ez_eth_minted in topics[3] and topics[4]
        Some(Deposit {
            depositor,
            token,
            amount: ScalarBigInt::from_unsigned_bytes_be(
                &log.topics
                    .get(3)
                    .cloned()
                    .unwrap_or_default(),
            ),
            ez_eth_minted: ScalarBigInt::from_unsigned_bytes_be(
                &log.topics
                    .get(4)
                    .cloned()
                    .unwrap_or_default(),
            ),
            referral_id: ScalarBigInt::zero(), // Type 1 doesn't have referral_id
        })
    } else {
        // Type 2 has referral_id as an additional field
        Some(Deposit {
            depositor,
            token,
            amount: ScalarBigInt::from_unsigned_bytes_be(
                &log.topics
                    .get(3)
                    .cloned()
                    .unwrap_or_default(),
            ),
            ez_eth_minted: ScalarBigInt::from_unsigned_bytes_be(
                &log.topics
                    .get(4)
                    .cloned()
                    .unwrap_or_default(),
            ),
            referral_id: ScalarBigInt::from_unsigned_bytes_be(
                &log.topics
                    .get(5)
                    .cloned()
                    .unwrap_or_default(),
            ),
        })
    }
}