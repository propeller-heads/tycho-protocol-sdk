// @generated
// This file contains the proto definitions for Substreams common to all integrations.

/// A struct describing a block.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Block {
    /// The blocks hash.
    #[prost(bytes="vec", tag="1")]
    pub hash: ::prost::alloc::vec::Vec<u8>,
    /// The parent blocks hash.
    #[prost(bytes="vec", tag="2")]
    pub parent_hash: ::prost::alloc::vec::Vec<u8>,
    /// The block number.
    #[prost(uint64, tag="3")]
    pub number: u64,
    /// The block timestamp.
    #[prost(uint64, tag="4")]
    pub ts: u64,
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
    #[prost(uint64, tag="4")]
    pub index: u64,
}
/// A custom struct representing an arbitrary attribute of a protocol component.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Attribute {
    /// The name of the attribute.
    #[prost(string, tag="1")]
    pub name: ::prost::alloc::string::String,
    /// The value of the attribute.
    #[prost(bytes="vec", tag="2")]
    pub value: ::prost::alloc::vec::Vec<u8>,
    /// The type of change the attribute underwent.
    #[prost(enumeration="ChangeType", tag="3")]
    pub change: i32,
}
/// A struct describing a part of the protocol.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProtocolComponent {
    /// A unique identifier for the component within the protocol.
    /// Can be a stringified address or a string describing the trading pair.
    #[prost(string, tag="1")]
    pub id: ::prost::alloc::string::String,
    /// Addresses of the ERC20 tokens used by the component.
    #[prost(bytes="vec", repeated, tag="2")]
    pub tokens: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
    /// Addresses of the contracts used by the component.
    #[prost(bytes="vec", repeated, tag="3")]
    pub contracts: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
    /// Attributes of the component.
    /// The inner ChangeType of the attribute has to match the ChangeType of the ProtocolComponent.
    #[prost(message, repeated, tag="4")]
    pub static_att: ::prost::alloc::vec::Vec<Attribute>,
    /// Type of change the component underwent.
    #[prost(enumeration="ChangeType", tag="5")]
    pub change: i32,
}
/// A struct for following the changes of Total Value Locked (TVL) of a protocol component.
/// Note that if the ProtocolComponent contains multiple contracts, the TVL is tracked for the component as a whole.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BalanceChange {
    /// The address of the ERC20 token whose balance changed.
    #[prost(bytes="vec", tag="1")]
    pub token: ::prost::alloc::vec::Vec<u8>,
    /// The new balance of the token.
    #[prost(bytes="vec", tag="2")]
    pub balance: ::prost::alloc::vec::Vec<u8>,
    /// The id of the component whose TVL is tracked.
    #[prost(bytes="vec", tag="3")]
    pub component_id: ::prost::alloc::vec::Vec<u8>,
}
/// Enum to specify the type of a change.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum ChangeType {
    Unspecified = 0,
    Update = 1,
    Creation = 2,
    Deletion = 3,
}
impl ChangeType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            ChangeType::Unspecified => "CHANGE_TYPE_UNSPECIFIED",
            ChangeType::Update => "CHANGE_TYPE_UPDATE",
            ChangeType::Creation => "CHANGE_TYPE_CREATION",
            ChangeType::Deletion => "CHANGE_TYPE_DELETION",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "CHANGE_TYPE_UNSPECIFIED" => Some(Self::Unspecified),
            "CHANGE_TYPE_UPDATE" => Some(Self::Update),
            "CHANGE_TYPE_CREATION" => Some(Self::Creation),
            "CHANGE_TYPE_DELETION" => Some(Self::Deletion),
            _ => None,
        }
    }
}
// This file contains proto definitions specific to the VM integration.

/// A key value entry into contract storage.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ContractSlot {
    /// A contract's storage slot.
    #[prost(bytes="vec", tag="2")]
    pub slot: ::prost::alloc::vec::Vec<u8>,
    /// The new value for this storage slot.
    #[prost(bytes="vec", tag="3")]
    pub value: ::prost::alloc::vec::Vec<u8>,
}
/// Changes made to a single contract's state.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ContractChange {
    /// The contract's address
    #[prost(bytes="vec", tag="1")]
    pub address: ::prost::alloc::vec::Vec<u8>,
    /// The new balance of the contract, empty bytes indicates no change.
    #[prost(bytes="vec", tag="2")]
    pub balance: ::prost::alloc::vec::Vec<u8>,
    /// The new code of the contract, empty bytes indicates no change.
    #[prost(bytes="vec", tag="3")]
    pub code: ::prost::alloc::vec::Vec<u8>,
    /// The changes to this contract's slots, empty sequence indicates no change.
    #[prost(message, repeated, tag="4")]
    pub slots: ::prost::alloc::vec::Vec<ContractSlot>,
    /// Whether this is an update, a creation or a deletion.
    #[prost(enumeration="ChangeType", tag="5")]
    pub change: i32,
}
/// A set of changes aggregated by transaction.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TransactionContractChanges {
    /// The transaction instance that results in the changes.
    #[prost(message, optional, tag="1")]
    pub tx: ::core::option::Option<Transaction>,
    /// Contains the changes induced by the above transaction, aggregated on a per-contract basis.
    #[prost(message, repeated, tag="2")]
    pub contract_changes: ::prost::alloc::vec::Vec<ContractChange>,
    /// An array of newly added components.
    #[prost(message, repeated, tag="3")]
    pub component_changes: ::prost::alloc::vec::Vec<ProtocolComponent>,
    /// An array of balance changes to components.
    #[prost(message, repeated, tag="4")]
    pub balance_changes: ::prost::alloc::vec::Vec<BalanceChange>,
}
/// A set of transaction changes within a single block.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BlockContractChanges {
    /// The block for which these changes are collectively computed.
    #[prost(message, optional, tag="1")]
    pub block: ::core::option::Option<Block>,
    /// The set of transaction changes observed in the specified block.
    #[prost(message, repeated, tag="2")]
    pub changes: ::prost::alloc::vec::Vec<TransactionContractChanges>,
}
// @@protoc_insertion_point(module)