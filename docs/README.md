# Propeller Protocol Lib

Protocol lib is a library used by Propellerheads.xyz solvers to integrate decentralized protocols. Currently, only swap/exchange protocols are supported.

## Integration Process

To integrate with PropellerHeads solvers, 3 components need to be provided:

1. **Protocol logic:** Provides simulations of the protocols logic.
2. **Indexing**: Provides access to the protocol state used by the simulation.
3. **Execution**: Given a solution, this component will encode and execute it on-chain.

To propose an integration, create a pull request in this repository with the above components implemented.

### Protocol Logic

PropellerHeads currently exposes two integration modes to specify the protocols' underlying logic:

* **VM Integration:** This integration type requires implementing an adapter interface in any language that compiles to the respective vm byte code. This SDK provides the interface only in Solidity. [**Read more here.**](logic/vm-integration/)
* **Native Rust Integration:** Coming soon, this integration type requires implementing a Rust trait that describes the protocol logic.

{% hint style="info" %}
While VM integration is certainly the quickest and probably most accessible one for protocol developers, native implementations are much faster and allow us to consider the protocol for more time-sensitive use cases - e.g. quoting.
{% endhint %}

### Indexing

For indexing purposes, it is required that you provide a [substreams](https://substreams.streamingfast.io/) package that emits a specified set of messages. If your protocol already has a [substreams package](https://github.com/messari/substreams) for indexing implemented, you can adjust it to emit the required messages.

**VM Integration** Currently the only supported integration is for EVM protocols in order to complement the Solidity protocol logic. [**Read more here**](indexing/general-integration-steps/3.-substream-package-structure.md)**.**&#x20;

**Native Integration** Coming soon, this integration will complement the upcoming native Rust protocol logic.

### Execution

To create and execute a valid transaction that routes tokens through protocols, each protocol needs to   provide implementations for the `SwapExecutor` Solidity interface and the `SwapEncoder` Python interface. These interfaces work together to form and execute a swap on a protocol.[ Learn more here](execution/overview.md).
