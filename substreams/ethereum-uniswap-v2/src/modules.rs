use crate::pair_factory;
use std::collections::{hash_map::Entry, HashMap};
use substreams::store::{StoreGet, StoreGetProto, StoreSetProto, StoreSet};
use substreams_ethereum::pb::eth::v2::Block;
use tycho_substreams::prelude::*;
use hex_literal::hex;

const UNISWAP_V2_FACTORY: [u8; 20] = hex!("5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f");

struct SlotValue {
    new_value: Vec<u8>,
    start_value: Vec<u8>,
}

impl SlotValue {
    fn has_changed(&self) -> bool {
        self.start_value != self.new_value
    }
}

struct InterimContractChange {
    address: Vec<u8>,
    balance: Vec<u8>,
    code: Vec<u8>,
    slots: HashMap<Vec<u8>, SlotValue>,
    change: ChangeType,
}

impl From<InterimContractChange> for ContractChange {
    fn from(value: InterimContractChange) -> Self {
        ContractChange {
            address: value.address,
            balance: value.balance,
            code: value.code,
            slots: value
                .slots
                .into_iter()
                .filter(|(_, value)| value.has_changed())
                .map(|(slot, value)| ContractSlot { slot, value: value.new_value })
                .collect(),
            change: value.change.into(),
        }
    }
}

pub fn map_changes(
    block: Block,
) -> Result<BlockContractChanges, substreams::errors::Error> {
    let mut block_changes = BlockContractChanges::default();
    let mut tx_change = TransactionContractChanges::default();
    let mut changed_contracts: HashMap<Vec<u8>, InterimContractChange> = HashMap::new();

    let created_accounts: HashMap<_, _> = block
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter().flat_map(|call| {
                call.account_creations
                    .iter()
                    .map(|ac| (&ac.account, ac.ordinal))
            })
        })
        .collect();

    for block_tx in block.transactions() {
        for call in block_tx.calls.iter().filter(|call| !call.state_reverted) {
            if call.address == UNISWAP_V2_FACTORY
                && call.input.len() >= 4
                && call.input[0..4] == pair_factory::CREATE_PAIR_SIGNATURE
            {
                if let Ok(new_component) =
                    pair_factory::decode_create_pair_call(call, &block_tx.into())
                {
                    tx_change.component_changes.push(new_component.clone());
                }
            }
        }

        let mut storage_changes = block_tx
            .calls
            .iter()
            .filter(|call| !call.state_reverted)
            .flat_map(|call| {
                call.storage_changes.iter().filter(|c| {
                    tx_change
                        .component_changes
                        .iter()
                        .any(|component| component.contracts.contains(&call.address))
                })
            })
            .collect::<Vec<_>>();
        storage_changes.sort_unstable_by_key(|change| change.ordinal);

        for storage_change in storage_changes.iter() {
            match changed_contracts.entry(storage_change.address.clone()) {
                Entry::Occupied(mut e) => {
                    let contract_change = e.get_mut();
                    match contract_change.slots.entry(storage_change.key.clone()) {
                        Entry::Occupied(mut v) => {
                            let slot_value = v.get_mut();
                            slot_value.new_value.copy_from_slice(&storage_change.new_value);
                        }
                        Entry::Vacant(v) => {
                            v.insert(SlotValue {
                                new_value: storage_change.new_value.clone(),
                                start_value: storage_change.old_value.clone(),
                            });
                        }
                    }
                }
                Entry::Vacant(e) => {
                    let mut slots = HashMap::new();
                    slots.insert(
                        storage_change.key.clone(),
                        SlotValue {
                            new_value: storage_change.new_value.clone(),
                            start_value: storage_change.old_value.clone(),
                        },
                    );
                    e.insert(InterimContractChange {
                        address: storage_change.address.clone(),
                        balance: Vec::new(),
                        code: Vec::new(),
                        slots,
                        change: if created_accounts.contains_key(&storage_change.address) {
                            ChangeType::Creation
                        } else {
                            ChangeType::Update
                        },
                    });
                }
            }
        }

        let mut balance_changes = block_tx
            .calls
            .iter()
            .filter(|call| !call.state_reverted)
            .flat_map(|call| {
                call.balance_changes.iter().filter(|c| {
                    tx_change
                        .component_changes
                        .iter()
                        .any(|component| component.contracts.contains(&call.address))
                })
            })
            .collect::<Vec<_>>();
        balance_changes.sort_unstable_by_key(|change| change.ordinal);

        for balance_change in balance_changes.iter() {
            match changed_contracts.entry(balance_change.address.clone()) {
                Entry::Occupied(mut e) => {
                    let contract_change = e.get_mut();
                    if let Some(new_balance) = &balance_change.new_value {
                        contract_change.balance.clear();
                        contract_change.balance.extend_from_slice(&new_balance.bytes);
                    }
                }
                Entry::Vacant(e) => {
                    if let Some(new_balance) = &balance_change.new_value {
                        e.insert(InterimContractChange {
                            address: balance_change.address.clone(),
                            balance: new_balance.bytes.clone(),
                            code: Vec::new(),
                            slots: HashMap::new(),
                            change: if created_accounts.contains_key(&balance_change.address) {
                                ChangeType::Creation
                            } else {
                                ChangeType::Update
                            },
                        });
                    }
                }
            }
        }

        let mut code_changes = block_tx
            .calls
            .iter()
            .filter(|call| !call.state_reverted)
            .flat_map(|call| {
                call.code_changes.iter().filter(|c| {
                    tx_change
                        .component_changes
                        .iter()
                        .any(|component| component.contracts.contains(&call.address))
                })
            })
            .collect::<Vec<_>>();
        code_changes.sort_unstable_by_key(|change| change.ordinal);

        for code_change in code_changes.iter() {
            match changed_contracts.entry(code_change.address.clone()) {
                Entry::Occupied(mut e) => {
                    let contract_change = e.get_mut();
                    contract_change.code.clear();
                    contract_change.code.extend_from_slice(&code_change.new_code);
                }
                Entry::Vacant(e) => {
                    e.insert(InterimContractChange {
                        address: code_change.address.clone(),
                        balance: Vec::new(),
                        code: code_change.new_code.clone(),
                        slots: HashMap::new(),
                        change: if created_accounts.contains_key(&code_change.address) {
                            ChangeType::Creation
                        } else {
                            ChangeType::Update
                        },
                    });
                }
            }
        }

        if !storage_changes.is_empty() || !balance_changes.is_empty() || !code_changes.is_empty() {
            tx_change.tx = Some(Transaction {
                hash: block_tx.hash.clone(),
                from: block_tx.from.clone(),
                to: block_tx.to.clone(),
                index: block_tx.index as u64,
            });

            for (_, change) in changed_contracts.drain() {
                tx_change.contract_changes.push(change.into());
            }

            block_changes.changes.push(tx_change.clone());

            tx_change.tx = None;
            tx_change.contract_changes.clear();
        }
    }

    block_changes.block = Some(Block {
        number: block.number,
        hash: block.hash.clone(),
        parent_hash: block
            .header
            .as_ref()
            .expect("Block header not present")
            .parent_hash
            .clone(),
        ts: block.header.as_ref().expect("Block header not present").timestamp,
    }.into());

    Ok(block_changes)
}

pub fn store_pools(
    output: tycho::BlockContractChanges,
    pool_store: substreams::store::StoreSetProto<ProtocolComponent>,
) {
    for tx_change in output.changes {
        for component in tx_change.component_changes {
            pool_store.set(format!("pool:{}", hex::encode(&component.contracts[0])), &component);
        }
    }
}