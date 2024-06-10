// @generated
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Events {
    #[prost(message, repeated, tag="1")]
    pub factory_pool_createds: ::prost::alloc::vec::Vec<FactoryPoolCreated>,
    #[prost(message, repeated, tag="2")]
    pub factory_set_fee_configurations: ::prost::alloc::vec::Vec<FactorySetFeeConfiguration>,
    #[prost(message, repeated, tag="3")]
    pub factory_set_fee_to_setters: ::prost::alloc::vec::Vec<FactorySetFeeToSetter>,
    #[prost(message, repeated, tag="4")]
    pub pool_approvals: ::prost::alloc::vec::Vec<PoolApproval>,
    #[prost(message, repeated, tag="5")]
    pub pool_burns: ::prost::alloc::vec::Vec<PoolBurn>,
    #[prost(message, repeated, tag="6")]
    pub pool_mints: ::prost::alloc::vec::Vec<PoolMint>,
    #[prost(message, repeated, tag="7")]
    pub pool_swaps: ::prost::alloc::vec::Vec<PoolSwap>,
    #[prost(message, repeated, tag="8")]
    pub pool_syncs: ::prost::alloc::vec::Vec<PoolSync>,
    #[prost(message, repeated, tag="9")]
    pub pool_transfers: ::prost::alloc::vec::Vec<PoolTransfer>,
    #[prost(message, repeated, tag="10")]
    pub pool_update_emas: ::prost::alloc::vec::Vec<PoolUpdateEma>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FactoryPoolCreated {
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
    #[prost(bytes="vec", tag="7")]
    pub pool: ::prost::alloc::vec::Vec<u8>,
    #[prost(uint64, tag="8")]
    pub amp_bps: u64,
    #[prost(string, tag="9")]
    pub total_pool: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FactorySetFeeConfiguration {
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
    pub government_fee_bps: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FactorySetFeeToSetter {
    #[prost(string, tag="1")]
    pub evt_tx_hash: ::prost::alloc::string::String,
    #[prost(uint32, tag="2")]
    pub evt_index: u32,
    #[prost(message, optional, tag="3")]
    pub evt_block_time: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(uint64, tag="4")]
    pub evt_block_number: u64,
    #[prost(bytes="vec", tag="5")]
    pub fee_to_setter: ::prost::alloc::vec::Vec<u8>,
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
    pub sender: ::prost::alloc::vec::Vec<u8>,
    #[prost(string, tag="7")]
    pub amount0: ::prost::alloc::string::String,
    #[prost(string, tag="8")]
    pub amount1: ::prost::alloc::string::String,
    #[prost(bytes="vec", tag="9")]
    pub to: ::prost::alloc::vec::Vec<u8>,
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
    #[prost(string, tag="7")]
    pub amount0: ::prost::alloc::string::String,
    #[prost(string, tag="8")]
    pub amount1: ::prost::alloc::string::String,
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
    #[prost(string, tag="7")]
    pub amount0_in: ::prost::alloc::string::String,
    #[prost(string, tag="8")]
    pub amount1_in: ::prost::alloc::string::String,
    #[prost(string, tag="9")]
    pub amount0_out: ::prost::alloc::string::String,
    #[prost(string, tag="10")]
    pub amount1_out: ::prost::alloc::string::String,
    #[prost(bytes="vec", tag="11")]
    pub to: ::prost::alloc::vec::Vec<u8>,
    #[prost(string, tag="12")]
    pub fee_in_precision: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PoolSync {
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
    pub v_reserve0: ::prost::alloc::string::String,
    #[prost(string, tag="7")]
    pub v_reserve1: ::prost::alloc::string::String,
    #[prost(string, tag="8")]
    pub reserve0: ::prost::alloc::string::String,
    #[prost(string, tag="9")]
    pub reserve1: ::prost::alloc::string::String,
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
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PoolUpdateEma {
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
    pub short_ema: ::prost::alloc::string::String,
    #[prost(string, tag="7")]
    pub long_ema: ::prost::alloc::string::String,
    #[prost(string, tag="8")]
    pub last_block_volume: ::prost::alloc::string::String,
    #[prost(string, tag="9")]
    pub skip_block: ::prost::alloc::string::String,
}
// @@protoc_insertion_point(module)
