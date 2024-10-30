use crate::abi;
use anyhow::Result;
use itertools::Itertools;
use std::collections::HashMap;
use substreams::{
    hex,
    pb::substreams::StoreDeltas,
    store::{StoreAdd, StoreAddBigInt, StoreAddInt64, StoreGet, StoreGetInt64, StoreNew},
};
use substreams_ethereum::{
    pb::eth::{self},
    Event,
};
use tycho_substreams::{
    balances::aggregate_balances_changes, contract::extract_contract_changes, prelude::*,
};

#[substreams::handlers::map]
pub fn map_components(
    params: String,
    block: eth::v2::Block,
) -> Result<BlockTransactionProtocolComponents> {
    // vault address is sDai contract address
    let vault_address = hex::decode(params).unwrap();
    // locked asset is Dai contract address
    let locked_asset = find_deployed_underlying_address(&vault_address).unwrap();

    let deployment_tx = block
        .transactions()
        .find(|tx| is_deployment_tx(tx, &vault_address));

    let tx_protocol_components = deployment_tx
        .map(|tx| {
            let protocol_component = ProtocolComponent::at_contract(&vault_address, &tx.into())
                .with_tokens(&[locked_asset.as_slice(), vault_address.as_slice()])
                .as_swap_type("sdai_vault", ImplementationType::Vm);

            TransactionProtocolComponents {
                tx: Some(tx.into()),
                components: vec![protocol_component],
            }
        })
        .iter()
        .cloned()
        .collect::<Vec<_>>();

    Ok(BlockTransactionProtocolComponents { tx_components: tx_protocol_components })
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
        // .filter(|log| find_deployed_underlying_address(log.address()).is_some())
        .flat_map(|vault_log| {
            let mut deltas = Vec::new();

            // Take a deeper look if the balance is not returned correctly.
            // When withdrawing, _burn function is called:
            // 1. shares are subtracted to the balance of the owner and to the totalSupply (https://etherscan.io/address/0x83F20F44975D03b1b09e64809B757c47f942BEeA#code#L259)
            // 2. pot.exit(shares) is called. pot is an immutable instance of PotLike initialized in
            //    sDai constructor. Deployed address: 0x197E90f9FAD81970bA7976f33CbD77088E5D7cf7
            // 3. daiJoin.exit(receiver, assets) is called. daiJoin is an immutable instance of
            //    DaiJoinLike. Deployed address: 0x9759A6Ac90977b93B58547b4A71c78317f391A28
            //  - It calls:
            //      - vat.move(msg.sender, address(this), mul(ONE, wad));
            //          - dai[src] = sub(dai[src], rad); // subtract dai from the balance of the
            //            sender
            //          - dai[dst] = add(dai[dst], rad); // adds dai to the balance of sDai contract
            //      - dai.mint(usr, wad);
            //         - calls _mint function of sDai contract (https://etherscan.io/address/0x83F20F44975D03b1b09e64809B757c47f942BEeA#code#L227)
            //         -
            if let Some(ev) = abi::sdai_contract::events::Withdraw::match_and_decode(vault_log.log)
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
                        "Withdraw: vault: {}, dai:- {}, sdai:- {}",
                        address_hex,
                        ev.assets,
                        ev.shares
                    );
                }
            } else if let Some(ev) =
                abi::sdai_contract::events::Deposit::match_and_decode(vault_log.log)
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
                        "Deposit: vault: {}, dai:+ {}, sdai:+ {}",
                        address_hex,
                        ev.assets,
                        ev.shares
                    );
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
///  coresponding changes (balance, component, contract), inserting a new one if it doesn't exist.
///  At the very end, the map can easily be sorted by index to ensure the final
/// `BlockContractChanges`  is ordered by transactions properly.
#[substreams::handlers::map]
pub fn map_protocol_changes(
    block: eth::v2::Block,
    grouped_components: BlockTransactionProtocolComponents,
    deltas: BlockBalanceDeltas,
    components_store: StoreGetInt64,
    balance_store: StoreDeltas, // Note, this map module is using the `deltas` mode for the store.
) -> Result<BlockChanges, anyhow::Error> {
    // We merge contract changes by transaction (identified by transaction index) making it easy to
    //  sort them at the very end.
    let mut transaction_contract: HashMap<u64, TransactionChanges> = HashMap::new();

    // `ProtocolComponents` are gathered from `map_pools_created` which just need a bit of work to
    //   convert into `TransactionContractChanges`
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

    // Balance changes are gathered by the `StoreDelta` based on `PoolBalanceChanged` creating
    //  `BlockBalanceDeltas`. We essentially just process the changes that occurred to the `store`
    // this  block. Then, these balance changes are merged onto the existing map of tx contract
    // changes,  inserting a new one if it doesn't exist.
    aggregate_balances_changes(balance_store, deltas)
        .into_iter()
        .for_each(|(_, (tx, balances))| {
            transaction_contract
                .entry(tx.index)
                .or_insert_with(|| TransactionChanges::new(&tx))
                .balance_changes
                .extend(
                    balances
                        .into_values()
                        .flat_map(|inner_map| inner_map.into_values()),
                );
        });

    // Extract and insert any storage changes that happened for any of the components.
    extract_contract_changes(
        &block,
        |addr| {
            components_store
                .get_last(format!("pool:0x{0}", hex::encode(addr)))
                .is_some()
        },
        &mut transaction_contract,
    );

    // Process all `transaction_contract_changes` for final output in the `BlockContractChanges`,
    //  sorted by transaction index (the key).
    Ok(BlockChanges {
        block: Some((&block).into()),
        changes: transaction_contract
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

fn find_deployed_underlying_address(vault_address: &[u8]) -> Option<[u8; 20]> {
    // Ethereum
    match vault_address {
        // sDai
        hex!("83F20F44975D03b1b09e64809B757c47f942BEeA") => {
            // Dai
            Some(hex!("6B175474E89094C44Da98b954EedeAC495271d0F"))
        }
        _ => None,
    }
}
