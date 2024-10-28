---
description: Keeping track of a protocols components.
---

# Tracking Components

A common pattern if protocols use factories to deploy components, is to detect the creation of new these and store their contract addresses to track them downstream. Later, you might need to emit balance and state changes based on the current set of tracked components.&#x20;

{% hint style="danger" %}
Emitting state or balance changes for components not previously registered is considered an error.
{% endhint %}

Start by implementing a map module that identifies and emits any newly created components. These are detected by inspecting the `sf.ethereum.type.v2.Block` model. The output message should include all available information about the component at the time of creation, along with the transaction that deployed it.

A recommended approach is to create a `factory.rs` module to facilitate the detection of newly deployed components. Use this module in a map handler to detect and emit new protocol components. The recommended output model for this initial handler is `BlockTransactionProtocolComponents`:

```protobuf
// A message containing protocol components that were created by a single tx.
message TransactionProtocolComponents {
  Transaction tx = 1;
  repeated ProtocolComponent components = 2;
}

// All protocol components that were created within a block with their corresponding tx.
message BlockTransactionProtocolComponents {
  repeated TransactionProtocolComponents tx_components = 1;
}
```

Note that a single transaction may create multiple components. In such cases, `TransactionProtocolComponents.components` should list all newly created `ProtocolComponents`.

After emitting, store the protocol components in a `Store` to determine later whether a contract is relevant for tracking.
