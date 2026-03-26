#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TwocryptoCustomImpl {
    #[prost(string, tag = "1")]
    pub implementation_id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub view_address: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub math_address: ::prost::alloc::string::String,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TwocryptoCustomImpls {
    #[prost(message, repeated, tag = "1")]
    pub impls: ::prost::alloc::vec::Vec<TwocryptoCustomImpl>,
}
