use crate::abi;
use anyhow::Result;
use itertools::Itertools;
use std::collections::HashMap;
use substreams::{
    hex,
    pb::substreams::StoreDeltas,
    store::{StoreAdd, StoreAddBigInt, StoreAddInt64, StoreGet, StoreGetInt64, StoreNew},
};
use substreams_ethereum::{pb::eth, Event};
use tycho_substreams::{
    balances::aggregate_balances_changes, contract::extract_contract_changes, prelude::*,
};

#[substreams::handlers::map]
pub fn map_components(
    params: String,
    block: eth::v2::Block,
) -> Result<BlockTransactionProtocolComponents, anyhow::Error> {
    let vault_address = hex::decode(params).unwrap();
    let locked_asset = find_deployed_underlying_address(&vault_address).unwrap();

    let deployment_tx = block
        .transactions()
        .find(|tx| is_deployment_tx(tx, &vault_address));

    Ok(BlockTransactionProtocolComponents {
        tx_components: if let Some(tx) = deployment_tx {
            vec![TransactionProtocolComponents {
                tx: Some(tx.into()),
                components: vec![create_vault_component(&tx.into(), &vault_address, &locked_asset)],
            }]
        } else {
            Vec::new()
        },
    })
}

#[substreams::handlers::store]
pub fn store_components(map: BlockTransactionProtocolComponents, store: StoreAddInt64) {
    store.add_many(
        0,
        &map.tx_components
            .iter()
            .flat_map(|tx_components| &tx_components.components)
            .map(|component| format!("pool:{0}", component.id))
            .collect::<Vec<_>>(),
        1,
    );
}

#[substreams::handlers::map]
pub fn map_relative_balances(
    block: eth::v2::Block,
    store: StoreGetInt64,
) -> Result<BlockBalanceDeltas, anyhow::Error> {
    let balance_deltas = block
        .logs()
        .filter(|log| find_deployed_underlying_address(log.address()).is_some())
        .flat_map(|vault_log| {
            let mut deltas = Vec::new();

            if let Some(ev) =
                abi::stakedfrax_contract::events::Withdraw::match_and_decode(vault_log.log)
            {
                let address_bytes_be = vault_log.address();
                let address_hex = format!("0x{}", hex::encode(address_bytes_be));
                if store
                    .get_last(format!("pool:{}", address_hex))
                    .is_some()
                {
                    deltas.extend_from_slice(&[
                        BalanceDelta {
                            ord: vault_log.ordinal(),
                            tx: Some(vault_log.receipt.transaction.into()),
                            token: find_deployed_underlying_address(address_bytes_be)
                                .unwrap()
                                .to_vec(),
                            delta: ev.assets.neg().to_signed_bytes_be(),
                            component_id: address_hex.as_bytes().to_vec(),
                        },
                        BalanceDelta {
                            ord: vault_log.ordinal(),
                            tx: Some(vault_log.receipt.transaction.into()),
                            token: address_bytes_be.to_vec(),
                            delta: ev.shares.neg().to_signed_bytes_be(),
                            component_id: address_hex.as_bytes().to_vec(),
                        },
                    ]);
                    substreams::log::debug!(
                        "Withdraw: vault: {}, frax:- {}, sfrax:- {}",
                        address_hex,
                        ev.assets,
                        ev.shares
                    );
                }
            } else if let Some(ev) =
                abi::stakedfrax_contract::events::Deposit::match_and_decode(vault_log.log)
            {
                let address_bytes_be = vault_log.address();
                let address_hex = format!("0x{}", hex::encode(address_bytes_be));

                if store
                    .get_last(format!("pool:{}", address_hex))
                    .is_some()
                {
                    deltas.extend_from_slice(&[
                        BalanceDelta {
                            ord: vault_log.ordinal(),
                            tx: Some(vault_log.receipt.transaction.into()),
                            token: find_deployed_underlying_address(address_bytes_be)
                                .unwrap()
                                .to_vec(),
                            delta: ev.assets.to_signed_bytes_be(),
                            component_id: address_hex.as_bytes().to_vec(),
                        },
                        BalanceDelta {
                            ord: vault_log.ordinal(),
                            tx: Some(vault_log.receipt.transaction.into()),
                            token: address_bytes_be.to_vec(),
                            delta: ev.shares.to_signed_bytes_be(),
                            component_id: address_hex.as_bytes().to_vec(),
                        },
                    ]);
                    substreams::log::debug!(
                        "Deposit: vault: {}, frax:+ {}, sfrax:+ {}",
                        address_hex,
                        ev.assets,
                        ev.shares
                    );
                }
            } else if let Some(ev) =
                abi::stakedfrax_contract::events::DistributeRewards::match_and_decode(vault_log.log)
            {
                let address_bytes_be = vault_log.address();
                let address_hex = format!("0x{}", hex::encode(address_bytes_be));

                if store
                    .get_last(format!("pool:{}", address_hex))
                    .is_some()
                {
                    deltas.extend_from_slice(&[BalanceDelta {
                        ord: vault_log.ordinal(),
                        tx: Some(vault_log.receipt.transaction.into()),
                        token: find_deployed_underlying_address(address_bytes_be)
                            .unwrap()
                            .to_vec(),
                        delta: ev
                            .rewards_to_distribute
                            .to_signed_bytes_be(),
                        component_id: address_hex.as_bytes().to_vec(),
                    }]);
                    // Log token and amount without encoding
                    substreams::log::debug!(
                        "DistributeRewards: vault: {}, frax:+ {}",
                        address_hex,
                        ev.rewards_to_distribute
                    );
                }
            }
            deltas
        })
        .collect::<Vec<_>>();

    Ok(BlockBalanceDeltas { balance_deltas })
}

