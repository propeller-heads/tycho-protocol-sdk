# balancer_v3 Substreams modules

This package was initialized via `substreams init`, using the `evm-events-calls` template.

## Utility Tools

This directory contains several utility tools to help with development and configuration:

### `buffer.py`
Tool script to fetch all initialized liquidity buffer tokens from Balancer V3 Vault. This is not main application logic - it's a utility to discover buffer token mappings and organize them as a mapping structure for configuration purposes.

Usage: Set `ETH_RPC_URL` environment variable and run the script to scan for buffer tokens.

### `codec.py`
Tool to generate encoding strings for MappingToken structures used in Balancer V3. This utility helps encode/decode token mappings for wrapped, underlying, and none-type tokens.

Usage examples:
```bash
# Encode underlying tokens
python codec.py encode --underlying c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2 7f39c581f595b53c5cb19bd0b3f8da6c935e2ca0

# Encode with zero address for none type
python codec.py encode --none

# Encode with specific address for none type
python codec.py encode --none 1234567890abcdef1234567890abcdef12345678

# Mixed types encoding
python codec.py encode --wrapped abc123 --underlying def456 --none

# Decode hex string
python codec.py decode <hex_string>
```

## Usage

```bash
substreams build
substreams auth
substreams gui       			  # Get streaming!
substreams registry login         # Login to substreams.dev
substreams registry publish       # Publish your Substreams to substreams.dev
```

## Modules

All of these modules produce data filtered by these contracts:
- _vault_ at **0xba1333333333a1ba1108e8412f11850a5c319ba9**
- _stable_pool_factory_ at **0xb9d01ca61b9c181da1051bfdd28e1097e920ab14**
- _weighted_pool_factory_ at **0x201efd508c8dfe9de1a13c2452863a78cb2a86cc**
- stable_pool contracts created from _stable_pool_factory_
- weighted_pool contracts created from _weighted_pool_factory_

### `map_components`
Maps blockchain events and calls to protocol components, extracting pool creation events and related data from Balancer V3 contracts.

### `store_components`
Stores protocol components using a set_if_not_exists policy to maintain component state across blocks.

### `store_token_set`
Stores token set information to track which tokens are associated with which pools.

### `map_relative_balances`
Maps balance changes relative to the stored components, tracking liquidity changes in pools.

### `store_balances`
Accumulates balance changes using an additive update policy to maintain running totals.

### `map_protocol_changes`
The main output module that combines component data, balance changes, and protocol state to produce comprehensive block change information. This module outputs `BlockChanges` that can be consumed by downstream systems.


