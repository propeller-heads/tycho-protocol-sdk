---
description: Provide protocol logic using the ethereum virtual machine
---

# Ethereum: Solidity

## Swap/Exchange Protocol Guide

### Implementing the Protocol

To integrate an EVM exchange protocol:

1. Implement the [`ISwapAdapter.sol`](https://github.com/propeller-heads/propeller-protocol-lib/blob/main/evm/interfaces/ISwapAdapter.sol) interface.
2. Create a manifest file summarizing the protocol's metadata.

While we specify the interface for Solidity, you can use any compiled EVM bytecode. If you prefer Vyper, feel free to implement the interface using it. You can submit compiled Vyper bytecode, although we don't yet provide all the tooling for Vyper contracts.

### The Manifest File

The manifest file contains author information and additional static details about the protocol and its testing. Here's a list of all valid keys:

```yaml
yamlCopy# Author information helps us reach out in case of issues
author:
  name: Propellerheads.xyz
  email: alan@propellerheads.xyz

# Protocol Constants
constants:
  # Minimum gas usage for a swap, excluding token transfers
  protocol_gas: 30000
  # Minimum expected capabilities (individual pools may extend these)
  # To learn about Capabilities, see ISwapAdapter.sol
  capabilities:
    - SellSide
    - BuySide
    - PriceFunction

# Adapter contract (byte)code files
contract: 
  # Contract runtime (deployed) bytecode (required if no source is provided)
  runtime: UniswapV2SwapAdapter.bin
  # Source code (our CI can generate bytecode if you submit this)
  source: UniswapV2SwapAdapter.sol

# Deployment instances for chain-specific bytecode
# Used by the runtime bytecode build script
instances:
  - chain:
      name: mainnet
      id: 1
    # Constructor arguments for building the contract
    arguments:
      - "0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f"

# Automatic test cases (useful if getPoolIds and getTokens aren't implemented)
tests:
  instances:
    - pool_id: "0xB4e16d0168e52d35CaCD2c6185b44281Ec28C9Dc"
      sell_token: "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"
      buy_token: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"
      block: 17000000
      chain:
        name: mainnet
        id: 1
```

### Key Functions

#### Price (optional)

Calculates pool prices for specified amounts.

```solidity
solidityCopyfunction price(
    bytes32 poolId,
    IERC20 sellToken,
    IERC20 buyToken,
    uint256[] memory sellAmounts
) external view returns (Fraction[] memory prices);
```

* Return prices in buyToken/sellToken units.
* Include all protocol fees (use minimum fee for dynamic fees).
* Implement this method as `view` for efficiency and parallel execution.
* If you don't implement this function, flag it accordingly in capabilities and make it revert using the `NotImplemented` error.
* While optional, we highly recommend implementing this function. If unavailable, we'll numerically estimate the price function from the swap function.

#### Swap

Simulates token swapping on a given pool.

```solidity
solidityCopyfunction swap(
    bytes32 poolId,
    IERC20 sellToken,
    IERC20 buyToken,
    OrderSide side,
    uint256 specifiedAmount
) external returns (Trade memory trade);
```

* Execute the swap and change the VM state accordingly.
* Include a gas usage estimate for each amount (use `gasleft()` function).
* Return a `Trade` struct with a `price` attribute containing `price(specifiedAmount)`.
* If the price function isn't supported, return `Fraction(0, 1)` for the price (we'll estimate it numerically).

#### GetLimits

Retrieves token trading limits.

```solidity
solidityCopyfunction getLimits(bytes32 poolId, OrderSide side)
    external
    returns (uint256[] memory);
```

* Return the maximum tradeable amount for each token.
* The limit is reached when the change in received amounts is zero or close to zero.
* Overestimate the limit if in doubt.
* Ensure the swap function doesn't error with `LimitExceeded` for amounts below the limit.

#### getCapabilities

Retrieves pool capabilities.

```solidity
solidityCopyfunction getCapabilities(bytes32 poolId, IERC20 sellToken, IERC20 buyToken)
    external
    returns (Capability[] memory);
```

#### getTokens (optional)

Retrieves tokens for a given pool.

```solidity
solidityCopyfunction getTokens(bytes32 poolId)
    external
    returns (IERC20[] memory tokens);
```

* We mainly use this for testing, as it's redundant with the required substreams implementation.

#### getPoolIds (optional)

Retrieves a range of pool IDs.

```solidity
solidityCopyfunction getPoolIds(uint256 offset, uint256 limit)
    external
    returns (bytes32[] memory ids);
```

* We mainly use this for testing. It's okay not to return all available pools here.
* This function helps us test against the substreams implementation.
* If you implement it, it saves us time writing custom tests.
