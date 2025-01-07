# VM Integration

Our indexing integrations use the Substreams library to transform raw blockchain data into higher level data streams. This is done by implementing a Rust module that is compiled into a SPKG file and then loaded by the Substreams server.

## Example

We have integrated the **Balancer** protocol as a reference, see `/substreams/ethereum-balancer` for more information.

## Step by step

1. Install [Rust](https://www.rust-lang.org/tools/install), you can do so with the following command:

    ```bash
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    ```

1. Install [Substreams CLI](https://substreams.streamingfast.io/getting-started/installing-the-cli), you can either use brew:

    ```bash
    brew install streamingfast/tap/substreams
    ```
    use precompiled binaries
    ```bash
    # Use correct binary for your platform
    LINK=$(curl -s https://api.github.com/repos/streamingfast/substreams/releases/latest | awk '/download.url.*linux/ {print $2}' | sed 's/"//g')
    curl -L  $LINK  | tar zxf -
    ```
    or compile from source:
    ```bash
    git clone https://github.com/streamingfast/substreams
    cd substreams
    go install -v ./cmd/substreams
    ```

1. Start by making a local copy of the Propeller Protocol Lib repository:
    ```bash
    git clone https://github.com/propeller-heads/tycho-protocol-sdk
    ```

## Understanding the Substreams VM integration

Substreams is a new indexing technology, which uses Rust modules to compose raw blockchain data streams into higher level data streams, in our case specific to the protocol. These modules together with the protobuf definitions and manifest are then wrapped into SPKG packages (more info [here](https://substreams.streamingfast.io/quick-access/glossary#spkg-.spkg)) that are then run remotely on the Substreams server.

For more information, read the [quick explanation of Substreams](https://thegraph.com/docs/en/substreams/) or jump into the [Substreams documentation](https://substreams.streamingfast.io/). It describes the functions that need to be implemented as well as the manifest file.

### ProtoBuf files

Generally these describe the raw blockchain data that we get on the input stream and the output data that we want to produce using the Rust module.

If you are unfamiliar with ProtoBuf at all, you can start with the [official documentation](https://protobuf.dev/overview/).

First get familiar with the raw ProtoBuf definitions provided by us:
- [common.proto](../../../proto/tycho/evm/v1/common.proto) - Common types used by all integration types

You can also create your own intermediate ProtoBufs. These files should reside in your own substreams package, e.g. `./substreams/ethereum-template/proto/custom-messages.proto`. You have to link these files in the `substreams.yaml` file, see the [manifest docs](https://substreams.streamingfast.io/developers-guide/creating-your-manifest) for more information or you can look at the official substreams example integration of [UniswapV2](https://github.com/messari/substreams/blob/master/uniswap-v2/substreams.yaml#L20-L22).

*Note: Internally we are referring to our indexing library as `Tycho`, which is why our protobuf files are under the `proto/tycho` directory.*

### Rust module

The goal of the rust module is to implement the logic that will transform the raw blockchain data into the desired output data. 

*This is the actual integration code that you will be writing!*

The module is a Rust library that is compiled into a SPKG (`.spkg`) file using the Substreams CLI and then loaded by the Substreams server. It is defined by the `lib.rs` file (see the [Balancer reference example](../../../substreams/ethereum-balancer/src/lib.rs)).

Read our [Substreams README.md](../../../substreams/README.md) for more information on how to write the Rust module.

### How to implement the integration

1. Create a new directory for your integration by cloning the template, rename all the references to `ethereum-template` to `[CHAIN]-[PROTOCOL_SYSTEM]`:

    ```bash
    cp -r ./substreams/ethereum-template ./substreams/[CHAIN]-[PROTOCOL_SYSTEM]
    ```
1. Implement the logic in the Rust module `lib.rs`. The main function to implement is the `map_protocol_changes` function, which is called for every block. 
    
    ```rust
    #[substreams::handlers::map]
    fn map_protocol_changes(
        block: eth::v2::Block,
    ) -> Result<tycho::BlockChanges, substreams::errors::Error> {}
    ```
    The `map_protocol_changes` function takes a raw block as input and returns a `BlockChanges` struct, which is derived from the `BlockChanges` protobuf message in [common.proto](../../../proto/tycho/evm/v1/common.proto). 


1. The `BlockChanges` is a list of `TransactionChanges`, which includes these main fields:
    - list of `ContractChange` - All storage slots that have changed in the transaction for every contract tracked by any ProtocolComponent
    - list of `EntityChanges` - All the attribute changes in the transaction
    - list of `ProtocolComponent` - All the protocol component changes in the transaction
    - list of `BalanceChange` - All the token balances changes in the transaction

    See the [Balancer reference example](../../../substreams/ethereum-balancer/src/lib.rs) for more information.

1. If you are more advanced with Substreams, you can define more steps than a single "map" step, including defining your own protobuf files. Add these protobuf files in your `pb` folder and update the manifest accordingly. This allows for better parallelization of the indexing process. See the official documentation of [modules](https://substreams.streamingfast.io/concepts-and-fundamentals/modules#modules-basics-overview).

### Testing

Read the [Substreams testing docs](https://github.com/propeller-heads/propeller-venue-lib/blob/main/testing/README.md) for more information on how to test your integration.
