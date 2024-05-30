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

// ref:
// https://docs.frax.finance/smart-contracts/sfrax-contract-addresses
// https://docs.frax.finance/smart-contracts/frax
type AddressPair = ([u8; 20], [u8; 20]);
const ADDRESS_MAP: &[AddressPair] = &[
    (
        hex!("e3b3FE7bcA19cA77Ad877A5Bebab186bEcfAD906"),
        hex!("17FC002b466eEc40DaE837Fc4bE5c67993ddBd6F"),
    ), // Arbitrum
    (
        hex!("a63f56985F9C7F3bc9fFc5685535649e0C1a55f3"),
        hex!("90C97F71E18723b0Cf0dfa30ee176Ab653E89F40"),
    ), // BSC
    (
        hex!("A663B02CF0a4b149d2aD41910CB81e23e1c41c32"),
        hex!("853d955aCEf822Db058eb8505911ED77F175b99e"),
    ), // Ethereum
    (
        hex!("3405E88af759992937b84E58F2Fe691EF0EeA320"), 
        hex!("D24C2Ad096400B6FBcd2ad8B24E7acBc21A1da64"),
    ), //Avalanche
    (
        hex!("2Dd1B4D4548aCCeA497050619965f91f78b3b532"),
        hex!("2E3D870790dC77A83DD1d18184Acc7439A53f475"),
    ), // Optimism
    (
        hex!("2C37fb628b35dfdFD515d41B0cAAe11B542773C3"),
        hex!("45c32fA6DF82ead1e2EF74d17b76547EDdFaFF89"),
    ), // Polygon
];

