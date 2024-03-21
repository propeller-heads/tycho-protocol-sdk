use crate::abi;
use anyhow::Result;
use itertools::Itertools;
use std::collections::HashMap;
use substreams::{hex, pb::substreams::StoreDeltas, store::{StoreAdd, StoreAddBigInt, StoreAddInt64, StoreGet, StoreGetInt64, StoreNew}};
use substreams_ethereum::{pb::eth, Event};
use tycho_substreams::{balances::aggregate_balances_changes, contract::extract_contract_changes, prelude::*};

const FACTORY_ADDRESS: [u8; 20] = hex!("5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f");
const PAIR_CREATED_EVENT_SIG: [u8; 32] = hex!("0d3648bd0f6ba80134a33ba9275ac585d9d315f0ad8355cddefde31afa28d0e9");
 
#[substreams::handlers::map]
pub fn map_components(block: eth::v2::Block) -> Result<BlockTransactionProtocolComponents> {
    Ok(BlockTransactionProtocolComponents {
        tx_components: block
            .events()
            .filter(|event| event.address == FACTORY_ADDRESS && event.topics.first().unwrap() == &PAIR_CREATED_EVENT_SIG)
            .flat_map(|event| {
                let pair_address = event.params.get(2).unwrap().into_address().unwrap();
                let component = ProtocolComponent {
                    id: hex::encode(&pair_address),
                    tokens: vec![
                        event.params.get(0).unwrap().into_address().unwrap(),
                        event.params.get(1).unwrap().into_address().unwrap(),
                    ],
                    contracts: vec![pair_address],
                    change: tycho::ChangeType::Creation.into(),
                };

                Some(TransactionProtocolComponents { 
                    tx: Some(event.receipt.transaction.into()),
                    components: vec![component]
                })
            })
            .collect::<Vec<_>>(),
    })
}

#[substreams::handlers::store]
pub fn store_components(map: BlockTransactionProtocolComponents, store: StoreAddInt64) {
    store.add_many(
        0,
        &map.tx_components
            .iter()
            .flat_map(|tx_components| &tx_components.components)
            .map(|component| format!("pair:{0}", component.id))
            .collect::<Vec<_>>(),
        1,
    );
}

#[substreams::handlers::get]
pub fn store_components_get() -> StoreGetInt64 {
    StoreGetInt64::new()
}

#[substreams::handlers::map]
pub fn map_relative_balances(
    block: eth::v2::Block,
    store: StoreGetInt64
) -> Result<BlockBalanceDeltas, anyhow::Error> {
    let balance_deltas = block
        .events()
        .filter(|event| event.address == FACTORY_ADDRESS && event.topics.first().unwrap() == &hex!("ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef"))
        .flat_map(|event| {
            let pair_address = event.address;
            let component_id = pair_address.to_vec();

            if store.get_last(format!("pair:{}", hex::encode(&component_id))).is_some() {
                let token = event.params.get(2).unwrap().into_address().unwrap();
                let amount = event.params.get(3).unwrap().into_bigint().unwrap();

                vec![
                    BalanceDelta {
                        ord: event.ordinal(),
                        tx: Some(event.receipt.transaction.into()),
                        token: token.to_vec(),
                        delta: amount.to_signed_bytes_be(),
                        component_id,
                    }
                ]
            } else {
                vec![]
            }
        })
        .collect::<Vec<_>>();

    Ok(BlockBalanceDeltas { balance_deltas })
}

#[substreams::handlers::store]
pub fn store_balances(deltas: BlockBalanceDeltas, store: StoreAddBigInt) {
    tycho_substreams::balances::store_balance_changes(deltas, store);
}

#[substreams::handlers::store]
pub fn store_balances_deltas() -> StoreDeltas {
    StoreDeltas::new(StoreNew::<BigInt>::new())
}

#[substreams::handlers::map]
pub fn map_protocol_changes(
    block: eth::v2::Block,
    grouped_components: BlockTransactionProtocolComponents,
    deltas: BlockBalanceDeltas,
    components_store: StoreGetInt64,
    balance_store: StoreDeltas,
) -> Result<BlockContractChanges> {
    let mut transaction_contract_changes: HashMap<_, TransactionContractChanges> = HashMap::new();

    grouped_components
        .tx_components
        .iter()
        .for_each(|tx_component| {
            let tx = tx_component.tx.as_ref().unwrap();
            transaction_contract_changes
                .entry(tx.index)
                .or_insert_with(|| TransactionContractChanges::new(tx))
                .component_changes
                .extend_from_slice(&tx_component.components);
        });

    aggregate_balances_changes(balance_store, deltas)
        .into_iter()
        .for_each(|(_, (tx, balances))| {
            transaction_contract_changes
                .entry(tx.index)
                .or_insert_with(|| TransactionContractChanges::new(&tx))
                .balance_changes
                .extend(balances.into_values());
        });

    extract_contract_changes(
        &block,
        |addr| {
            components_store
                .get_last(format!("pair:{0}", hex::encode(addr)))
                .is_some()
        },
        &mut transaction_contract_changes,
    );

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