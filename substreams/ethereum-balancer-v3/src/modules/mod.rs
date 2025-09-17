pub use map_components::map_components;
pub use map_protocol_changes::map_protocol_changes;
pub use map_relative_balance::map_relative_balances;
pub use store_balances::store_balances;
pub use store_components::store_components;
pub use store_token_mapping::store_token_mapping;
pub use store_token_set::store_token_set;
use substreams::hex;
#[path = "1_map_components.rs"]
mod map_components;
#[path = "7_map_protocol_changes.rs"]
mod map_protocol_changes;
#[path = "5_map_relative_balance.rs"]
mod map_relative_balance;
#[path = "6_store_balances.rs"]
mod store_balances;
#[path = "2_store_components.rs"]
mod store_components;
#[path = "4_store_token_mapping.rs"]
mod store_token_mapping;
#[path = "3_store_token_set.rs"]
mod store_token_set;

pub const VAULT_ADDRESS: &[u8] = &hex!("bA1333333333a1BA1108E8412f11850A5C319bA9");
pub const VAULT_EXTENSION_ADDRESS: &[u8; 20] = &hex!("0E8B07657D719B86e06bF0806D6729e3D528C9A9");
pub const VAULT_EXPLORER: &[u8; 20] = &hex!("Fc2986feAB34713E659da84F3B1FA32c1da95832");
pub const VAULT_ADMIN: &[u8; 20] = &hex!("35fFB749B273bEb20F40f35EdeB805012C539864");
pub const BATCH_ROUTER_ADDRESS: &[u8; 20] = &hex!("136f1efcc3f8f88516b9e94110d56fdbfb1778d1");
pub const PERMIT_2_ADDRESS: &[u8; 20] = &hex!("000000000022D473030F116dDEE9F6B43aC78BA3");
