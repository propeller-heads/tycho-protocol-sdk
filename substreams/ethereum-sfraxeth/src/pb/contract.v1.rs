// @generated
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RewardCycle {
    #[prost(uint64, tag="1")]
    pub ord: u64,
    #[prost(bytes="vec", tag="2")]
    pub next_reward_amount: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="3")]
    pub vault_address: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BlockRewardCycles {
    #[prost(message, repeated, tag="1")]
    pub reward_cycles: ::prost::alloc::vec::Vec<RewardCycle>,
}
// @@protoc_insertion_point(module)
