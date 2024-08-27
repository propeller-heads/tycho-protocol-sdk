use std::collections::HashMap;
use substreams_ethereum::pb::eth::v2::{self as sf, StorageChange};

// re-export the protobuf types here.
pub use crate::pb::tycho::evm::v1::*;

impl TransactionContractChanges {
    /// Creates a new empty `TransactionContractChanges` instance.
    pub fn new(tx: &Transaction) -> Self {
        Self { tx: Some(tx.clone()), ..Default::default() }
    }
}

impl TransactionChanges {
    /// Creates a new empty `TransactionChanges` instance.
    pub fn new(tx: &Transaction) -> Self {
        Self { tx: Some(tx.clone()), ..Default::default() }
    }
}

/// Builds `TransactionChanges` struct
///
/// Ensures uniqueness for contract addresses and component ids.
#[derive(Default)]
pub struct TransactionChangesBuilder {
    tx: Option<Transaction>,
    contract_changes: HashMap<Vec<u8>, InterimContractChange>,
    entity_changes: HashMap<String, InterimEntityChanges>,
    component_changes: HashMap<String, ProtocolComponent>,
    balance_changes: HashMap<(Vec<u8>, Vec<u8>), BalanceChange>,
}

impl TransactionChangesBuilder {
    /// Initialize a new builder for a transaction.
    pub fn new(tx: &Transaction) -> Self {
        Self { tx: Some(tx.clone()), ..Default::default() }
    }

    /// Register a new contract change.
    ///
    /// Will prioritize the new change over any already present one.
    pub fn add_contract_changes(&mut self, change: &InterimContractChange) {
        self.contract_changes
            .entry(change.address.clone())
            .and_modify(|c| {
                if !change.balance.is_empty() {
                    c.set_balance(&change.balance)
                }
                if !change.slots.is_empty() {
                    c.upsert_slots(&change.slots)
                }
                if !change.code.is_empty() {
                    c.set_code(&change.code)
                }
            })
            .or_insert_with(|| {
                let mut c = InterimContractChange::new(
                    &change.address,
                    change.change == ChangeType::Creation,
                );
                c.upsert_slots(&change.slots);
                c.set_code(&change.code);
                c.set_balance(&change.balance);
                c
            });
    }

    /// Unique contract/account addresses that have been changed so far.
    pub fn changed_contracts(&self) -> impl Iterator<Item = &[u8]> {
        self.contract_changes
            .keys()
            .map(|k| k.as_slice())
    }

    /// Marks a component as updated.
    ///
    /// If the protocol does not follow a 1:1 logic between components and contracts.
    /// Components can be manually marked as updated using this method.
    pub fn mark_component_as_updated(&mut self, component_id: &str) {
        let attr = Attribute {
            name: "update_marker".to_string(),
            value: vec![1u8],
            change: ChangeType::Update.into(),
        };
        if let Some(entry) = self
            .entity_changes
            .get_mut(component_id)
        {
            entry.set_attribute(&attr);
        } else {
            let mut change = InterimEntityChanges::new(component_id);
            change.set_attribute(&attr);
            self.entity_changes
                .insert(component_id.to_string(), change);
        }
    }

    /// Registers a new entity change.
    ///
    /// Will prioritize the new change over any already present one.
    pub fn add_entity_change(&mut self, change: &EntityChanges) {
        self.entity_changes
            .entry(change.component_id.clone())
            .and_modify(|ec| {
                for attr in change.attributes.iter() {
                    ec.set_attribute(attr);
                }
            })
            .or_insert_with(|| InterimEntityChanges {
                component_id: change.component_id.clone(),
                attributes: change
                    .attributes
                    .clone()
                    .into_iter()
                    .map(|a| (a.name.clone(), a))
                    .collect(),
            });
    }

