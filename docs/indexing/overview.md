---
description: Understanding how the indexing layer works.
---

# Overview

This page provides an overview of the data model necessary to ingest protocol state into Tycho Indexer.

### Deliverable

Our indexing integrations require a Substreams SPKG to transform raw blockchain data into high-level data streams.&#x20;

Substreams is a new indexing technology that uses Rust modules to process raw blockchain data into more structured, protocol-specific data streams. These modules, along with protobuf definitions and a manifest, are packaged into an SPKG file (more info [here](https://substreams.streamingfast.io/quick-access/glossary#spkg-.spkg)) which can then be run on the Substreams server.

For further information, refer to the [Substreams](https://thegraph.com/docs/en/substreams/) [quick explanation](https://thegraph.com/docs/en/substreams/) or explore the full [documentation](https://docs.substreams.dev/), which outlines the required functions and manifest file structure.

#### **Integration Modes: VM and Native**

#### VM

VM integrations primarily track contract storage associated with the protocol’s behavior. A key limitation in Substreams to keep in mind is that you must witness a contract’s creation to access its full storage. Most integrations will likely use the VM method due to its relative simplicity, so this guide focuses on VM-based integrations.

#### Native

Native integrations follow a similar approach with one main difference: instead of emitting changes in contract storage slots, they should emit values for all created and updated attributes relevant to the protocol’s behavior.

### Understanding the Data Model

The Tycho Indexer ingests all data versioned by block and transaction. This approach helps maintain a low-latency feed and correctly handles chains that may undergo reorgs.

Each state change communicated must include the transaction that caused the change. Additionally, each transaction carrying state changes must be paired with its corresponding block.

In summary, when processing a block, we need to emit the block itself, all transactions that introduce protocol state changes, and, finally, the state changes associated with their corresponding transactions.

**Details of the data model that encodes these changes, transactions, and blocks in messages are available** [**here**](https://github.com/propeller-heads/propeller-protocol-lib/tree/main/proto/tycho/evm/v1)**.**

#### Models

The models below facilitate communication between Substreams and the Tycho Indexer, as well as within Substreams modules. Tycho Indexer expects to receive a `BlockChanges` output from your Substreams package.

{% @github-files/github-code-block url="https://github.com/propeller-heads/propeller-protocol-lib/blob/main/proto/tycho/evm/v1/common.proto" %}

Changes must be aggregated at the transaction level; it is considered an error to emit `BlockChanges` with duplicate transactions in the `changes` attributes.

#### Integer Byte encoding

To ensure compatibility across blockchains, many of the data types listed above are encoded as variable-length bytes. This flexible approach requires an informal interface so that consuming applications can interpret these bytes consistently.

**Integers:** When encoding integers, particularly those representing balances, always use unsigned big-endian format. Balances are referenced at multiple points within the system and need to be consistently decoded along their entire journey.

**Strings**: Use UTF-8 encoding for any string data stored as bytes.

**Attributes:** Attribute encoding is variable and depends on the specific use case. However, whenever possible, follow the encoding standards mentioned above for integers and string

#### Special attribute names

Certain attribute names are reserved for specific functions in our simulation process. Use these names only for their intended purposes. Refer to the [list of reserved attributes](reserved-attributes.md).

### Changes of interest

Tycho Protocol Integrations should communicate the following changes:

1. **New Protocol Components**: Notify any newly added protocol components, such as pools, pairs, or markets—essentially, anything that indicates a new operation can now be executed using the protocol.
2. **ERC20 Balances**: Whenever the balances of any contracts involved with the protocol change, report these changes in terms of absolute balances.
3. **Protocol State Changes**: For VM integrations, this typically involves reporting contract storage changes for all contracts whose state may be accessed during a swap operation (except token contracts).

For a hands-on integration guide, refer to the[ Getting Started](general-integration-steps/getting-started.md) page.
