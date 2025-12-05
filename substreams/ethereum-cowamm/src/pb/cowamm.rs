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
    #[prost(bytes="vec", tag="5")]
    pub weight_a: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="6")]
    pub weight_b: ::prost::alloc::vec::Vec<u8>,
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

/// A struct describing a transaction.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Transaction {
    /// The transaction hash.
    #[prost(bytes="vec", tag="1")]
    pub hash: ::prost::alloc::vec::Vec<u8>,
    /// The sender of the transaction.
    #[prost(bytes="vec", tag="2")]
    pub from: ::prost::alloc::vec::Vec<u8>,
    /// The receiver of the transaction.
    #[prost(bytes="vec", tag="3")]
    pub to: ::prost::alloc::vec::Vec<u8>,
    /// The transactions index within the block.
    /// TODO: should this be uint32? to match the type from the native substream type?
    #[prost(uint64, tag="4")]
    pub index: u64,
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
    #[prost(bytes="vec", tag="4")]
    pub amount: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag="5")]
    pub tx: ::core::option::Option<Transaction>,
    /// The block's global ordinal when the log was recorded, refer to \[Block\]
    /// documentation for further information about ordinals and total ordering.
    #[prost(uint64, tag="7")]
    pub ordinal: u64,
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