#[substreams::handlers::map]
pub fn map_components(
    params: String,
    block: eth::v2::Block,
) -> Result<BlockTransactionProtocolComponents> {
    let (vault_address, locked_asset) =
        find_deployed_vault_address(hex::decode(params).unwrap().as_slice()).unwrap();
    // We store these as a hashmap by tx hash since we need to agg by tx hash later
    Ok(BlockTransactionProtocolComponents {
        tx_components: block
            .transactions()
            .filter_map(|tx| {
                let components = tx
                    .calls()
                    .filter(| call| !call.call.state_reverted)
                    .filter_map(|_| {
                        if is_deployment_tx(tx, &vault_address) {
                            Some(create_vault_component(&tx.into(), &vault_address, &locked_asset))
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

/// Simply stores the `ProtocolComponent`s with the pool id as the key
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

/// Since the `PoolBalanceChanged` and `Swap` events administer only deltas, we need to leverage a
/// map and a  store to be able to tally up final balances for tokens in a pool.
#[substreams::handlers::map]
pub fn map_relative_balances(
    block: eth::v2::Block,
    store: StoreGetInt64,
) -> Result<BlockBalanceDeltas, anyhow::Error> {
    let balance_deltas = block
        .logs()
        // .filter(|log| find_deployed_vault_address(log.address()).is_some())
        .flat_map(|vault_log| {
            let mut deltas = Vec::new();

            if let Some(ev) =
                abi::stakedfrax_contract::events::Withdraw::match_and_decode(vault_log.log)
            {
                let address = hex::encode(vault_log.address());
                let component_id = format!("0x{}", address);
                if store
                    .get_last(format!("pool:{}", component_id))
                    .is_some()
                {
                    deltas.extend_from_slice(&[
                        BalanceDelta {
                            ord: vault_log.ordinal(),
                            tx: Some(vault_log.receipt.transaction.into()),
                            token: match_underlying_asset(vault_log.address())
                                .unwrap()
                                .to_vec(),
                            delta: ev.assets.neg().to_signed_bytes_be(),
                            component_id: component_id.as_bytes().to_vec(),
                        },
                        BalanceDelta {
                            ord: vault_log.ordinal(),
                            tx: Some(vault_log.receipt.transaction.into()),
                            token: component_id.as_bytes().to_vec(),
                            delta: ev.shares.neg().to_signed_bytes_be(),
                            component_id: component_id.as_bytes().to_vec(),
                        },
                    ])
                }
            } else if let Some(ev) =
                abi::stakedfrax_contract::events::Deposit::match_and_decode(vault_log.log)
            {
                let address = hex::encode(vault_log.address());
                let component_id = format!("0x{}", address);
                if store
                    .get_last(format!("pool:{}", component_id))
                    .is_some()
                {
                    deltas.extend_from_slice(&[
                        BalanceDelta {
                            ord: vault_log.ordinal(),
                            tx: Some(vault_log.receipt.transaction.into()),
                            token: match_underlying_asset(vault_log.address())
                                .unwrap()
                                .to_vec(),
                            delta: ev.assets.to_signed_bytes_be(),
                            component_id: component_id.as_bytes().to_vec(),
                        },
                        BalanceDelta {
                            ord: vault_log.ordinal(),
                            tx: Some(vault_log.receipt.transaction.into()),
                            token: component_id.as_bytes().to_vec(),
                            delta: ev.shares.to_signed_bytes_be(),
                            component_id: component_id.as_bytes().to_vec(),
                        },
                    ])
                }
            } else if let Some(ev) =
                abi::stakedfrax_contract::events::DistributeRewards::match_and_decode(vault_log.log)
            {
                let address = hex::encode(vault_log.address());
                let component_id = format!("0x{}", address);
                if store
                    .get_last(format!("pool:{}", component_id))
                    .is_some()
                {
                    deltas.push(BalanceDelta {
                        ord: vault_log.ordinal(),
                        tx: Some(vault_log.receipt.transaction.into()),
                        token: match_underlying_asset(vault_log.address()).unwrap().to_vec(),
                        delta: ev
                            .rewards_to_distribute
                            .to_signed_bytes_be(),
                        component_id: component_id.as_bytes().to_vec(),
                    });
                }
            }
            deltas
        })
        .collect::<Vec<_>>();

    Ok(BlockBalanceDeltas { balance_deltas })
}

/// It's significant to include both the `pool_id` and the `token_id` for each balance delta as the
///  store key to ensure that there's a unique balance being tallied for each.
#[substreams::handlers::store]
pub fn store_balances(deltas: BlockBalanceDeltas, store: StoreAddBigInt) {
    tycho_substreams::balances::store_balance_changes(deltas, store);
}

/// This is the main map that handles most of the indexing of this substream.
/// Every contract change is grouped by transaction index via the `transaction_contract_changes`
///  map. Each block of code will extend the `TransactionContractChanges` struct with the
///  cooresponding changes (balance, component, contract), inserting a new one if it doesn't exist.
///  At the very end, the map can easily be sorted by index to ensure the final
/// `BlockContractChanges`  is ordered by transactions properly.
#[substreams::handlers::map]
pub fn map_protocol_changes(
    block: eth::v2::Block,
    grouped_components: BlockTransactionProtocolComponents,
    deltas: BlockBalanceDeltas,
    components_store: StoreGetInt64,
    balance_store: StoreDeltas, // Note, this map module is using the `deltas` mode for the store.
) -> Result<BlockContractChanges> {
    // We merge contract changes by transaction (identified by transaction index) making it easy to
    //  sort them at the very end.
    let mut transaction_contract_changes: HashMap<_, TransactionContractChanges> = HashMap::new();

    // `ProtocolComponents` are gathered from `map_pools_created` which just need a bit of work to
    //   convert into `TransactionContractChanges`
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

    // Balance changes are gathered by the `StoreDelta` based on `PoolBalanceChanged` creating
    //  `BlockBalanceDeltas`. We essentially just process the changes that occurred to the `store`
    // this  block. Then, these balance changes are merged onto the existing map of tx contract
    // changes,  inserting a new one if it doesn't exist.
    aggregate_balances_changes(balance_store, deltas)
        .into_iter()
        .for_each(|(_, (tx, balances))| {
            transaction_contract_changes
                .entry(tx.index)
                .or_insert_with(|| TransactionContractChanges::new(&tx))
                .balance_changes
                .extend(balances.into_values());
        });

    // Extract and insert any storage changes that happened for any of the components.
    extract_contract_changes(
        &block,
        |addr| {
            components_store
                .get_last(format!("pool:0x{0}", hex::encode(addr)))
                .is_some()
        },
        &mut transaction_contract_changes,
    );

    // Process all `transaction_contract_changes` for final output in the `BlockContractChanges`,
    //  sorted by transaction index (the key).
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
    ProtocolComponent::at_contract(component_id, tx)
        .with_tokens(&[locked_asset, component_id])
        .as_swap_type("sfrax_vault", ImplementationType::Vm)
}

fn match_underlying_asset(address: &[u8]) -> Option<[u8; 20]> {
    find_deployed_vault_address(address).map(|(_, underlying_asset)| underlying_asset)
}

fn find_deployed_vault_address(vault_address: &[u8]) -> Option<AddressPair> {
    ADDRESS_MAP
        .iter()
        .find(|( addr, _)| addr == vault_address)
        .copied()
}
