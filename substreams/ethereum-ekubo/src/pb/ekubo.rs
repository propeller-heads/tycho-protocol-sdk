// @generated
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PoolDetails {
    #[prost(bytes="vec", tag="1")]
    pub token0: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="2")]
    pub token1: ::prost::alloc::vec::Vec<u8>,
    #[prost(fixed64, tag="3")]
    pub fee: u64,
}
// @@protoc_insertion_point(module)
