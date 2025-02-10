// @generated
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RebaseEvent {
    #[prost(uint64, tag="1")]
    pub ordinal: u64,
    #[prost(string, tag="2")]
    pub epoch: ::prost::alloc::string::String,
    #[prost(string, tag="3")]
    pub total_supply: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AmplRebases {
    #[prost(message, repeated, tag="1")]
    pub rebase_events: ::prost::alloc::vec::Vec<RebaseEvent>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AmplGonBalanceDelta {
    #[prost(uint64, tag="1")]
    pub ordinal: u64,
    #[prost(string, tag="2")]
    pub delta: ::prost::alloc::string::String,
    #[prost(bytes="vec", tag="3")]
    pub component_id: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AmplGonBalanceDeltas {
    #[prost(message, repeated, tag="1")]
    pub balance_deltas: ::prost::alloc::vec::Vec<AmplGonBalanceDelta>,
}
// @@protoc_insertion_point(module)