#[substreams::handlers::store]
pub fn store_balances(deltas: BlockBalanceDeltas, store: StoreAddBigInt) {
    tycho_substreams::balances::store_balance_changes(deltas, store);
}

#[substreams::handlers::map]
pub fn map_protocol_changes(
    block: eth::v2::Block,
    grouped_components: BlockTransactionProtocolComponents,
    deltas: BlockBalanceDeltas,
    components_store: StoreGetInt64,
    balance_store: StoreDeltas,
) -> Result<BlockChanges, anyhow::Error> {
    let mut transaction_contract: HashMap<u64, TransactionChanges> = HashMap::new();

    grouped_components
        .tx_components
        .iter()
        .for_each(|tx_component| {
            let tx = tx_component.tx.as_ref().unwrap();
            transaction_contract
                .entry(tx.index)
                .or_insert_with(|| TransactionChanges::new(tx))
                .component_changes
                .extend_from_slice(&tx_component.components);
        });

    aggregate_balances_changes(balance_store, deltas)
        .into_iter()
        .for_each(|(_, (tx, balances))| {
            transaction_contract
                .entry(tx.index)
                .or_insert_with(|| TransactionChanges::new(&tx))
                .balance_changes
                .extend(balances.into_values());
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
            .drain()
            .sorted_unstable_by_key(|(index, _)| *index)
            .filter_map(|(_, change)| {
                if change.contract_changes.is_empty()
                    && change.component_changes.is_empty()
                    && change.balance_changes.is_empty()
                {
                    None
                } else {
                    Some(change)
                }
            })
            .collect::<Vec<_>>(),
    })
}

fn is_deployment_tx(tx: &eth::v2::TransactionTrace, vault_address: &[u8]) -> bool {
    let created_accounts = tx
        .calls
        .iter()
        .flat_map(|call| {
            call.account_creations
                .iter()
                .map(|ac| ac.account.to_owned())
        })
        .collect::<Vec<_>>();

    if let Some(deployed_address) = created_accounts.first() {
        return deployed_address.as_slice() == vault_address;
    }
    false
}

fn create_vault_component(
    tx: &Transaction,
    component_id: &[u8],
    locked_asset: &[u8],
) -> ProtocolComponent {
    substreams::log::debug!("create_vault_component: {}", hex::encode(component_id));
    ProtocolComponent::at_contract(component_id, tx)
        .with_tokens(&[locked_asset, component_id])
        .as_swap_type("sfrax_vault", ImplementationType::Vm)
}

fn find_deployed_underlying_address(vault_address: &[u8]) -> Option<[u8; 20]> {
    match vault_address {
        hex!("A663B02CF0a4b149d2aD41910CB81e23e1c41c32") => {
            Some(hex!("853d955aCEf822Db058eb8505911ED77F175b99e"))
        }
        _ => None,
    }
}
