pub use map_components::map_components;
pub use map_cowpool_binds::map_cowpool_binds;
pub use map_cowpool_creations::map_cowpool_creations;
pub use map_cowpools::map_cowpools;
pub use map_protocol_changes::map_protocol_changes;
pub use map_relative_balances::map_relative_balances;
pub use store_balances::store_balances;
pub use store_components::store_components;
pub use store_cowpool_binds::store_cowpool_binds;
pub use store_cowpools::store_cowpools;
use substreams_ethereum::pb::eth::v2::TransactionTrace;
use crate::pb::cowamm::Transaction;

#[path = "1_map_cowpool_creations.rs"]
mod map_cowpool_creations;

#[path = "2_map_cowpool_binds.rs"]
mod map_cowpool_binds;

#[path = "2_store_cowpool_binds.rs"]
mod store_cowpool_binds;

// #[path = "3_map_cowpool_binds_balances.rs"]
// mod map_cowpool_binds_balances;

// #[path = "3_store_cowpool_binds_balances.rs"]
// mod store_cowpool_binds_balances;

#[path = "4_map_cowpools.rs"]
mod map_cowpools;

#[path = "4_store_cowpools.rs"]
mod store_cowpools;

#[path = "5_map_components.rs"]
mod map_components;

#[path = "5_store_components.rs"]
mod store_components;

#[path = "6_map_relative_balances.rs"]
mod map_relative_balances;

#[path = "6_store_balances.rs"]
mod store_balances;

#[path = "7_map_protocol_changes.rs"]
mod map_protocol_changes;
mod utils;

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
