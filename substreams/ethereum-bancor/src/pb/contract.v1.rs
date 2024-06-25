// @generated
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Events {
    #[prost(message, repeated, tag="1")]
    pub pool_admin_changeds: ::prost::alloc::vec::Vec<PoolAdminChanged>,
    #[prost(message, repeated, tag="2")]
    pub pool_beacon_upgradeds: ::prost::alloc::vec::Vec<PoolBeaconUpgraded>,
    #[prost(message, repeated, tag="3")]
    pub pool_flash_loan_completeds: ::prost::alloc::vec::Vec<PoolFlashLoanCompleted>,
    #[prost(message, repeated, tag="4")]
    pub pool_funds_migrateds: ::prost::alloc::vec::Vec<PoolFundsMigrated>,
    #[prost(message, repeated, tag="5")]
    pub pool_min_network_fee_burn_updateds: ::prost::alloc::vec::Vec<PoolMinNetworkFeeBurnUpdated>,
    #[prost(message, repeated, tag="6")]
    pub pool_network_fees_burneds: ::prost::alloc::vec::Vec<PoolNetworkFeesBurned>,
    #[prost(message, repeated, tag="7")]
    pub pool_pol_rewards_ppm_updateds: ::prost::alloc::vec::Vec<PoolPolRewardsPpmUpdated>,
    #[prost(message, repeated, tag="8")]
    pub pool_pol_withdrawns: ::prost::alloc::vec::Vec<PoolPolWithdrawn>,
    #[prost(message, repeated, tag="9")]
    pub pool_pauseds: ::prost::alloc::vec::Vec<PoolPaused>,
    #[prost(message, repeated, tag="10")]
    pub pool_pool_addeds: ::prost::alloc::vec::Vec<PoolPoolAdded>,
    #[prost(message, repeated, tag="11")]
    pub pool_pool_collection_addeds: ::prost::alloc::vec::Vec<PoolPoolCollectionAdded>,
    #[prost(message, repeated, tag="12")]
    pub pool_pool_collection_removeds: ::prost::alloc::vec::Vec<PoolPoolCollectionRemoved>,
    #[prost(message, repeated, tag="13")]
    pub pool_pool_createds: ::prost::alloc::vec::Vec<PoolPoolCreated>,
    #[prost(message, repeated, tag="14")]
    pub pool_pool_removeds: ::prost::alloc::vec::Vec<PoolPoolRemoved>,
    #[prost(message, repeated, tag="15")]
    pub pool_role_admin_changeds: ::prost::alloc::vec::Vec<PoolRoleAdminChanged>,
    #[prost(message, repeated, tag="16")]
    pub pool_role_granteds: ::prost::alloc::vec::Vec<PoolRoleGranted>,
    #[prost(message, repeated, tag="17")]
    pub pool_role_revokeds: ::prost::alloc::vec::Vec<PoolRoleRevoked>,
    #[prost(message, repeated, tag="18")]
    pub pool_tokens_tradeds: ::prost::alloc::vec::Vec<PoolTokensTraded>,
    #[prost(message, repeated, tag="19")]
    pub pool_unpauseds: ::prost::alloc::vec::Vec<PoolUnpaused>,
    #[prost(message, repeated, tag="20")]
    pub pool_upgradeds: ::prost::alloc::vec::Vec<PoolUpgraded>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PoolAdminChanged {
    #[prost(string, tag="1")]
    pub evt_tx_hash: ::prost::alloc::string::String,
    #[prost(uint32, tag="2")]
    pub evt_index: u32,
    #[prost(message, optional, tag="3")]
    pub evt_block_time: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(uint64, tag="4")]
    pub evt_block_number: u64,
    #[prost(bytes="vec", tag="5")]
    pub previous_admin: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="6")]
    pub new_admin: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PoolBeaconUpgraded {
    #[prost(string, tag="1")]
    pub evt_tx_hash: ::prost::alloc::string::String,
    #[prost(uint32, tag="2")]
    pub evt_index: u32,
    #[prost(message, optional, tag="3")]
    pub evt_block_time: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(uint64, tag="4")]
    pub evt_block_number: u64,
    #[prost(bytes="vec", tag="5")]
    pub beacon: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PoolFlashLoanCompleted {
    #[prost(string, tag="1")]
    pub evt_tx_hash: ::prost::alloc::string::String,
    #[prost(uint32, tag="2")]
    pub evt_index: u32,
    #[prost(message, optional, tag="3")]
    pub evt_block_time: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(uint64, tag="4")]
    pub evt_block_number: u64,
    #[prost(bytes="vec", tag="5")]
    pub token: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="6")]
    pub borrower: ::prost::alloc::vec::Vec<u8>,
    #[prost(string, tag="7")]
    pub amount: ::prost::alloc::string::String,
    #[prost(string, tag="8")]
    pub fee_amount: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PoolFundsMigrated {
    #[prost(string, tag="1")]
    pub evt_tx_hash: ::prost::alloc::string::String,
    #[prost(uint32, tag="2")]
    pub evt_index: u32,
    #[prost(message, optional, tag="3")]
    pub evt_block_time: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(uint64, tag="4")]
    pub evt_block_number: u64,
    #[prost(bytes="vec", tag="5")]
    pub context_id: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="6")]
    pub token: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="7")]
    pub provider: ::prost::alloc::vec::Vec<u8>,
    #[prost(string, tag="8")]
    pub amount: ::prost::alloc::string::String,
    #[prost(string, tag="9")]
    pub available_amount: ::prost::alloc::string::String,
    #[prost(string, tag="10")]
    pub original_amount: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PoolMinNetworkFeeBurnUpdated {
    #[prost(string, tag="1")]
    pub evt_tx_hash: ::prost::alloc::string::String,
    #[prost(uint32, tag="2")]
    pub evt_index: u32,
    #[prost(message, optional, tag="3")]
    pub evt_block_time: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(uint64, tag="4")]
    pub evt_block_number: u64,
    #[prost(string, tag="5")]
    pub old_min_network_fee_burn: ::prost::alloc::string::String,
    #[prost(string, tag="6")]
    pub new_min_network_fee_burn: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PoolNetworkFeesBurned {
    #[prost(string, tag="1")]
    pub evt_tx_hash: ::prost::alloc::string::String,
    #[prost(uint32, tag="2")]
    pub evt_index: u32,
    #[prost(message, optional, tag="3")]
    pub evt_block_time: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(uint64, tag="4")]
    pub evt_block_number: u64,
    #[prost(bytes="vec", tag="5")]
    pub caller: ::prost::alloc::vec::Vec<u8>,
    #[prost(string, tag="6")]
    pub amount: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PoolPolRewardsPpmUpdated {
    #[prost(string, tag="1")]
    pub evt_tx_hash: ::prost::alloc::string::String,
    #[prost(uint32, tag="2")]
    pub evt_index: u32,
    #[prost(message, optional, tag="3")]
    pub evt_block_time: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(uint64, tag="4")]
    pub evt_block_number: u64,
    #[prost(uint64, tag="5")]
    pub old_rewards_ppm: u64,
    #[prost(uint64, tag="6")]
    pub new_rewards_ppm: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PoolPolWithdrawn {
    #[prost(string, tag="1")]
    pub evt_tx_hash: ::prost::alloc::string::String,
    #[prost(uint32, tag="2")]
    pub evt_index: u32,
    #[prost(message, optional, tag="3")]
    pub evt_block_time: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(uint64, tag="4")]
    pub evt_block_number: u64,
    #[prost(bytes="vec", tag="5")]
    pub caller: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="6")]
    pub token: ::prost::alloc::vec::Vec<u8>,
    #[prost(string, tag="7")]
    pub pol_token_amount: ::prost::alloc::string::String,
    #[prost(string, tag="8")]
    pub user_reward: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PoolPaused {
    #[prost(string, tag="1")]
    pub evt_tx_hash: ::prost::alloc::string::String,
    #[prost(uint32, tag="2")]
    pub evt_index: u32,
    #[prost(message, optional, tag="3")]
    pub evt_block_time: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(uint64, tag="4")]
    pub evt_block_number: u64,
    #[prost(bytes="vec", tag="5")]
    pub account: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PoolPoolAdded {
    #[prost(string, tag="1")]
    pub evt_tx_hash: ::prost::alloc::string::String,
    #[prost(uint32, tag="2")]
    pub evt_index: u32,
    #[prost(message, optional, tag="3")]
    pub evt_block_time: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(uint64, tag="4")]
    pub evt_block_number: u64,
    #[prost(bytes="vec", tag="5")]
    pub pool: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="6")]
    pub pool_collection: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PoolPoolCollectionAdded {
    #[prost(string, tag="1")]
    pub evt_tx_hash: ::prost::alloc::string::String,
    #[prost(uint32, tag="2")]
    pub evt_index: u32,
    #[prost(message, optional, tag="3")]
    pub evt_block_time: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(uint64, tag="4")]
    pub evt_block_number: u64,
    #[prost(uint64, tag="5")]
    pub pool_type: u64,
    #[prost(bytes="vec", tag="6")]
    pub pool_collection: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PoolPoolCollectionRemoved {
    #[prost(string, tag="1")]
    pub evt_tx_hash: ::prost::alloc::string::String,
    #[prost(uint32, tag="2")]
    pub evt_index: u32,
    #[prost(message, optional, tag="3")]
    pub evt_block_time: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(uint64, tag="4")]
    pub evt_block_number: u64,
    #[prost(uint64, tag="5")]
    pub pool_type: u64,
    #[prost(bytes="vec", tag="6")]
    pub pool_collection: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PoolPoolCreated {
    #[prost(string, tag="1")]
    pub evt_tx_hash: ::prost::alloc::string::String,
    #[prost(uint32, tag="2")]
    pub evt_index: u32,
    #[prost(message, optional, tag="3")]
    pub evt_block_time: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(uint64, tag="4")]
    pub evt_block_number: u64,
    #[prost(bytes="vec", tag="5")]
    pub pool: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="6")]
    pub pool_collection: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PoolPoolRemoved {
    #[prost(string, tag="1")]
    pub evt_tx_hash: ::prost::alloc::string::String,
    #[prost(uint32, tag="2")]
    pub evt_index: u32,
    #[prost(message, optional, tag="3")]
    pub evt_block_time: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(uint64, tag="4")]
    pub evt_block_number: u64,
    #[prost(bytes="vec", tag="5")]
    pub pool: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="6")]
    pub pool_collection: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PoolRoleAdminChanged {
    #[prost(string, tag="1")]
    pub evt_tx_hash: ::prost::alloc::string::String,
    #[prost(uint32, tag="2")]
    pub evt_index: u32,
    #[prost(message, optional, tag="3")]
    pub evt_block_time: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(uint64, tag="4")]
    pub evt_block_number: u64,
    #[prost(bytes="vec", tag="5")]
    pub role: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="6")]
    pub previous_admin_role: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="7")]
    pub new_admin_role: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PoolRoleGranted {
    #[prost(string, tag="1")]
    pub evt_tx_hash: ::prost::alloc::string::String,
    #[prost(uint32, tag="2")]
    pub evt_index: u32,
    #[prost(message, optional, tag="3")]
    pub evt_block_time: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(uint64, tag="4")]
    pub evt_block_number: u64,
    #[prost(bytes="vec", tag="5")]
    pub role: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="6")]
    pub account: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="7")]
    pub sender: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PoolRoleRevoked {
    #[prost(string, tag="1")]
    pub evt_tx_hash: ::prost::alloc::string::String,
    #[prost(uint32, tag="2")]
    pub evt_index: u32,
    #[prost(message, optional, tag="3")]
    pub evt_block_time: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(uint64, tag="4")]
    pub evt_block_number: u64,
    #[prost(bytes="vec", tag="5")]
    pub role: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="6")]
    pub account: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="7")]
    pub sender: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PoolTokensTraded {
    #[prost(string, tag="1")]
    pub evt_tx_hash: ::prost::alloc::string::String,
    #[prost(uint32, tag="2")]
    pub evt_index: u32,
    #[prost(message, optional, tag="3")]
    pub evt_block_time: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(uint64, tag="4")]
    pub evt_block_number: u64,
    #[prost(bytes="vec", tag="5")]
    pub context_id: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="6")]
    pub source_token: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="7")]
    pub target_token: ::prost::alloc::vec::Vec<u8>,
    #[prost(string, tag="8")]
    pub source_amount: ::prost::alloc::string::String,
    #[prost(string, tag="9")]
    pub target_amount: ::prost::alloc::string::String,
    #[prost(string, tag="10")]
    pub bnt_amount: ::prost::alloc::string::String,
    #[prost(string, tag="11")]
    pub target_fee_amount: ::prost::alloc::string::String,
    #[prost(string, tag="12")]
    pub bnt_fee_amount: ::prost::alloc::string::String,
    #[prost(bytes="vec", tag="13")]
    pub trader: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PoolUnpaused {
    #[prost(string, tag="1")]
    pub evt_tx_hash: ::prost::alloc::string::String,
    #[prost(uint32, tag="2")]
    pub evt_index: u32,
    #[prost(message, optional, tag="3")]
    pub evt_block_time: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(uint64, tag="4")]
    pub evt_block_number: u64,
    #[prost(bytes="vec", tag="5")]
    pub account: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PoolUpgraded {
    #[prost(string, tag="1")]
    pub evt_tx_hash: ::prost::alloc::string::String,
    #[prost(uint32, tag="2")]
    pub evt_index: u32,
    #[prost(message, optional, tag="3")]
    pub evt_block_time: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(uint64, tag="4")]
    pub evt_block_number: u64,
    #[prost(bytes="vec", tag="5")]
    pub implementation: ::prost::alloc::vec::Vec<u8>,
}
// @@protoc_insertion_point(module)
