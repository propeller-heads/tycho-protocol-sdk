# Protocol Lib

Protocol lib enables solvers, searchers, and orderflow partners to tap into liquidity from decentralized protocols, currently focusing on swap protocols.

## Integration Process

To integrate, provide these three components:

1. **Protocol Logic:** Simulates protocol behavior, accurately modeling swaps and exchanges.
2. **Indexing:** Provides access to protocol state, tracking pools, pairs, and balances. Optional for stateless protocols.
3. **Execution:** Encodes and executes solutions on-chain.

To propose an integration, implement these components and create a pull request in this repository.

### Protocol Logic

Implement your protocol's logic using one of two modes:

1. **VM Integration:** Implement an adapter interface in any language that compiles to the respective VM bytecode. This SDK provides the interface in Solidity only. [Read more here.](logic/vm-integration/)

2. **Native Rust Integration:** (Coming soon) Implement a Rust trait describing the protocol logic.

{% hint style="info" %}
While VM integration is certainly the quickest and probably most accessible one for protocol developers, native implementations are much faster and allow us to consider the protocol for more time-sensitive use cases - e.g. quoting.
{% endhint %}

### Indexing

Provide a [substreams](https://substreams.streamingfast.io/) package that emits a specified set of messages. If your protocol has an existing [substreams package](https://github.com/messari/substreams), adjust it to emit the required messages.

- **VM Integration:** Currently supports EVM protocols to complement Solidity protocol logic. [Read more here.](https://github.com/propeller-heads/propeller-venue-lib/blob/main/docs/indexing/vm-integration/README.md)
- **Native Integration:** (Coming soon) Will complement the upcoming native Rust protocol logic.

### Execution

Implement the `SwapExecutor` and `SwapStructEncoder` interfaces to enable on-chain trade execution.

**SwapExecutor**

A Solidity contract that performs swaps by interacting with liquidity pools, handling token approvals, managing input/output amounts, and ensuring efficient, secure execution.

**SwapStructEncoder**

A Python class that encodes data structures for the `SwapExecutor`. It formats swap details, including input/output tokens, pool addresses, and protocol-specific parameters.

Each protocol must implement its own `SwapExecutor` and `SwapStructEncoder`, tailored to its specific logic and requirements.
