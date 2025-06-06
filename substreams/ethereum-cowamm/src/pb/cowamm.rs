// @generated
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CowPool {
    #[prost(bytes="vec", tag="1")]
    pub address: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="2")]
    pub token_a: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="3")]
    pub token_b: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="4")]
    pub lp_token: ::prost::alloc::vec::Vec<u8>,
    #[prost(uint64, tag="5")]
    pub weight_a: u64,
    #[prost(uint64, tag="6")]
    pub weight_b: u64,
    #[prost(uint64, tag="7")]
    pub fee: u64,
    #[prost(bytes="vec", tag="8")]
    pub created_tx_hash: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CowPools {
    #[prost(message, repeated, tag="1")]
    pub pools: ::prost::alloc::vec::Vec<CowPool>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CowPoolBind {
    #[prost(bytes="vec", tag="1")]
    pub address: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="2")]
    pub token: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="3")]
    pub weight: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CowPoolBinds {
    #[prost(message, repeated, tag="1")]
    pub binds: ::prost::alloc::vec::Vec<CowPoolBind>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CowPoolCreation {
    #[prost(bytes="vec", tag="1")]
    pub address: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="2")]
    pub lp_token: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="3")]
    pub created_tx_hash: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CowPoolCreations {
    #[prost(message, repeated, tag="1")]
    pub pools: ::prost::alloc::vec::Vec<CowPoolCreation>,
}
// @@protoc_insertion_point(module)
