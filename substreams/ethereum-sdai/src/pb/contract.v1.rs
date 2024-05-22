// @generated
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Events {
    #[prost(message, repeated, tag="1")]
    pub sdai_approvals: ::prost::alloc::vec::Vec<SdaiApproval>,
    #[prost(message, repeated, tag="2")]
    pub sdai_deposits: ::prost::alloc::vec::Vec<SdaiDeposit>,
    #[prost(message, repeated, tag="3")]
    pub sdai_transfers: ::prost::alloc::vec::Vec<SdaiTransfer>,
    #[prost(message, repeated, tag="4")]
    pub sdai_withdraws: ::prost::alloc::vec::Vec<SdaiWithdraw>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SdaiApproval {
    #[prost(string, tag="1")]
    pub evt_tx_hash: ::prost::alloc::string::String,
    #[prost(uint32, tag="2")]
    pub evt_index: u32,
    #[prost(message, optional, tag="3")]
    pub evt_block_time: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(uint64, tag="4")]
    pub evt_block_number: u64,
    #[prost(bytes="vec", tag="5")]
    pub owner: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="6")]
    pub spender: ::prost::alloc::vec::Vec<u8>,
    #[prost(string, tag="7")]
    pub value: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SdaiDeposit {
    #[prost(string, tag="1")]
    pub evt_tx_hash: ::prost::alloc::string::String,
    #[prost(uint32, tag="2")]
    pub evt_index: u32,
    #[prost(message, optional, tag="3")]
    pub evt_block_time: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(uint64, tag="4")]
    pub evt_block_number: u64,
    #[prost(bytes="vec", tag="5")]
    pub sender: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="6")]
    pub owner: ::prost::alloc::vec::Vec<u8>,
    #[prost(string, tag="7")]
    pub assets: ::prost::alloc::string::String,
    #[prost(string, tag="8")]
    pub shares: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SdaiTransfer {
    #[prost(string, tag="1")]
    pub evt_tx_hash: ::prost::alloc::string::String,
    #[prost(uint32, tag="2")]
    pub evt_index: u32,
    #[prost(message, optional, tag="3")]
    pub evt_block_time: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(uint64, tag="4")]
    pub evt_block_number: u64,
    #[prost(bytes="vec", tag="5")]
    pub from: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="6")]
    pub to: ::prost::alloc::vec::Vec<u8>,
    #[prost(string, tag="7")]
    pub value: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SdaiWithdraw {
    #[prost(string, tag="1")]
    pub evt_tx_hash: ::prost::alloc::string::String,
    #[prost(uint32, tag="2")]
    pub evt_index: u32,
    #[prost(message, optional, tag="3")]
    pub evt_block_time: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(uint64, tag="4")]
    pub evt_block_number: u64,
    #[prost(bytes="vec", tag="5")]
    pub sender: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="6")]
    pub receiver: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="7")]
    pub owner: ::prost::alloc::vec::Vec<u8>,
    #[prost(string, tag="8")]
    pub assets: ::prost::alloc::string::String,
    #[prost(string, tag="9")]
    pub shares: ::prost::alloc::string::String,
}
// @@protoc_insertion_point(module)
