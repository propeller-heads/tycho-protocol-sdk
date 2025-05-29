use substreams_ethereum::pb::eth::v2::TransactionTrace;

use crate::pb::cowamm::Transaction;

pub use map_cowpool_creations::map_cowpool_creations;
pub use map_cowpool_binds::map_cowpool_binds;
pub use map_cowpools::map_cowpools;
pub use map_components::map_components;
pub use store_components::store_components;
pub use map_protocol_changes::map_protocol_changes;
pub use map_relative_balances::map_relative_balances;
pub use store_balances::store_balances;


#[path = "1_map_cowpool_creations.rs"]
mod map_cowpool_creations;

#[path = "2_map_cowpool_binds.rs"]
mod map_cowpool_binds;

#[path = "3_map_cowpools.rs"]
mod map_cowpools;

#[path = "4_map_components.rs"]
mod map_components;

#[path = "5_store_components.rs"]
mod store_components;

#[path = "6_map_relative_balances.rs"]
mod map_relative_balances;

#[path = "7_store_balances.rs"]
mod store_balances;

#[path = "8_map_protocol_changes.rs"]
mod map_protocol_changes;
mod utils;

// we add a custom trait to convert the TransactionTrace type to the Transaction 
// type that tycho protocol uses 
impl From<TransactionTrace> for Transaction {
    fn from(value: TransactionTrace) -> Self {
        Self { hash: value.hash, from: value.from, to: value.to, index: value.index.into() }
    }
}

impl From<&TransactionTrace> for Transaction {
    fn from(value: &TransactionTrace) -> Self {
        Self {
            hash: value.hash.clone(),
            from: value.from.clone(),
            to: value.to.clone(),
            index: value.index.into(),
        }
    }
}

impl From<Transaction> for tycho_substreams::prelude::Transaction {
    fn from(value: Transaction) -> Self {
        Self { hash: value.hash, from: value.from, to: value.to, index: value.index }
    }
}

impl From<&Transaction> for tycho_substreams::prelude::Transaction {
    fn from(value: &Transaction) -> Self {
        Self {
            hash: value.hash.clone(),
            from: value.from.clone(),
            to: value.to.clone(),
            index: value.index,
        }
    }
}
