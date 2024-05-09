use crate::abi;
use anyhow::Context;
use itertools::Itertools;
use serde::Deserialize;
use std::collections::HashMap;
use substreams::{
    pb::substreams::StoreDeltas,
    store::{StoreAddBigInt, StoreNew},
};
use substreams_ethereum::pb::eth;
use tycho_substreams::{
    balances::aggregate_balances_changes, contract::extract_contract_changes, prelude::*,
};

#[derive(Debug, Deserialize)]
struct Params {
    creation_block_nos: Vec<u64>,
    creation_hashes: Vec<String>,
    proxies: Vec<String>,
    stablecoins: Vec<String>,
    anglecoins: Vec<String>,
}

trait ConvertToAngleProxyDefinition {
    fn convert(&self) -> Vec<AngleProxyDefinition>;
}

impl ConvertToAngleProxyDefinition for Params {
    fn convert(&self) -> Vec<AngleProxyDefinition> {
        let mut angle_proxy_definitions = Vec::new();
        for i in 0..self.creation_block_nos.len() {
            let creation_block_no = self.creation_block_nos[i];
            let creation_hash =
                hex::decode(&self.creation_hashes[i]).expect("Failed to decode creation hash");
            let proxy = hex::decode(&self.proxies[i]).expect("Failed to decode proxy address");
            let stablecoin =
                hex::decode(&self.stablecoins[i]).expect("Failed to decode stablecoin address");
            let anglecoin =
                hex::decode(&self.anglecoins[i]).expect("Failed to decode anglecoin address");

            let angle_proxy_definition = AngleProxyDefinition {
                creation_block_no,
                creation_hash: {
                    let mut arr = [0u8; 32];
                    arr.copy_from_slice(&creation_hash);
                    arr
                },
                proxy: {
                    let mut arr = [0u8; 20];
                    arr.copy_from_slice(&proxy);
                    arr
                },
                stablecoin: {
                    let mut arr = [0u8; 20];
                    arr.copy_from_slice(&stablecoin);
                    arr
                },
                anglecoin: {
                    let mut arr = [0u8; 20];
                    arr.copy_from_slice(&anglecoin);
                    arr
                },
            };

            angle_proxy_definitions.push(angle_proxy_definition);
        }

        angle_proxy_definitions
    }
}

#[derive(Debug, Deserialize)]
struct AngleProxyDefinition {
    creation_block_no: u64,
    creation_hash: [u8; 32],
    proxy: [u8; 20],
    stablecoin: [u8; 20],
    anglecoin: [u8; 20],
}

fn parse_params(params: &str) -> Result<Params, anyhow::Error> {
    serde_qs::from_str(params).context("Failed to parse params")
}

/// Maps the `Redeemed` and `Swap` events to `BalanceDelta`s representing the Redemptions, Mints,
///  and burns by the transmuter.
#[substreams::handlers::map]
pub fn map_relative_balances(
    raw_params: String,
    block: eth::v2::Block,
) -> Result<BlockBalanceDeltas, anyhow::Error> {
    let params = parse_params(&raw_params)?.convert();

    let mut balance_deltas: Vec<BalanceDelta> = vec![];

    params.iter().for_each(|param| {
        balance_deltas.extend(
            block
                .events::<abi::redeemer::events::Redeemed>(&[&param.proxy])
                .flat_map(|(event, log)| {
                    event
                        .tokens
                        .into_iter()
                        .map(move |token| BalanceDelta {
                            ord: log.ordinal(),
                            tx: Some(log.receipt.transaction.into()),
                            token,
                            delta: event.amount.to_signed_bytes_be(),
                            component_id: hex::encode(param.proxy).into(),
                        })
                })
                .collect::<Vec<_>>(),
        );

        // The `Swap` event covers both Mints and Burns.
        // - `token_in` and`token_out` must be a stablecoin and a collateral (or vice versa)
        balance_deltas.extend(
            block
                .events::<abi::swapper::events::Swap>(&[&param.proxy])
                .flat_map(|(event, log)| {
                    vec![
                        BalanceDelta {
                            ord: log.ordinal(),
                            tx: Some(log.receipt.transaction.into()),
                            token: event.token_out,
                            delta: event.amount_in.to_signed_bytes_be(),
                            component_id: hex::encode(param.proxy).into(),
                        },
                        BalanceDelta {
                            ord: log.ordinal(),
                            tx: Some(log.receipt.transaction.into()),
                            token: event.token_in,
                            delta: event
                                .amount_out
                                .neg()
                                .to_signed_bytes_be(),
                            component_id: hex::encode(param.proxy).into(),
                        },
                    ]
                })
                .collect::<Vec<_>>(),
        );
    });

    Ok(BlockBalanceDeltas { balance_deltas })
}