    /// Adds a new protocol component.
    ///
    /// ## Note
    /// This method is a noop, in case the component is already present. Since
    /// components are assumed to be immutable.
    pub fn add_protocol_component(&mut self, component: &ProtocolComponent) {
        if !self
            .component_changes
            .contains_key(&component.id)
        {
            self.component_changes
                .insert(component.id.clone(), component.clone());
        }
    }

    /// Updates a components balances
    ///
    /// Overwrites any previous balance changes of the component if present.
    pub fn add_balance_change(&mut self, change: &BalanceChange) {
        self.balance_changes
            .insert((change.component_id.clone(), change.token.clone()), change.clone());
    }

    pub fn build(self) -> Option<TransactionChanges> {
        if self.contract_changes.is_empty() &&
            self.component_changes.is_empty() &&
            self.balance_changes.is_empty() &&
            self.entity_changes.is_empty()
        {
            None
        } else {
            Some(TransactionChanges {
                tx: self.tx,
                contract_changes: self
                    .contract_changes
                    .into_values()
                    .map(|interim| interim.into())
                    .collect::<Vec<_>>(),
                entity_changes: self
                    .entity_changes
                    .into_values()
                    .map(|interim| interim.into())
                    .collect::<Vec<_>>(),
                component_changes: self
                    .component_changes
                    .into_values()
                    .collect::<Vec<_>>(),
                balance_changes: self
                    .balance_changes
                    .into_values()
                    .collect::<Vec<_>>(),
            })
        }
    }
}

impl From<&sf::TransactionTrace> for Transaction {
    fn from(tx: &sf::TransactionTrace) -> Self {
        Self {
            hash: tx.hash.clone(),
            from: tx.from.clone(),
            to: tx.to.clone(),
            index: tx.index.into(),
        }
    }
}

impl From<&sf::Block> for Block {
    fn from(block: &sf::Block) -> Self {
        Self {
            number: block.number,
            hash: block.hash.clone(),
            parent_hash: block
                .header
                .as_ref()
                .expect("Block header not present")
                .parent_hash
                .clone(),
            ts: block.timestamp_seconds(),
        }
    }
}

impl ProtocolComponent {
    /// Constructs a new, empty `ProtocolComponent`.
    ///
    /// Initializes an instance with default values. Use `with_*` methods to populate fields
    /// conveniently.
    ///
    /// ## Parameters
    /// - `id`: Identifier for the component.
    /// - `tx`: Reference to the associated transaction.
    pub fn new(id: &str, tx: &Transaction) -> Self {
        Self {
            id: id.to_string(),
            tokens: Vec::new(),
            contracts: Vec::new(),
            static_att: Vec::new(),
            change: ChangeType::Creation.into(),
            protocol_type: None,
            tx: Some(tx.clone()),
        }
    }

    /// Initializes a `ProtocolComponent` with a direct association to a contract.
    ///
    /// Sets the component's ID to the hex-encoded address with a `0x` prefix and includes the
    /// contract in the contracts list.
    ///
    /// ## Parameters
    /// - `id`: Contract address to be encoded and set as the component's ID.
    /// - `tx`: Reference to the associated transaction.
    pub fn at_contract(id: &[u8], tx: &Transaction) -> Self {
        Self {
            id: format!("0x{}", hex::encode(id)),
            tokens: Vec::new(),
            contracts: vec![id.to_vec()],
            static_att: Vec::new(),
            change: ChangeType::Creation.into(),
            protocol_type: None,
            tx: Some(tx.clone()),
        }
    }

    /// Updates the tokens associated with this component.
    ///
    /// ## Parameters
    /// - `tokens`: Slice of byte slices representing the tokens to associate.
    pub fn with_tokens<B: AsRef<[u8]>>(mut self, tokens: &[B]) -> Self {
        self.tokens = tokens
            .iter()
            .map(|e| e.as_ref().to_vec())
            .collect::<Vec<Vec<u8>>>();
        self
    }

