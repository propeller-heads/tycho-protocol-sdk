# Indexing Layer Overview

## Introduction

This guide explains the data model required to ingest protocol state into the PropellerHeads solver. The indexing sink that organizes, forwards, and stores all protocol-specific data is called Tycho Indexer.

Most integrations will likely use the VM approach, which usually requires less effort. This guide primarily focuses on providing state for VM integrations. Native integrations should follow the same pattern but emit changed attributes instead of changed contract storage slots.

## Understanding the Data Model

Tycho Indexer ingests all data versioned by block and transaction. This approach maintains a low-latency feed and correctly handles chains that can experience reverts.

Key points:
- Each state change must be communicated with its respective transaction that caused the change.
- For each transaction carrying state changes, the corresponding block must be provided.
- When processing a block, we need to emit:
  1. The block itself
  2. All transactions that introduced protocol state changes
  3. The state changes associated with their corresponding transaction

The data model encoding changes, transactions, and blocks in messages can be found [here](https://github.com/propeller-heads/propeller-protocol-lib/tree/main/proto/tycho/evm/v1).

## Models

These models are used for communication between Substreams and Tycho indexer, as well as between Substreams modules. Our indexer expects to receive a `BlockChanges` output from your Substreams package.

{% @github-files/github-code-block url="https://github.com/propeller-heads/propeller-protocol-lib/blob/main/proto/tycho/evm/v1/common.proto" %}

{% hint style="warning" %}
Changes must be aggregated at the transaction level. It's considered an error to emit `BlockChanges` with duplicate transactions in the `changes` attribute.
{% endhint %}

## Integer Byte Encoding

Many types are variable-length bytes, allowing flexibility across blockchains but requiring an agreed informal interface for interpretation:

- **Integers:** (especially for balances) should always be encoded as unsigned big-endian integers.
- **Strings:** Use UTF-8 encoding to store as bytes.
- **Attributes:** Value encoding is variable and depends on the use case. It can be tailored to the logic implementation (e.g., little-endian encoding for integers if the native logic module is in Rust).

## Special Attribute Names

Certain attribute names are reserved for specific purposes in our simulation process. Use them only for their intended functions. See the [list of reserved attributes](reserved-attributes.md).

## Changes of Interest

PropellerHeads integrations should communicate at least the following changes:

1. Protocol state changes:
   - For VM integrations, this usually means contract storage changes of all contracts whose state may be accessed during a swap operation.

2. Newly added protocol components:
   - Any new pool, pair, market, etc., that signifies a new operation can be executed using the protocol.

3. ERC20 Balances:
   - Whenever the balances of contracts involved with the protocol change, communicate this change in terms of absolute balances.

For implementation details, please refer to the "Getting Started" page.
