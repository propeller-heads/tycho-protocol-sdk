# Indexing Integration Guide

## Understanding the Protocol

Before beginning integration, thoroughly understand the protocol:

1. Identify involved contracts and their functions. How do they affect the behavior of the component being integrated?
2. Determine conditions (e.g., oracle updates) or method calls that lead to relevant state changes, which ultimately change the protocol's behavior if observed externally.
3. Understand how components are added or removed (e.g., factory contracts, provisioning methods within the overall system).

## Setup

1. Install the [substreams CLI](https://substreams.streamingfast.io/documentation/consume/installing-the-cli).
2. Copy the `ethereum-template` to a new package named `[CHAIN]-[PROTOCOL_SYSTEM]`.
3. Adjust `cargo.toml` and `substreams.yaml` accordingly.
4. Generate protobuf code:
   ```bash
   substreams protogen substreams.yaml --exclude-paths="sf/substreams,google"
   ```
5. Register the new package in `substreams/Cargo.toml` as a workspace member.

The template already contains the `tycho-substreams` package as a dependency, providing necessary output types and helper functions.

## Integration Structure

Typical integration modules:

1. `map_components(block)`: Extracts newly created components from the block model.
2. `store_components(components, components_store)`: Stores necessary component information for downstream modules.
3. `map_relative_balances(block, components_store)`: Extracts relative balance changes for protocols without absolute balance emissions.
4. `store_balances(balance_deltas, balance_store)`: Converts relative balance deltas into absolute balances using an additive store.
5. `map_protocol_changes(balance_deltas, balance_store, components_store, ...)`: Combines all information to build the final `BlockChanges` output model.

{% hint style="info" %}
Examples are from the ethereum-balancer substream. Full code available [here](https://github.com/propeller-heads/propeller-protocol-lib/tree/main/substreams/ethereum-balancer).
{% endhint %}

## Tracking Components

1. Implement a `factory.rs` module to detect newly deployed components.
2. Use `BlockTransactionProtocolComponents` as the output model:
   ```protobuf
   message TransactionProtocolComponents {
     Transaction tx = 1;
     repeated ProtocolComponent components = 2;
   }

   message BlockTransactionProtocolComponents {
     repeated TransactionProtocolComponents tx_components = 1;
   }
   ```
3. Store component addresses for downstream use.

{% hint style="danger" %}
Never emit state changes for unannounced components.
{% endhint %}

## Tracking Absolute Balances

### 1. Index relative balance changes

Implement a handler using `BlockBalanceDeltas`:

```rust
#[substreams::handlers::map]
pub fn map_relative_balances(
    block: eth::v2::Block,
    components_store: StoreGetInt64,
) -> Result<BlockBalanceDeltas, anyhow::Error> {
    todo!()
}
```

Use `tycho_substream::balances::extract_balance_deltas_from_tx` for ERC20 `Transfer` events. Ensure each `BalanceDelta` within a component token pair has a strictly increasing ordinal to maintain transaction-level integrity.

### 2. Aggregate balances with an additive store

Use `StoreAddBigInt` and `tycho_substream::balances::store_balance_changes`:

```rust
#[substreams::handlers::store]
pub fn store_balances(deltas: BlockBalanceDeltas, store: StoreAddBigInt) {
    tycho_substreams::balances::store_balance_changes(deltas, store);
}
```

This efficiently aggregates `BlockBalanceDeltas` messages into absolute values while maintaining transaction-level granularity.

### 3. Combine absolute values with component and address

Use `tycho_substream::balances::aggregate_balances_changes`:

```rust
#[substreams::handlers::map]
pub fn map_protocol_changes(
    block: eth::v2::Block,
    grouped_components: BlockTransactionProtocolComponents,
    deltas: BlockBalanceDeltas,
    components_store: StoreGetInt64,
    balance_store: StoreDeltas,
) -> Result<BlockChanges> {
    let mut transaction_contract_changes: HashMap<_, TransactionChanges> = HashMap::new();

    aggregate_balances_changes(balance_store, deltas)
        .into_iter()
        .for_each(|(_, (tx, balances))| {
            transaction_contract_changes
                .entry(tx.index)
                .or_insert_with(|| TransactionChanges::new(&tx))
                .balance_changes
                .extend(balances.into_values());
        });
}
```

This function outputs aggregated `BalanceChange` structs per transaction, which can be integrated into `map_protocol_changes` for retrieving absolute balance changes associated with each transaction.

## Tracking State Changes

For VM implementations:

1. Use the hex-encoded address as the component's ID (typical one-to-one mapping between contracts and components).
2. Use `tycho_substreams::contract::extract_contract_changes` to extract relevant changes from the expanded block model:

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

This example uses a component store to define a predicate that identifies contract addresses of interest.

This guide provides a detailed approach to indexing integration, focusing on key steps, best practices, and important technical details for accurate implementation.