    /// Updates the contracts associated with this component.
    ///
    /// ## Parameters
    /// - `contracts`: Slice of byte slices representing the contracts to associate.
    pub fn with_contracts<B: AsRef<[u8]>>(mut self, contracts: &[B]) -> Self {
        self.contracts = contracts
            .iter()
            .map(|e| e.as_ref().to_vec())
            .collect::<Vec<Vec<u8>>>();
        self
    }

    /// Updates the static attributes of this component.
    ///
    /// Sets the change type to `Creation` for all attributes.
    ///
    /// ## Parameters
    /// - `attributes`: Slice of key-value pairs representing the attributes to set.
    pub fn with_attributes<K: AsRef<str>, V: AsRef<[u8]>>(mut self, attributes: &[(K, V)]) -> Self {
        self.static_att = attributes
            .iter()
            .map(|(k, v)| Attribute {
                name: k.as_ref().to_string(),
                value: v.as_ref().to_vec(),
                change: ChangeType::Creation.into(),
            })
            .collect::<Vec<Attribute>>();
        self
    }

    /// Designates this component as a swap type within the protocol.
    ///
    /// Sets the `protocol_type` accordingly, including `financial_type` as `Swap` and leaving
    /// `attribute_schema` empty.
    ///
    /// ## Parameters
    /// - `name`: The name of the swap protocol.
    /// - `implementation_type`: The implementation type of the protocol.
    pub fn as_swap_type(mut self, name: &str, implementation_type: ImplementationType) -> Self {
        self.protocol_type = Some(ProtocolType {
            name: name.to_string(),
            financial_type: FinancialType::Swap.into(),
            attribute_schema: Vec::new(),
            implementation_type: implementation_type.into(),
        });
        self
    }

    /// Checks if the instance contains all specified attributes.
    ///
    /// This function verifies whether the `ProtocolComponent` has all the given static attributes.
    /// Each attribute is represented by a tuple containing a name and a value. The function
    /// iterates over the provided attributes and checks if they exist in the instance's
    /// `static_att`.
    ///
    /// # Arguments
    ///
    /// * `attributes` - A slice of tuples where each tuple consists of a `String` representing the
    ///   attribute name and a `Vec<u8>` representing the attribute value.
    ///
    /// # Returns
    ///
    /// A boolean indicating whether all specified attributes are present in the instance.
    ///
    /// # Example
    ///
    /// ```
    /// let attributes_to_check = vec![
    ///     ("attribute1".to_string(), vec![1, 2, 3]),
    ///     ("attribute2".to_string(), vec![4, 5, 6]),
    /// ];
    ///
    /// let has_all_attributes = instance.has_attributes(&attributes_to_check);
    /// assert!(has_all_attributes);
    /// ```
    ///
    /// # Notes
    ///
    /// - The function assumes that the `static_att` collection contains attributes with a
    ///   `ChangeType` of `Creation` when they are initially added. This is fine because
    ///   `static_att` can't be updated
    pub fn has_attributes(&self, attributes: &[(&str, Vec<u8>)]) -> bool {
        attributes.iter().all(|(name, value)| {
            self.static_att.contains(&Attribute {
                name: name.to_string(),
                value: value.clone(),
                change: ChangeType::Creation.into(),
            })
        })
    }

    /// Retrieves the value of a specified attribute by name.
    ///
    /// This function searches the instance's `static_att` collection for an attribute with the
    /// given name. If found, it returns a copy of the attribute's value. If the attribute is
    /// not found, it returns `None`.
    ///
    /// # Arguments
    ///
    /// * `name` - A string slice that holds the name of the attribute to be searched.
    ///
    /// # Returns
    ///
    /// An `Option<Vec<u8>>` containing the attribute value if found, or `None` if the attribute
    /// does not exist.
    ///
    /// # Example
    ///
    /// ```
    /// let attribute_name = "attribute1";
    /// if let Some(value) = instance.get_attribute_value(attribute_name) {
    ///     // Use the attribute value
    ///     println!("Attribute value: {:?}", value);
    /// } else {
    ///     println!("Attribute not found");
    /// }
    /// ```
    ///
    /// # Notes
    ///
    /// - The function performs a search based on the attribute name and returns the first match
    ///   found. If there are multiple attributes with the same name, only the first one is
    ///   returned.
    pub fn get_attribute_value(&self, name: &str) -> Option<Vec<u8>> {
        self.static_att
            .iter()
            .find(|attr| attr.name == name)
            .map(|attr| attr.value.clone())
    }
}

