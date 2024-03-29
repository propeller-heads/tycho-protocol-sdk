// @generated
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Events {
    #[prost(message, repeated, tag="1")]
    pub factory_pair_createds: ::prost::alloc::vec::Vec<FactoryPairCreated>,
    #[prost(message, repeated, tag="2")]
    pub pair_approvals: ::prost::alloc::vec::Vec<PairApproval>,
    #[prost(message, repeated, tag="3")]
    pub pair_burns: ::prost::alloc::vec::Vec<PairBurn>,
    #[prost(message, repeated, tag="4")]
    pub pair_cancel_long_term_orders: ::prost::alloc::vec::Vec<PairCancelLongTermOrder>,
    #[prost(message, repeated, tag="5")]
    pub pair_long_term_swap0_to_1s: ::prost::alloc::vec::Vec<PairLongTermSwap0To1>,
    #[prost(message, repeated, tag="6")]
    pub pair_long_term_swap1_to_0s: ::prost::alloc::vec::Vec<PairLongTermSwap1To0>,
    #[prost(message, repeated, tag="7")]
    pub pair_lp_fee_updateds: ::prost::alloc::vec::Vec<PairLpFeeUpdated>,
    #[prost(message, repeated, tag="8")]
    pub pair_mints: ::prost::alloc::vec::Vec<PairMint>,
    #[prost(message, repeated, tag="9")]
    pub pair_swaps: ::prost::alloc::vec::Vec<PairSwap>,
    #[prost(message, repeated, tag="10")]
    pub pair_syncs: ::prost::alloc::vec::Vec<PairSync>,
    #[prost(message, repeated, tag="11")]
    pub pair_transfers: ::prost::alloc::vec::Vec<PairTransfer>,
    #[prost(message, repeated, tag="12")]
    pub pair_withdraw_proceeds_from_long_term_orders: ::prost::alloc::vec::Vec<PairWithdrawProceedsFromLongTermOrder>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Calls {
    #[prost(message, repeated, tag="1")]
    pub factory_call_create_pair_1s: ::prost::alloc::vec::Vec<FactoryCreatePair1Call>,
    #[prost(message, repeated, tag="2")]
    pub factory_call_create_pair_2s: ::prost::alloc::vec::Vec<FactoryCreatePair2Call>,
    #[prost(message, repeated, tag="3")]
    pub factory_call_set_fee_tos: ::prost::alloc::vec::Vec<FactorySetFeeToCall>,
    #[prost(message, repeated, tag="4")]
    pub factory_call_set_fee_to_setters: ::prost::alloc::vec::Vec<FactorySetFeeToSetterCall>,
    #[prost(message, repeated, tag="5")]
    pub factory_call_toggle_global_pauses: ::prost::alloc::vec::Vec<FactoryToggleGlobalPauseCall>,
    #[prost(message, repeated, tag="6")]
    pub pair_call_approves: ::prost::alloc::vec::Vec<PairApproveCall>,
    #[prost(message, repeated, tag="7")]
    pub pair_call_burns: ::prost::alloc::vec::Vec<PairBurnCall>,
    #[prost(message, repeated, tag="8")]
    pub pair_call_cancel_long_term_swaps: ::prost::alloc::vec::Vec<PairCancelLongTermSwapCall>,
    #[prost(message, repeated, tag="9")]
    pub pair_call_execute_virtual_orders: ::prost::alloc::vec::Vec<PairExecuteVirtualOrdersCall>,
    #[prost(message, repeated, tag="10")]
    pub pair_call_get_twamm_order_proceeds: ::prost::alloc::vec::Vec<PairGetTwammOrderProceedsCall>,
    #[prost(message, repeated, tag="11")]
    pub pair_call_initializes: ::prost::alloc::vec::Vec<PairInitializeCall>,
    #[prost(message, repeated, tag="12")]
    pub pair_call_long_term_swap_from0_to_1s: ::prost::alloc::vec::Vec<PairLongTermSwapFrom0To1Call>,
    #[prost(message, repeated, tag="13")]
    pub pair_call_long_term_swap_from1_to_0s: ::prost::alloc::vec::Vec<PairLongTermSwapFrom1To0Call>,
    #[prost(message, repeated, tag="14")]
    pub pair_call_mints: ::prost::alloc::vec::Vec<PairMintCall>,
    #[prost(message, repeated, tag="15")]
    pub pair_call_permits: ::prost::alloc::vec::Vec<PairPermitCall>,
    #[prost(message, repeated, tag="16")]
    pub pair_call_set_fees: ::prost::alloc::vec::Vec<PairSetFeeCall>,
    #[prost(message, repeated, tag="17")]
    pub pair_call_skims: ::prost::alloc::vec::Vec<PairSkimCall>,
    #[prost(message, repeated, tag="18")]
    pub pair_call_swaps: ::prost::alloc::vec::Vec<PairSwapCall>,
    #[prost(message, repeated, tag="19")]
    pub pair_call_syncs: ::prost::alloc::vec::Vec<PairSyncCall>,
    #[prost(message, repeated, tag="20")]
    pub pair_call_toggle_pause_new_swaps: ::prost::alloc::vec::Vec<PairTogglePauseNewSwapsCall>,
    #[prost(message, repeated, tag="21")]
    pub pair_call_transfers: ::prost::alloc::vec::Vec<PairTransferCall>,
    #[prost(message, repeated, tag="22")]
    pub pair_call_transfer_froms: ::prost::alloc::vec::Vec<PairTransferFromCall>,
    #[prost(message, repeated, tag="23")]
    pub pair_call_withdraw_proceeds_from_long_term_swaps: ::prost::alloc::vec::Vec<PairWithdrawProceedsFromLongTermSwapCall>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FactoryPairCreated {
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
    pub pair: ::prost::alloc::vec::Vec<u8>,
    #[prost(string, tag="8")]
    pub param3: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FactoryCreatePair1Call {
    #[prost(string, tag="1")]
    pub call_tx_hash: ::prost::alloc::string::String,
    #[prost(message, optional, tag="2")]
    pub call_block_time: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(uint64, tag="3")]
    pub call_block_number: u64,
    #[prost(uint64, tag="4")]
    pub call_ordinal: u64,
    #[prost(bool, tag="5")]
    pub call_success: bool,
    #[prost(bytes="vec", tag="6")]
    pub token_a: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="7")]
    pub token_b: ::prost::alloc::vec::Vec<u8>,
    #[prost(string, tag="8")]
    pub fee: ::prost::alloc::string::String,
    #[prost(bytes="vec", tag="9")]
    pub output_pair: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FactoryCreatePair2Call {
    #[prost(string, tag="1")]
    pub call_tx_hash: ::prost::alloc::string::String,
    #[prost(message, optional, tag="2")]
    pub call_block_time: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(uint64, tag="3")]
    pub call_block_number: u64,
    #[prost(uint64, tag="4")]
    pub call_ordinal: u64,
    #[prost(bool, tag="5")]
    pub call_success: bool,
    #[prost(bytes="vec", tag="6")]
    pub token_a: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="7")]
    pub token_b: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="8")]
    pub output_pair: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FactorySetFeeToCall {
    #[prost(string, tag="1")]
    pub call_tx_hash: ::prost::alloc::string::String,
    #[prost(message, optional, tag="2")]
    pub call_block_time: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(uint64, tag="3")]
    pub call_block_number: u64,
    #[prost(uint64, tag="4")]
    pub call_ordinal: u64,
    #[prost(bool, tag="5")]
    pub call_success: bool,
    #[prost(bytes="vec", tag="6")]
    pub u_fee_to: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FactorySetFeeToSetterCall {
    #[prost(string, tag="1")]
    pub call_tx_hash: ::prost::alloc::string::String,
    #[prost(message, optional, tag="2")]
    pub call_block_time: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(uint64, tag="3")]
    pub call_block_number: u64,
    #[prost(uint64, tag="4")]
    pub call_ordinal: u64,
    #[prost(bool, tag="5")]
    pub call_success: bool,
    #[prost(bytes="vec", tag="6")]
    pub u_fee_to_setter: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FactoryToggleGlobalPauseCall {
    #[prost(string, tag="1")]
    pub call_tx_hash: ::prost::alloc::string::String,
    #[prost(message, optional, tag="2")]
    pub call_block_time: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(uint64, tag="3")]
    pub call_block_number: u64,
    #[prost(uint64, tag="4")]
    pub call_ordinal: u64,
    #[prost(bool, tag="5")]
    pub call_success: bool,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PairApproval {
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
pub struct PairBurn {
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
pub struct PairCancelLongTermOrder {
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
    pub addr: ::prost::alloc::vec::Vec<u8>,
    #[prost(string, tag="7")]
    pub order_id: ::prost::alloc::string::String,
    #[prost(bytes="vec", tag="8")]
    pub sell_token: ::prost::alloc::vec::Vec<u8>,
    #[prost(string, tag="9")]
    pub unsold_amount: ::prost::alloc::string::String,
    #[prost(bytes="vec", tag="10")]
    pub buy_token: ::prost::alloc::vec::Vec<u8>,
    #[prost(string, tag="11")]
    pub purchased_amount: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PairLongTermSwap0To1 {
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
    pub addr: ::prost::alloc::vec::Vec<u8>,
    #[prost(string, tag="7")]
    pub order_id: ::prost::alloc::string::String,
    #[prost(string, tag="8")]
    pub amount0_in: ::prost::alloc::string::String,
    #[prost(string, tag="9")]
    pub number_of_time_intervals: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PairLongTermSwap1To0 {
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
    pub addr: ::prost::alloc::vec::Vec<u8>,
    #[prost(string, tag="7")]
    pub order_id: ::prost::alloc::string::String,
    #[prost(string, tag="8")]
    pub amount1_in: ::prost::alloc::string::String,
    #[prost(string, tag="9")]
    pub number_of_time_intervals: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PairLpFeeUpdated {
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
    pub fee: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PairExecuteVirtualOrders {
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
pub struct PairMint {
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
pub struct PairSwap {
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
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PairSync {
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
    pub reserve0: ::prost::alloc::string::String,
    #[prost(string, tag="7")]
    pub reserve1: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PairTransfer {
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
pub struct PairWithdrawProceedsFromLongTermOrder {
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
    pub addr: ::prost::alloc::vec::Vec<u8>,
    #[prost(string, tag="7")]
    pub order_id: ::prost::alloc::string::String,
    #[prost(bytes="vec", tag="8")]
    pub proceed_token: ::prost::alloc::vec::Vec<u8>,
    #[prost(string, tag="9")]
    pub proceeds: ::prost::alloc::string::String,
    #[prost(bool, tag="10")]
    pub order_expired: bool,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PairApproveCall {
    #[prost(string, tag="1")]
    pub call_tx_hash: ::prost::alloc::string::String,
    #[prost(message, optional, tag="2")]
    pub call_block_time: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(uint64, tag="3")]
    pub call_block_number: u64,
    #[prost(uint64, tag="4")]
    pub call_ordinal: u64,
    #[prost(bool, tag="5")]
    pub call_success: bool,
    #[prost(string, tag="6")]
    pub call_address: ::prost::alloc::string::String,
    #[prost(bytes="vec", tag="7")]
    pub spender: ::prost::alloc::vec::Vec<u8>,
    #[prost(string, tag="8")]
    pub value: ::prost::alloc::string::String,
    #[prost(bool, tag="9")]
    pub output_param0: bool,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PairBurnCall {
    #[prost(string, tag="1")]
    pub call_tx_hash: ::prost::alloc::string::String,
    #[prost(message, optional, tag="2")]
    pub call_block_time: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(uint64, tag="3")]
    pub call_block_number: u64,
    #[prost(uint64, tag="4")]
    pub call_ordinal: u64,
    #[prost(bool, tag="5")]
    pub call_success: bool,
    #[prost(string, tag="6")]
    pub call_address: ::prost::alloc::string::String,
    #[prost(bytes="vec", tag="7")]
    pub to: ::prost::alloc::vec::Vec<u8>,
    #[prost(string, tag="8")]
    pub output_amount0: ::prost::alloc::string::String,
    #[prost(string, tag="9")]
    pub output_amount1: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PairCancelLongTermSwapCall {
    #[prost(string, tag="1")]
    pub call_tx_hash: ::prost::alloc::string::String,
    #[prost(message, optional, tag="2")]
    pub call_block_time: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(uint64, tag="3")]
    pub call_block_number: u64,
    #[prost(uint64, tag="4")]
    pub call_ordinal: u64,
    #[prost(bool, tag="5")]
    pub call_success: bool,
    #[prost(string, tag="6")]
    pub call_address: ::prost::alloc::string::String,
    #[prost(string, tag="7")]
    pub order_id: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PairExecuteVirtualOrdersCall {
    #[prost(string, tag="1")]
    pub call_tx_hash: ::prost::alloc::string::String,
    #[prost(message, optional, tag="2")]
    pub call_block_time: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(uint64, tag="3")]
    pub call_block_number: u64,
    #[prost(uint64, tag="4")]
    pub call_ordinal: u64,
    #[prost(bool, tag="5")]
    pub call_success: bool,
    #[prost(string, tag="6")]
    pub call_address: ::prost::alloc::string::String,
    #[prost(string, tag="7")]
    pub block_timestamp: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PairGetTwammOrderProceedsCall {
    #[prost(string, tag="1")]
    pub call_tx_hash: ::prost::alloc::string::String,
    #[prost(message, optional, tag="2")]
    pub call_block_time: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(uint64, tag="3")]
    pub call_block_number: u64,
    #[prost(uint64, tag="4")]
    pub call_ordinal: u64,
    #[prost(bool, tag="5")]
    pub call_success: bool,
    #[prost(string, tag="6")]
    pub call_address: ::prost::alloc::string::String,
    #[prost(string, tag="7")]
    pub order_id: ::prost::alloc::string::String,
    #[prost(bool, tag="8")]
    pub output_order_expired: bool,
    #[prost(string, tag="9")]
    pub output_total_reward: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PairInitializeCall {
    #[prost(string, tag="1")]
    pub call_tx_hash: ::prost::alloc::string::String,
    #[prost(message, optional, tag="2")]
    pub call_block_time: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(uint64, tag="3")]
    pub call_block_number: u64,
    #[prost(uint64, tag="4")]
    pub call_ordinal: u64,
    #[prost(bool, tag="5")]
    pub call_success: bool,
    #[prost(string, tag="6")]
    pub call_address: ::prost::alloc::string::String,
    #[prost(bytes="vec", tag="7")]
    pub u_token0: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="8")]
    pub u_token1: ::prost::alloc::vec::Vec<u8>,
    #[prost(string, tag="9")]
    pub u_fee: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PairLongTermSwapFrom0To1Call {
    #[prost(string, tag="1")]
    pub call_tx_hash: ::prost::alloc::string::String,
    #[prost(message, optional, tag="2")]
    pub call_block_time: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(uint64, tag="3")]
    pub call_block_number: u64,
    #[prost(uint64, tag="4")]
    pub call_ordinal: u64,
    #[prost(bool, tag="5")]
    pub call_success: bool,
    #[prost(string, tag="6")]
    pub call_address: ::prost::alloc::string::String,
    #[prost(string, tag="7")]
    pub amount0_in: ::prost::alloc::string::String,
    #[prost(string, tag="8")]
    pub number_of_time_intervals: ::prost::alloc::string::String,
    #[prost(string, tag="9")]
    pub output_order_id: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PairLongTermSwapFrom1To0Call {
    #[prost(string, tag="1")]
    pub call_tx_hash: ::prost::alloc::string::String,
    #[prost(message, optional, tag="2")]
    pub call_block_time: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(uint64, tag="3")]
    pub call_block_number: u64,
    #[prost(uint64, tag="4")]
    pub call_ordinal: u64,
    #[prost(bool, tag="5")]
    pub call_success: bool,
    #[prost(string, tag="6")]
    pub call_address: ::prost::alloc::string::String,
    #[prost(string, tag="7")]
    pub amount1_in: ::prost::alloc::string::String,
    #[prost(string, tag="8")]
    pub number_of_time_intervals: ::prost::alloc::string::String,
    #[prost(string, tag="9")]
    pub output_order_id: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PairMintCall {
    #[prost(string, tag="1")]
    pub call_tx_hash: ::prost::alloc::string::String,
    #[prost(message, optional, tag="2")]
    pub call_block_time: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(uint64, tag="3")]
    pub call_block_number: u64,
    #[prost(uint64, tag="4")]
    pub call_ordinal: u64,
    #[prost(bool, tag="5")]
    pub call_success: bool,
    #[prost(string, tag="6")]
    pub call_address: ::prost::alloc::string::String,
    #[prost(bytes="vec", tag="7")]
    pub to: ::prost::alloc::vec::Vec<u8>,
    #[prost(string, tag="8")]
    pub output_liquidity: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PairPermitCall {
    #[prost(string, tag="1")]
    pub call_tx_hash: ::prost::alloc::string::String,
    #[prost(message, optional, tag="2")]
    pub call_block_time: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(uint64, tag="3")]
    pub call_block_number: u64,
    #[prost(uint64, tag="4")]
    pub call_ordinal: u64,
    #[prost(bool, tag="5")]
    pub call_success: bool,
    #[prost(string, tag="6")]
    pub call_address: ::prost::alloc::string::String,
    #[prost(bytes="vec", tag="7")]
    pub owner: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="8")]
    pub spender: ::prost::alloc::vec::Vec<u8>,
    #[prost(string, tag="9")]
    pub value: ::prost::alloc::string::String,
    #[prost(string, tag="10")]
    pub deadline: ::prost::alloc::string::String,
    #[prost(uint64, tag="11")]
    pub v: u64,
    #[prost(bytes="vec", tag="12")]
    pub r: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="13")]
    pub s: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PairSetFeeCall {
    #[prost(string, tag="1")]
    pub call_tx_hash: ::prost::alloc::string::String,
    #[prost(message, optional, tag="2")]
    pub call_block_time: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(uint64, tag="3")]
    pub call_block_number: u64,
    #[prost(uint64, tag="4")]
    pub call_ordinal: u64,
    #[prost(bool, tag="5")]
    pub call_success: bool,
    #[prost(string, tag="6")]
    pub call_address: ::prost::alloc::string::String,
    #[prost(string, tag="7")]
    pub new_fee: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PairSkimCall {
    #[prost(string, tag="1")]
    pub call_tx_hash: ::prost::alloc::string::String,
    #[prost(message, optional, tag="2")]
    pub call_block_time: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(uint64, tag="3")]
    pub call_block_number: u64,
    #[prost(uint64, tag="4")]
    pub call_ordinal: u64,
    #[prost(bool, tag="5")]
    pub call_success: bool,
    #[prost(string, tag="6")]
    pub call_address: ::prost::alloc::string::String,
    #[prost(bytes="vec", tag="7")]
    pub to: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PairSwapCall {
    #[prost(string, tag="1")]
    pub call_tx_hash: ::prost::alloc::string::String,
    #[prost(message, optional, tag="2")]
    pub call_block_time: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(uint64, tag="3")]
    pub call_block_number: u64,
    #[prost(uint64, tag="4")]
    pub call_ordinal: u64,
    #[prost(bool, tag="5")]
    pub call_success: bool,
    #[prost(string, tag="6")]
    pub call_address: ::prost::alloc::string::String,
    #[prost(string, tag="7")]
    pub amount0_out: ::prost::alloc::string::String,
    #[prost(string, tag="8")]
    pub amount1_out: ::prost::alloc::string::String,
    #[prost(bytes="vec", tag="9")]
    pub to: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="10")]
    pub data: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PairSyncCall {
    #[prost(string, tag="1")]
    pub call_tx_hash: ::prost::alloc::string::String,
    #[prost(message, optional, tag="2")]
    pub call_block_time: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(uint64, tag="3")]
    pub call_block_number: u64,
    #[prost(uint64, tag="4")]
    pub call_ordinal: u64,
    #[prost(bool, tag="5")]
    pub call_success: bool,
    #[prost(string, tag="6")]
    pub call_address: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PairTogglePauseNewSwapsCall {
    #[prost(string, tag="1")]
    pub call_tx_hash: ::prost::alloc::string::String,
    #[prost(message, optional, tag="2")]
    pub call_block_time: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(uint64, tag="3")]
    pub call_block_number: u64,
    #[prost(uint64, tag="4")]
    pub call_ordinal: u64,
    #[prost(bool, tag="5")]
    pub call_success: bool,
    #[prost(string, tag="6")]
    pub call_address: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PairTransferCall {
    #[prost(string, tag="1")]
    pub call_tx_hash: ::prost::alloc::string::String,
    #[prost(message, optional, tag="2")]
    pub call_block_time: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(uint64, tag="3")]
    pub call_block_number: u64,
    #[prost(uint64, tag="4")]
    pub call_ordinal: u64,
    #[prost(bool, tag="5")]
    pub call_success: bool,
    #[prost(string, tag="6")]
    pub call_address: ::prost::alloc::string::String,
    #[prost(bytes="vec", tag="7")]
    pub to: ::prost::alloc::vec::Vec<u8>,
    #[prost(string, tag="8")]
    pub value: ::prost::alloc::string::String,
    #[prost(bool, tag="9")]
    pub output_param0: bool,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PairTransferFromCall {
    #[prost(string, tag="1")]
    pub call_tx_hash: ::prost::alloc::string::String,
    #[prost(message, optional, tag="2")]
    pub call_block_time: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(uint64, tag="3")]
    pub call_block_number: u64,
    #[prost(uint64, tag="4")]
    pub call_ordinal: u64,
    #[prost(bool, tag="5")]
    pub call_success: bool,
    #[prost(string, tag="6")]
    pub call_address: ::prost::alloc::string::String,
    #[prost(bytes="vec", tag="7")]
    pub from: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="8")]
    pub to: ::prost::alloc::vec::Vec<u8>,
    #[prost(string, tag="9")]
    pub value: ::prost::alloc::string::String,
    #[prost(bool, tag="10")]
    pub output_param0: bool,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PairWithdrawProceedsFromLongTermSwapCall {
    #[prost(string, tag="1")]
    pub call_tx_hash: ::prost::alloc::string::String,
    #[prost(message, optional, tag="2")]
    pub call_block_time: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(uint64, tag="3")]
    pub call_block_number: u64,
    #[prost(uint64, tag="4")]
    pub call_ordinal: u64,
    #[prost(bool, tag="5")]
    pub call_success: bool,
    #[prost(string, tag="6")]
    pub call_address: ::prost::alloc::string::String,
    #[prost(string, tag="7")]
    pub order_id: ::prost::alloc::string::String,
    #[prost(bool, tag="8")]
    pub output_is_expired: bool,
    #[prost(bytes="vec", tag="9")]
    pub output_reward_tkn: ::prost::alloc::vec::Vec<u8>,
    #[prost(string, tag="10")]
    pub output_total_reward: ::prost::alloc::string::String,
}
// @@protoc_insertion_point(module)