#[substreams::handlers::store]
pub fn store_balances(deltas: BlockBalanceDeltas, store: StoreAddBigInt) {
    tycho_substreams::balances::store_balance_changes(deltas, store);
}

#[substreams::handlers::map]
fn map_protocol_changes(
    raw_params: String,
    block: eth::v2::Block,
    deltas: BlockBalanceDeltas,
    balance_store: StoreDeltas, // Note, this map module is using the `deltas` mode for the store.
) -> Result<BlockContractChanges, substreams::errors::Error> {
    let mut transaction_contract_changes = HashMap::<u64, TransactionContractChanges>::new();

    let params = parse_params(&raw_params)?.convert();

    // We softcode the addition of the transmuter as the sole `ProtocolComponent` that gets created
    // These are passed via parameters to allow us to add new components w/o changing the code.
    params.iter().for_each(|param| {
        if block.number == param.creation_block_no {
            block
                .transactions()
                .filter(|tx| tx.hash == param.creation_hash)
                .for_each(|tx| {
                    let transaction = Transaction {
                        hash: tx.hash.clone(),
                        from: tx.from.clone(),
                        to: tx.to.clone(),
                        index: tx.index.into(),
                    };
                    transaction_contract_changes
                        .entry(0)
                        .or_insert_with(|| TransactionContractChanges::new(&transaction))
                        .component_changes
                        .push(ProtocolComponent {
                            tx: Some(transaction),
                            id: hex::encode(param.proxy).into(),
                            tokens: vec![param.stablecoin.to_vec(), param.anglecoin.to_vec()],
                            contracts: vec![param.proxy.into()],
                            change: ChangeType::Creation.into(),
                            static_att: vec![Attribute {
                                name: "name".into(),
                                value: "Transmuter".into(),
                                change: ChangeType::Creation.into(),
                            }],
                            ..Default::default()
                        });
                })
        }
    });

    // Balance changes are triggered for Redemptions, Burns, and Mints.
    aggregate_balances_changes(balance_store, deltas)
        .into_iter()
        .for_each(|(_, (tx, balances))| {
            transaction_contract_changes
                .entry(tx.index)
                .or_insert_with(|| TransactionContractChanges::new(&tx))
                .balance_changes
                .extend(balances.into_values());
        });

    // Most of the Transmuter's custom logic lies in the storage changes.
    params.iter().for_each(|param| {
        extract_contract_changes(
            &block,
            |addr| addr == param.proxy,
            &mut transaction_contract_changes,
        );
    });

    // Assemble and ship
    Ok(BlockContractChanges {
        block: Some((&block).into()),
        changes: transaction_contract_changes
            .drain()
            .sorted_unstable_by_key(|(index, _)| *index)
            .filter_map(|(_, change)| {
                if change.contract_changes.is_empty() &&
                    change.component_changes.is_empty() &&
                    change.balance_changes.is_empty()
                {
                    None
                } else {
                    Some(change)
                }
            })
            .collect::<Vec<_>>(),
    })
}