/// Same as `EntityChanges` but ensures attributes are unique by name.
#[derive(Default)]
pub struct InterimEntityChanges {
    component_id: String,
    attributes: HashMap<String, Attribute>,
}

impl InterimEntityChanges {
    pub fn new(id: &str) -> Self {
        Self { component_id: id.to_string(), ..Default::default() }
    }

    pub fn set_attribute(&mut self, attr: &Attribute) {
        self.attributes
            .entry(attr.name.clone())
            .and_modify(|existing| *existing = attr.clone())
            .or_insert(attr.clone());
    }
}

impl From<InterimEntityChanges> for EntityChanges {
    fn from(value: InterimEntityChanges) -> Self {
        EntityChanges {
            component_id: value.component_id.clone(),
            attributes: value
                .attributes
                .into_values()
                .collect::<Vec<_>>(),
        }
    }
}

#[derive(Clone)]
struct SlotValue {
    new_value: Vec<u8>,
    start_value: Vec<u8>,
}

impl SlotValue {
    fn has_changed(&self) -> bool {
        self.start_value != self.new_value
    }
}

impl From<&StorageChange> for SlotValue {
    fn from(change: &StorageChange) -> Self {
        Self { new_value: change.new_value.clone(), start_value: change.old_value.clone() }
    }
}

// Uses a map for slots, protobuf does not allow bytes in hashmap keys
#[derive(Clone)]
pub struct InterimContractChange {
    address: Vec<u8>,
    balance: Vec<u8>,
    code: Vec<u8>,
    slots: HashMap<Vec<u8>, SlotValue>,
    change: ChangeType,
}

impl InterimContractChange {
    pub fn new(address: &[u8], creation: bool) -> Self {
        Self {
            address: address.to_vec(),
            balance: vec![],
            code: vec![],
            slots: Default::default(),
            change: if creation { ChangeType::Creation } else { ChangeType::Update },
        }
    }

    pub fn upsert_slot(&mut self, change: &StorageChange) {
        if change.address != self.address {
            panic!("Bad storage change");
        }
        self.slots
            .entry(change.key.clone())
            .and_modify(|sv| {
                sv.new_value
                    .copy_from_slice(&change.new_value)
            })
            .or_insert_with(|| change.into());
    }

    fn upsert_slots(&mut self, changes: &HashMap<Vec<u8>, SlotValue>) {
        for (slot, value) in changes.iter() {
            self.slots
                .entry(slot.clone())
                .and_modify(|sv| {
                    sv.new_value
                        .copy_from_slice(&value.new_value)
                })
                .or_insert(value.clone());
        }
    }

    pub fn set_balance(&mut self, new_balance: &[u8]) {
        self.balance.clear();
        self.balance
            .extend_from_slice(new_balance);
    }

    pub fn set_code(&mut self, code: &[u8]) {
        self.code.clear();
        self.code.extend_from_slice(code);
    }
}

impl From<InterimContractChange> for ContractChange {
    fn from(value: InterimContractChange) -> Self {
        ContractChange {
            address: value.address,
            balance: value.balance,
            code: value.code,
            slots: value
                .slots
                .into_iter()
                .filter(|(_, value)| value.has_changed())
                .map(|(slot, value)| ContractSlot { slot, value: value.new_value })
                .collect(),
            change: value.change.into(),
        }
    }
}
