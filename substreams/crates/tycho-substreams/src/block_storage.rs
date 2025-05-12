use std::collections::HashMap;

use substreams_ethereum::pb::{eth, eth::v2::block::DetailLevel};

use crate::{
    models::{ContractSlot, StorageChanges, Transaction},
    pb::tycho::evm::v1::TransactionStorageChanges,
};

#[allow(dead_code)]
/// Helper function to extract all storage changes on a block.
/// This is used by the DCI as raw block information for it to extract which changes are relevant to
/// be indexed.This is used only for the contracts it has dynamically identified and is
/// indexing, the core protocol itself should still be properly integrated and indexed by the
/// substreams package as per usual.
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

        let mut storage_changes_per_address: HashMap<Vec<u8>, Vec<ContractSlot>> = HashMap::new();
        for call in block_tx.calls.iter() {
            for storage_change in call.storage_changes.iter() {
                let contract_slot = ContractSlot {
                    slot: storage_change.clone().key,
                    value: storage_change.clone().new_value,
                };
                storage_changes_per_address
                    .entry(storage_change.clone().address)
                    .or_default()
                    .push(contract_slot);
            }
        }

        let storage_changes = storage_changes_per_address
            .into_iter()
            .map(|(address, slots)| StorageChanges { address, slots })
            .collect::<Vec<_>>();

        block_storage_changes
            .push(TransactionStorageChanges { tx: Some(transaction), storage_changes });
    }

    block_storage_changes
}
