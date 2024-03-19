use std::collections::{hash_map::Entry, HashMap};

use anyhow::{anyhow, bail};
use hex_literal::hex;
use substreams_ethereum::pb::eth::{self};

use pb::tycho::evm::v1::{self as tycho};

mod pb;

const FACTORY_CONTRACT: [u8; 20] = hex!("5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f");
const PAIR_CREATED_SIG: [u8; 32] = hex!("0d3648bd0f6ba80134a33ba9275ac585d9d315f0ad8355cddefde31afa28d0e9");

struct SlotValue {
    new_value: Vec<u8>,
    start_value: Vec<u8>,
}

impl SlotValue {
    fn has_changed(&self) -> bool {
        self.start_value != self.new_value
    }
}

// Uses a map for slots, protobuf does not allow bytes in hashmap keys
struct InterimContractChange {
    address: Vec<u8>,
    balance: Vec<u8>,
    code: Vec<u8>,
    slots: HashMap<Vec<u8>, SlotValue>,
    change: tycho::ChangeType,
}

impl From<InterimContractChange> for tycho::ContractChange {
    fn from(value: InterimContractChange) -> Self {
        tycho::ContractChange {
            address: value.address,
            balance: value.balance,
            code: value.code,
            slots: value
                .slots
                .into_iter()
                .filter(|(_, value)| value.has_changed())
                .map(|(slot, value)| tycho::ContractSlot { slot, value: value.new_value })
                .collect(),
            change: value.change.into(),
        }
    }
}

#[substreams::handlers::map]
fn map_changes(
    block: eth::v2::Block,
) -> Result<tycho::BlockContractChanges, substreams::errors::Error> {
    let mut block_changes = tycho::BlockContractChanges::default();

    let mut tx_change = tycho::TransactionContractChanges::default();

    let mut changed_contracts: HashMap<Vec<u8>, InterimContractChange> = HashMap::new();

    // Collect all accounts created in this block
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

    // Detect PairCreated events from the factory contract
    let pair_created_events = block
        .transactions()
        .flat_map(|tx| {
            tx.calls.iter().flat_map(|call| {
                call.logs.iter().filter(|log| {
                    log.address == FACTORY_CONTRACT && log.topics.contains(&PAIR_CREATED_SIG)
                })
            })
        })
        .collect::<Vec<_>>();

    for pair_created_event in pair_created_events {
        if pair_created_event.topics.len() != 4 {
            bail!("Invalid PairCreated event topics".to_string());
        }

        // Decode PairCreated event parameters
        let token0 = pair_created_event.topics[1].as_ref().to_vec();
        let token1 = pair_created_event.topics[2].as_ref().to_vec();
        let pair_address = pair_created_event.topics[3].as_ref().to_vec();

        let mut tokens = vec![token0.clone(), token1.clone()];
        tokens.sort();

        let new_component = tycho::ProtocolComponent {
            id: format!("{}{}", hex::encode(token0), hex::encode(token1)),
            tokens,
            contracts: vec![pair_address.clone()],
            static_att: vec![],
            change: tycho::ChangeType::Creation.into(),
        };
        tx_change.component_changes.push(new_component);

        // Track the newly created pair contract
        changed_contracts.insert(
            pair_address,
            InterimContractChange {
                address: pair_address.clone(),
                balance: Vec::new(),
                code: Vec::new(),
                slots: HashMap::new(),
                change: tycho::ChangeType::Creation,
            },
        );
    }

    // Extract all contract changes
    for block_tx in block.transactions() {
        let mut storage_changes = block_tx
            .calls
            .iter()
            .filter(|call| !call.state_reverted)
            .flat_map(|call| {
                call.storage_changes
                    .iter()
                    .filter(|c| changed_contracts.contains_key(&c.address))
            })
            .collect::<Vec<_>>();
        storage_changes.sort_unstable_by_key(|change| change.ordinal);

        for storage_change in storage_changes.iter() {
            match changed_contracts.entry(storage_change.address.clone()) {
                Entry::Occupied(mut e) => {
                    let contract_change = e.get_mut();
                    match contract_change
                        .slots
                        .entry(storage_change.key.clone())
                    {
                        Entry::Occupied(mut v) => {
                            let slot_value = v.get_mut();
                            slot_value
                                .new_value
                                .copy_from_slice(&storage_change.new_value);
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
                            tycho::ChangeType::Creation
                        } else {
                            tycho::ChangeType::Update
                        },
                    });
                }
            }
        }

        // Extract balance changes
        let mut balance_changes = block_tx
            .calls
            .iter()
            .filter(|call| !call.state_reverted)
            .flat_map(|call| {
                call.balance_changes
                    .iter()
                    .filter(|c| changed_contracts.contains_key(&c.address))
            })
            .collect::<Vec<_>>();
        balance_changes.sort_unstable_by_key(|change| change.ordinal);

        for balance_change in balance_changes.iter() {
            match changed_contracts.entry(balance_change.address.clone()) {
                Entry::Occupied(mut e) => {
                    let contract_change = e.get_mut();
                    if let Some(new_balance) = &balance_change.new_value {
                        contract_change.balance.clear();
                        contract_change
                            .balance
                            .extend_from_slice(&new_balance.bytes);
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
                                tycho::ChangeType::Creation
                            } else {
                                tycho::ChangeType::Update
                            },
                        });
                    }
                }
            }
        }

        // Extract code changes
        let mut code_changes = block_tx
            .calls
            .iter()
            .filter(|call| !call.state_reverted)
            .flat_map(|call| {
                call.code_changes
                    .iter()
                    .filter(|c| changed_contracts.contains_key(&c.address))
            })
            .collect::<Vec<_>>();
        code_changes.sort_unstable_by_key(|change| change.ordinal);

        for code_change in code_changes.iter() {
            match changed_contracts.entry(code_change.address.clone()) {
                Entry::Occupied(mut e) => {
                    let contract_change = e.get_mut();
                    contract_change.code.clear();
                    contract_change
                        .code
                        .extend_from_slice(&code_change.new_code);
                }
                Entry::Vacant(e) => {
                    e.insert(InterimContractChange {
                        address: code_change.address.clone(),
                        balance: Vec::new(),
                        code: code_change.new_code.clone(),
                        slots: HashMap::new(),
                        change: if created_accounts.contains_key(&code_change.address) {
                            tycho::ChangeType::Creation
                        } else {
                            tycho::ChangeType::Update
                        },
                    });
                }
            }
        }

        if !storage_changes.is_empty() || !balance_changes.is_empty() || !code_changes.is_empty() {
            tx_change.tx = Some(tycho::Transaction {
                hash: block_tx.hash.clone(),
                from: block_tx.from.clone(),
                to: block_tx.to.clone(),
                index: block_tx.index as u64,
            });

            for (_, change) in changed_contracts.drain() {
                tx_change
                    .contract_changes
                    .push(change.into())
            }

            block_changes
                .changes
                .push(tx_change.clone());

            tx_change.tx = None;
            tx_change.contract_changes.clear();
        }
    }

    block_changes.block = Some(tycho::Block {
        number: block.number,
        hash: block.hash.clone(),
        parent_hash: block
            .header
            .as_ref()
            .expect("Block header not present")
            .parent_hash
            .clone(),
        ts: block.timestamp_seconds(),
    });

    Ok(block_changes)
}