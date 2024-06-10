// @generated
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Events {
    #[prost(message, repeated, tag="1")]
    pub elasticfactory_config_master_updateds: ::prost::alloc::vec::Vec<ElasticfactoryConfigMasterUpdated>,
    #[prost(message, repeated, tag="2")]
    pub elasticfactory_fee_configuration_updateds: ::prost::alloc::vec::Vec<ElasticfactoryFeeConfigurationUpdated>,
    #[prost(message, repeated, tag="3")]
    pub elasticfactory_nft_manager_addeds: ::prost::alloc::vec::Vec<ElasticfactoryNftManagerAdded>,
    #[prost(message, repeated, tag="4")]
    pub elasticfactory_nft_manager_removeds: ::prost::alloc::vec::Vec<ElasticfactoryNftManagerRemoved>,
    #[prost(message, repeated, tag="5")]
    pub elasticfactory_pool_createds: ::prost::alloc::vec::Vec<ElasticfactoryPoolCreated>,
    #[prost(message, repeated, tag="6")]
    pub elasticfactory_swap_fee_enableds: ::prost::alloc::vec::Vec<ElasticfactorySwapFeeEnabled>,
    #[prost(message, repeated, tag="7")]
    pub elasticfactory_vesting_period_updateds: ::prost::alloc::vec::Vec<ElasticfactoryVestingPeriodUpdated>,
    #[prost(message, repeated, tag="8")]
    pub elasticfactory_whitelist_disableds: ::prost::alloc::vec::Vec<ElasticfactoryWhitelistDisabled>,
    #[prost(message, repeated, tag="9")]
    pub elasticfactory_whitelist_enableds: ::prost::alloc::vec::Vec<ElasticfactoryWhitelistEnabled>,
    #[prost(message, repeated, tag="10")]
    pub pool_approvals: ::prost::alloc::vec::Vec<PoolApproval>,
    #[prost(message, repeated, tag="11")]
    pub pool_burns: ::prost::alloc::vec::Vec<PoolBurn>,
    #[prost(message, repeated, tag="12")]
    pub pool_burn_r_tokens: ::prost::alloc::vec::Vec<PoolBurnRTokens>,
    #[prost(message, repeated, tag="13")]
    pub pool_flashes: ::prost::alloc::vec::Vec<PoolFlash>,
    #[prost(message, repeated, tag="14")]
    pub pool_initializes: ::prost::alloc::vec::Vec<PoolInitialize>,
    #[prost(message, repeated, tag="15")]
    pub pool_mints: ::prost::alloc::vec::Vec<PoolMint>,
    #[prost(message, repeated, tag="16")]
    pub pool_swaps: ::prost::alloc::vec::Vec<PoolSwap>,
    #[prost(message, repeated, tag="17")]
    pub pool_transfers: ::prost::alloc::vec::Vec<PoolTransfer>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ElasticfactoryConfigMasterUpdated {
    #[prost(string, tag="1")]
    pub evt_tx_hash: ::prost::alloc::string::String,
    #[prost(uint32, tag="2")]
    pub evt_index: u32,
    #[prost(message, optional, tag="3")]
    pub evt_block_time: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(uint64, tag="4")]
    pub evt_block_number: u64,
    #[prost(bytes="vec", tag="5")]
    pub old_config_master: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="6")]
    pub new_config_master: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ElasticfactoryFeeConfigurationUpdated {
    #[prost(string, tag="1")]
    pub evt_tx_hash: ::prost::alloc::string::String,
    #[prost(uint32, tag="2")]
    pub evt_index: u32,
    #[prost(message, optional, tag="3")]
    pub evt_block_time: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(uint64, tag="4")]
    pub evt_block_number: u64,
    #[prost(bytes="vec", tag="5")]
    pub fee_to: ::prost::alloc::vec::Vec<u8>,
    #[prost(uint64, tag="6")]
    pub government_fee_units: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ElasticfactoryNftManagerAdded {
    #[prost(string, tag="1")]
    pub evt_tx_hash: ::prost::alloc::string::String,
    #[prost(uint32, tag="2")]
    pub evt_index: u32,
    #[prost(message, optional, tag="3")]
    pub evt_block_time: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(uint64, tag="4")]
    pub evt_block_number: u64,
    #[prost(bytes="vec", tag="5")]
    pub u_nft_manager: ::prost::alloc::vec::Vec<u8>,
    #[prost(bool, tag="6")]
    pub added: bool,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ElasticfactoryNftManagerRemoved {
    #[prost(string, tag="1")]
    pub evt_tx_hash: ::prost::alloc::string::String,
    #[prost(uint32, tag="2")]
    pub evt_index: u32,
    #[prost(message, optional, tag="3")]
    pub evt_block_time: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(uint64, tag="4")]
    pub evt_block_number: u64,
    #[prost(bytes="vec", tag="5")]
    pub u_nft_manager: ::prost::alloc::vec::Vec<u8>,
    #[prost(bool, tag="6")]
    pub removed: bool,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ElasticfactoryPoolCreated {
    #[prost(string, tag="1")]
    pub evt_tx_hash: ::prost::alloc::string::String,
    #[prost(uint32, tag="2")]
    pub evt_index: u32,
    #[prost(message, optional, tag="3")]
    pub evt_block_time: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(uint64, tag="4")]
    pub evt_block_number: u64,
    #[prost(bytes="vec", tag="5")]
    pub token0: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="6")]
    pub token1: ::prost::alloc::vec::Vec<u8>,
    #[prost(uint64, tag="7")]
    pub swap_fee_units: u64,
    #[prost(int64, tag="8")]
    pub tick_distance: i64,
    #[prost(bytes="vec", tag="9")]
    pub pool: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ElasticfactorySwapFeeEnabled {
    #[prost(string, tag="1")]
    pub evt_tx_hash: ::prost::alloc::string::String,
    #[prost(uint32, tag="2")]
    pub evt_index: u32,
    #[prost(message, optional, tag="3")]
    pub evt_block_time: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(uint64, tag="4")]
    pub evt_block_number: u64,
    #[prost(uint64, tag="5")]
    pub swap_fee_units: u64,
    #[prost(int64, tag="6")]
    pub tick_distance: i64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ElasticfactoryVestingPeriodUpdated {
    #[prost(string, tag="1")]
    pub evt_tx_hash: ::prost::alloc::string::String,
    #[prost(uint32, tag="2")]
    pub evt_index: u32,
    #[prost(message, optional, tag="3")]
    pub evt_block_time: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(uint64, tag="4")]
    pub evt_block_number: u64,
    #[prost(uint64, tag="5")]
    pub vesting_period: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ElasticfactoryWhitelistDisabled {
    #[prost(string, tag="1")]
    pub evt_tx_hash: ::prost::alloc::string::String,
    #[prost(uint32, tag="2")]
    pub evt_index: u32,
    #[prost(message, optional, tag="3")]
    pub evt_block_time: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(uint64, tag="4")]
    pub evt_block_number: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ElasticfactoryWhitelistEnabled {
    #[prost(string, tag="1")]
    pub evt_tx_hash: ::prost::alloc::string::String,
    #[prost(uint32, tag="2")]
    pub evt_index: u32,
    #[prost(message, optional, tag="3")]
    pub evt_block_time: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(uint64, tag="4")]
    pub evt_block_number: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PoolApproval {
    #[prost(string, tag="1")]
    pub evt_tx_hash: ::prost::alloc::string::String,
    #[prost(uint32, tag="2")]
    pub evt_index: u32,
    #[prost(message, optional, tag="3")]
    pub evt_block_time: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(uint64, tag="4")]
    pub evt_block_number: u64,
    #[prost(string, tag="5")]
    pub evt_address: ::prost::alloc::string::String,
    #[prost(bytes="vec", tag="6")]
    pub owner: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="7")]
    pub spender: ::prost::alloc::vec::Vec<u8>,
    #[prost(string, tag="8")]
    pub value: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PoolBurn {
    #[prost(string, tag="1")]
    pub evt_tx_hash: ::prost::alloc::string::String,
    #[prost(uint32, tag="2")]
    pub evt_index: u32,
    #[prost(message, optional, tag="3")]
    pub evt_block_time: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(uint64, tag="4")]
    pub evt_block_number: u64,
    #[prost(string, tag="5")]
    pub evt_address: ::prost::alloc::string::String,
    #[prost(bytes="vec", tag="6")]
    pub owner: ::prost::alloc::vec::Vec<u8>,
    #[prost(int64, tag="7")]
    pub tick_lower: i64,
    #[prost(int64, tag="8")]
    pub tick_upper: i64,
    #[prost(string, tag="9")]
    pub qty: ::prost::alloc::string::String,
    #[prost(string, tag="10")]
    pub qty0: ::prost::alloc::string::String,
    #[prost(string, tag="11")]
    pub qty1: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PoolBurnRTokens {
    #[prost(string, tag="1")]
    pub evt_tx_hash: ::prost::alloc::string::String,
    #[prost(uint32, tag="2")]
    pub evt_index: u32,
    #[prost(message, optional, tag="3")]
    pub evt_block_time: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(uint64, tag="4")]
    pub evt_block_number: u64,
    #[prost(string, tag="5")]
    pub evt_address: ::prost::alloc::string::String,
    #[prost(bytes="vec", tag="6")]
    pub owner: ::prost::alloc::vec::Vec<u8>,
    #[prost(string, tag="7")]
    pub qty: ::prost::alloc::string::String,
    #[prost(string, tag="8")]
    pub qty0: ::prost::alloc::string::String,
    #[prost(string, tag="9")]
    pub qty1: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PoolFlash {
    #[prost(string, tag="1")]
    pub evt_tx_hash: ::prost::alloc::string::String,
    #[prost(uint32, tag="2")]
    pub evt_index: u32,
    #[prost(message, optional, tag="3")]
    pub evt_block_time: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(uint64, tag="4")]
    pub evt_block_number: u64,
    #[prost(string, tag="5")]
    pub evt_address: ::prost::alloc::string::String,
    #[prost(bytes="vec", tag="6")]
    pub sender: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="7")]
    pub recipient: ::prost::alloc::vec::Vec<u8>,
    #[prost(string, tag="8")]
    pub qty0: ::prost::alloc::string::String,
    #[prost(string, tag="9")]
    pub qty1: ::prost::alloc::string::String,
    #[prost(string, tag="10")]
    pub paid0: ::prost::alloc::string::String,
    #[prost(string, tag="11")]
    pub paid1: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PoolInitialize {
    #[prost(string, tag="1")]
    pub evt_tx_hash: ::prost::alloc::string::String,
    #[prost(uint32, tag="2")]
    pub evt_index: u32,
    #[prost(message, optional, tag="3")]
    pub evt_block_time: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(uint64, tag="4")]
    pub evt_block_number: u64,
    #[prost(string, tag="5")]
    pub evt_address: ::prost::alloc::string::String,
    #[prost(string, tag="6")]
    pub sqrt_p: ::prost::alloc::string::String,
    #[prost(int64, tag="7")]
    pub tick: i64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PoolMint {
    #[prost(string, tag="1")]
    pub evt_tx_hash: ::prost::alloc::string::String,
    #[prost(uint32, tag="2")]
    pub evt_index: u32,
    #[prost(message, optional, tag="3")]
    pub evt_block_time: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(uint64, tag="4")]
    pub evt_block_number: u64,
    #[prost(string, tag="5")]
    pub evt_address: ::prost::alloc::string::String,
    #[prost(bytes="vec", tag="6")]
    pub sender: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="7")]
    pub owner: ::prost::alloc::vec::Vec<u8>,
    #[prost(int64, tag="8")]
    pub tick_lower: i64,
    #[prost(int64, tag="9")]
    pub tick_upper: i64,
    #[prost(string, tag="10")]
    pub qty: ::prost::alloc::string::String,
    #[prost(string, tag="11")]
    pub qty0: ::prost::alloc::string::String,
    #[prost(string, tag="12")]
    pub qty1: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PoolSwap {
    #[prost(string, tag="1")]
    pub evt_tx_hash: ::prost::alloc::string::String,
    #[prost(uint32, tag="2")]
    pub evt_index: u32,
    #[prost(message, optional, tag="3")]
    pub evt_block_time: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(uint64, tag="4")]
    pub evt_block_number: u64,
    #[prost(string, tag="5")]
    pub evt_address: ::prost::alloc::string::String,
    #[prost(bytes="vec", tag="6")]
    pub sender: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="7")]
    pub recipient: ::prost::alloc::vec::Vec<u8>,
    #[prost(string, tag="8")]
    pub delta_qty0: ::prost::alloc::string::String,
    #[prost(string, tag="9")]
    pub delta_qty1: ::prost::alloc::string::String,
    #[prost(string, tag="10")]
    pub sqrt_p: ::prost::alloc::string::String,
    #[prost(string, tag="11")]
    pub liquidity: ::prost::alloc::string::String,
    #[prost(int64, tag="12")]
    pub current_tick: i64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PoolTransfer {
    #[prost(string, tag="1")]
    pub evt_tx_hash: ::prost::alloc::string::String,
    #[prost(uint32, tag="2")]
    pub evt_index: u32,
    #[prost(message, optional, tag="3")]
    pub evt_block_time: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(uint64, tag="4")]
    pub evt_block_number: u64,
    #[prost(string, tag="5")]
    pub evt_address: ::prost::alloc::string::String,
    #[prost(bytes="vec", tag="6")]
    pub from: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="7")]
    pub to: ::prost::alloc::vec::Vec<u8>,
    #[prost(string, tag="8")]
    pub value: ::prost::alloc::string::String,
}
// @@protoc_insertion_point(module)
