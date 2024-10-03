# Reserved Attribute Names Guide

Certain attribute names are reserved for specific purposes in our system. Use them only for their intended applications.

## Static Attributes

These attributes must be set using `ProtocolComponent.static_att` and MUST be immutable. If an attribute can change, use a state attribute instead.

### manual_updates

**Description:** Controls whether component updates should be manually triggered using the `update_marker` state attribute.

**Type:** [1u8] to enable manual updates

**Example:**
```rust
Attribute {
    name: "manual_updates".to_string(),
    value: [1u8],
    change: ChangeType::Creation.into(),
}
```

### pool_id

**Description:** Specifies the pool identifier when it differs from `ProtocolComponent.id`.

**Type:** UTF-8 encoded string in bytes

**Example:**
```rust
Attribute {
    name: "pool_id".to_string(),
    value: format!("0x{}", hex::encode(pool_registered.pool_id)).as_bytes(),
    change: ChangeType::Creation.into(),
}
```

{% hint style="info" %}
In most cases, using `ProtocolComponent.id` directly is preferred over `pool_id`.
{% endhint %}

## State Attributes

These attributes must be set using `EntityChanges`. Unlike static attributes, state attributes can change at any time.

### stateless_contract_addr_{index}

**Description:** Specifies the address of a stateless contract required by the component.

**Type:** UTF-8 encoded string in bytes

**Examples:**

1. Direct Contract Address:
   ```rust
   Attribute {
       name: "stateless_contract_addr_0".into(),
       value: format!("0x{}", hex::encode(address)).into_bytes(),
       change: ChangeType::Creation.into(),
   }
   ```

2. Dynamic Address Resolution:
   ```rust
   Attribute {
       name: "stateless_contract_addr_0".into(),
       value: format!("call:0x{}:views_implementation()", hex::encode(TRICRYPTO_FACTORY)).into_bytes(),
       change: ChangeType::Creation.into(),
   }
   ```

### stateless_contract_code_{index}

**Description:** Specifies the code for a given `stateless_contract_addr`.

**Type:** Bytes

**Example:**
```rust
Attribute {
    name: "stateless_contract_code_0".to_string(),
    value: code.to_vec(),
    change: ChangeType::Creation.into(),
}
```

### balance_owner

**Description:** Specifies the address of the account that owns the protocol component tokens.

**Type:** Bytes

**Example:**
```rust
Attribute {
    name: "balance_owner".to_string(),
    value: VAULT_ADDRESS.to_vec(),
    change: ChangeType::Creation.into(),
}
```

### update_marker

**Description:** Indicates that a pool has changed, triggering an update on the protocol component when `manual_update` is enabled.

**Type:** Bytes

**Example:**
```rust
Attribute {
    name: "update_marker".to_string(),
    value: vec![1u8],
    change: ChangeType::Update.into(),
}
```

{% hint style="warning" %}
Ensure that you use these reserved attributes correctly to maintain consistency and proper functionality in your integration.
{% endhint %}
