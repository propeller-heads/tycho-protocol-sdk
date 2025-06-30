// @generated
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Trade {
    #[prost(string, tag="1")]
    pub calculated_amount: ::prost::alloc::string::String,
    #[prost(string, tag="2")]
    pub gas_used: ::prost::alloc::string::String,
    #[prost(string, tag="3")]
    pub marginal_price_after_trade: ::prost::alloc::string::String,
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
    #[prost(uint32, tag="4")]
    pub index: u32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FunctionCall {
    #[prost(string, tag="1")]
    pub id: ::prost::alloc::string::String,
    #[prost(string, tag="2")]
    pub sell_token: ::prost::alloc::string::String,
    #[prost(string, tag="3")]
    pub buy_token: ::prost::alloc::string::String,
    #[prost(message, optional, tag="4")]
    pub transaction: ::core::option::Option<Transaction>,
    #[prost(oneof="function_call::CallType", tags="5, 6, 7")]
    pub call_type: ::core::option::Option<function_call::CallType>,
}
/// Nested message and enum types in `FunctionCall`.
pub mod function_call {
    #[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum CallType {
        #[prost(message, tag="5")]
        Swap(super::SwapCallData),
        #[prost(message, tag="6")]
        Price(super::PriceCallData),
        #[prost(message, tag="7")]
        SwapToPrice(super::SwapToPriceCallData),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SwapCallData {
    #[prost(enumeration="OrderSide", tag="1")]
    pub side: i32,
    #[prost(uint64, tag="2")]
    pub specified_amount: u64,
    #[prost(message, optional, tag="3")]
    pub result: ::core::option::Option<Trade>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PriceCallData {
    #[prost(bytes="vec", repeated, tag="1")]
    pub specified_amounts: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
    #[prost(string, repeated, tag="2")]
    pub prices: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SwapToPriceCallData {
    #[prost(string, tag="1")]
    pub limit_price: ::prost::alloc::string::String,
    #[prost(message, optional, tag="2")]
    pub result: ::core::option::Option<Trade>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FunctionCalls {
    #[prost(message, repeated, tag="1")]
    pub calls: ::prost::alloc::vec::Vec<FunctionCall>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum OrderSide {
    Sell = 0,
    Buy = 1,
}
impl OrderSide {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            OrderSide::Sell => "SELL",
            OrderSide::Buy => "BUY",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "SELL" => Some(Self::Sell),
            "BUY" => Some(Self::Buy),
            _ => None,
        }
    }
}
// @@protoc_insertion_point(module)
