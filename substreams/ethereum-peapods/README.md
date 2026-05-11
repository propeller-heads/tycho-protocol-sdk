# Ethereum Peapods Substreams Package

This substreams package indexes token pair creation and price changes within peapod adapters on EVM networks.

## Modules Overview

The package consists of five core modules that work together to track and index protocol changes:

1. `map_function_calls` - Maps adapter contract function calls (Swap, SwapToPrice, Price) into structured FunctionCalls
2. `store_exchange_price` - Stores exchange prices from function calls in the store
3. `map_token_pairs` - Creates protocol components for token pairs based on function calls
4. `store_token_pairs` - Persists token pair protocol components in the store
5. `map_protocol_changes` - Aggregates changes from function calls and components into BlockChanges

## Usage

First, set your Substreams API token:

```bash
export SUBSTREAMS_API_TOKEN="your-jwt-token"
```

Run the substreams GUI:

```bash
substreams gui substreams.yaml map_protocol_changes \
  -e base-mainnet.streamingfast.io:443 \
  --start-block $start_block_num \ 
  --stop-block $offset \ # Like +50
  --network base
```

## Module Details

### map_function_calls
Identifies and decodes three types of function calls to the adapter contract:
- `Swap2` - Direct token swaps 
- `SwapToPrice` - Swaps targeting a specific price
- `Price` - Price queries for token pairs

### store_exchange_price
Stores exchange prices from successful swaps and price queries, computing the actual exchange price based on the trade results.

### map_token_pairs
Creates protocol components for each unique token pair encountered in function calls. Components include:
- Unique ID based on token addresses
- Token addresses
- Adapter contract address
- Protocol type metadata

### store_token_pairs
Persists protocol components in the store for later reference.

### map_protocol_changes
Aggregates protocol changes by:
- Tracking newly created components per transaction
- Recording price updates from exchange deltas
- Organizing changes by transaction

The final output is a BlockChanges structure containing all protocol activity within each block.