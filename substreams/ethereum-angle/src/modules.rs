use crate::abi::{redeemer::events::Redeemed, swapper::events::Swap};
use itertools::Itertools;
use std::collections::HashMap;
use substreams::{
    hex,
    pb::substreams::StoreDeltas,
    store::{StoreAddBigInt, StoreNew},
};
use substreams_ethereum::pb::eth;
use tycho_substreams::{
    balances::aggregate_balances_changes, contract::extract_contract_changes, prelude::*,
};

const ANGLE_PROXY: &[u8; 20] = &hex!("00253582b2a3FE112feEC532221d9708c64cEFAb");
const ANGLE_IMPL: &[u8; 20] = &hex!("5d34839A3d4051f630D36e26698d53c58DD39072");

const CREATION_BLOCK_NO: u64 = 17869756;
const CREATION_HASH: [u8; 32] =
    hex!("c12328a517e216ee37f974281e019e0041ad755c4868e3b7a8366948ebc55388");

const EURA_ADDR: [u8; 20] = hex!("1a7e4e63778b4f12a199c062f3efdd288afcbce8");
const EURC_ADDR: [u8; 20] = hex!("1abaea1f7c830bd89acc67ec4af516284b1bc33c");

/// Maps the `Redeemed` and `Swap` events to `BalanceDelta`s representing the Redemptions, Mints,
///  and burns by the transmuter.
#[substreams::handlers::map]
pub fn map_relative_balances(block: eth::v2::Block) -> Result<BlockBalanceDeltas, anyhow::Error> {
    let mut balance_deltas: Vec<BalanceDelta> = vec![];

    balance_deltas.extend(
        block
            .events::<Redeemed>(&[ANGLE_PROXY])
            .flat_map(|(event, log)| {
                event
                    .tokens
                    .into_iter()
                    .map(move |token| BalanceDelta {
                        ord: log.ordinal(),
                        tx: Some(log.receipt.transaction.into()),
                        token,
                        delta: event.amount.to_signed_bytes_be(),
                        component_id: hex::encode(ANGLE_PROXY).into(),
                    })
            })
            .collect::<Vec<_>>(),
    );

    // The `Swap` event covers both Mints and Burns.
    // - `token_in` and`token_out` must be a stablecoin and a collateral (or vice versa)
    balance_deltas.extend(
        block
            .events::<Swap>(&[ANGLE_PROXY])
            .flat_map(|(event, log)| {
                vec![
                    BalanceDelta {
                        ord: log.ordinal(),
                        tx: Some(log.receipt.transaction.into()),
                        token: event.token_out,
                        delta: event.amount_out.to_signed_bytes_be(),
                        component_id: hex::encode(ANGLE_PROXY).into(),
                    },
                    BalanceDelta {
                        ord: log.ordinal(),
                        tx: Some(log.receipt.transaction.into()),
                        token: event.token_in,
                        delta: event
                            .amount_in
                            .neg()
                            .to_signed_bytes_be(),
                        component_id: hex::encode(ANGLE_PROXY).into(),
                    },
                ]
            })
            .collect::<Vec<_>>(),
    );

    Ok(BlockBalanceDeltas { balance_deltas })
}

#[substreams::handlers::store]
pub fn store_balances(deltas: BlockBalanceDeltas, store: StoreAddBigInt) {
    tycho_substreams::balances::store_balance_changes(deltas, store);
}

#[substreams::handlers::map]
fn map_protocol_changes(
    block: eth::v2::Block,
    deltas: BlockBalanceDeltas,
    balance_store: StoreDeltas, // Note, this map module is using the `deltas` mode for the store.
) -> Result<BlockContractChanges, substreams::errors::Error> {
    let mut transaction_contract_changes = HashMap::<u64, TransactionContractChanges>::new();

    // We hardcode the addition of the transmuter as the sole `ProtocolComponent` that gets created
    if block.number == CREATION_BLOCK_NO {
        block
            .transactions()
            .filter(|tx| tx.hash == CREATION_HASH)
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
                        id: hex::encode(ANGLE_PROXY),
                        tokens: vec![EURC_ADDR.to_vec(), EURA_ADDR.to_vec()],
                        contracts: vec![ANGLE_PROXY.into(), ANGLE_IMPL.into()],
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
    extract_contract_changes(&block, |addr| addr == ANGLE_PROXY, &mut transaction_contract_changes);

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
