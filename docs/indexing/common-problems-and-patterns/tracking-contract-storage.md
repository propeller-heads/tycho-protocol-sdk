---
description: Indexing contract storage.
---

# Tracking Contract Storage

In VM implementations, accurately identifying and extracting relevant contract changes is essential. Each contract usually corresponds to a unique component, allowing its hex-encoded address to serve as the component ID, provided there is a one-to-one relationship between contracts and components. Note that this relationship may not always hold, so ensure this assumption applies to your specific protocol.

To streamline the extraction of relevant changes from the expanded block model, use the `tycho_substreams::contract::extract_contract_changes` helper function, which simplifies the process considerably.

The example below shows how to use a component store to define a predicate. This predicate filters for contract addresses of interest:

```rust
use tycho_substreams::contract::extract_contract_changes;

let mut transaction_contract_changes: HashMap<_, TransactionChanges> = HashMap::new();

extract_contract_changes(
    &block,
    |addr| {
        components_store
            .get_last(format!("pool:{0}", hex::encode(addr)))
            .is_some()
    },
    &mut transaction_contract_changes,
);
```
