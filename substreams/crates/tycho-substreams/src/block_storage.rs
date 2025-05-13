use std::collections::HashMap;

use substreams_ethereum::pb::{eth, eth::v2::block::DetailLevel};

use crate::{
    models::{ContractSlot, StorageChanges, Transaction},
    pb::tycho::evm::v1::TransactionStorageChanges,
};

#[allow(dead_code)]
/// Helper function to extract all storage changes on a block.
/// The raw block information collected is intended to be used by the DCI (Dynamic Contract Indexer)
/// to extract and index relevant changes. This is specifically for dynamically identified contracts
/// that the DCI has chosen to index. Note that core protocol data should still be properly
/// integrated and indexed by the substreams package as per usual.
///
/// ## Panics
/// Panics if the provided block is not an extended block model, as indicated by its detail level.
///
/// ## Warning
/// ⚠️ This function *only* works if the **extended block model** is available,
/// more [here](https://streamingfastio.medium.com/new-block-model-to-accelerate-chain-integration-9f65126e5425)
fn get_block_storage_changes(block: &eth::v2::Block) -> Vec<TransactionStorageChanges> {
    if block.detail_level != Into::<i32>::into(DetailLevel::DetaillevelExtended) {
        panic!("Only extended blocks are supported");
    }
    let mut block_storage_changes = vec![];

    for block_tx in block.transactions() {
        let transaction: Transaction = block_tx.into();

        let mut storage_changes = vec![];
        for call in block_tx.calls.iter() {
            // Filter out calls that are reverted
            if !call.state_reverted {
                for storage_change in call.storage_changes.iter() {
                    storage_changes.push(storage_change.clone());
                }
            }
        }

        // This is needed to get only the latest storage change for each address and slot
        storage_changes.sort_unstable_by_key(|change| change.ordinal);
        let mut storage_changes_per_address_and_slot: HashMap<(Vec<u8>, Vec<u8>), ContractSlot> =
            HashMap::new();
        for storage_change in storage_changes.iter() {
            let contract_slot = ContractSlot {
                slot: storage_change.clone().key,
                value: storage_change.clone().new_value,
            };
            storage_changes_per_address_and_slot.insert(
                (storage_change.address.clone(), storage_change.key.clone()),
                contract_slot,
            );
        }

        let tx_storage_changes: Vec<StorageChanges> = storage_changes_per_address_and_slot
            .into_iter()
            .fold(
                HashMap::new(),
                |mut acc: HashMap<Vec<u8>, Vec<ContractSlot>>, ((address, _key), slot)| {
                    acc.entry(address)
                        .or_default()
                        .push(slot);
                    acc
                },
            )
            .into_iter()
            .map(|(address, slots)| StorageChanges { address, slots })
            .collect();

        block_storage_changes.push(TransactionStorageChanges {
            tx: Some(transaction),
            storage_changes: tx_storage_changes,
        });
    }

    block_storage_changes
}
