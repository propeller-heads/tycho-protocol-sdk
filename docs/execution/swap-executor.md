---
description: Implementing a SwapExecutor for a Protocol
---

# Swap Executor

## Overview

The `ISwapExecutor` interface performs swaps on a liquidity pool. It accepts either the input or output token amount and returns the swapped amount. This interface is crucial for creating a protocol-specific `SwapExecutor`.

The `SwapExecutor` works with the `SwapStructEncoder`, which encodes data needed for the swap. This encoded data is then passed to the `SwapExecutor` to perform the swap according to the protocol's logic.

## Key Method

* **swap(uint256 givenAmount, bytes calldata data)**
  * **Purpose**: Perform a token swap.
  * **Parameters**:
    * `givenAmount`: Token amount (input or output) for the swap.
    * `data`: Encoded swap information from `SwapStructEncoder`.
  * **Returns**: Swapped token amount.

## Implementation Steps

1. **Define Protocol Logic**: Implement `swap` function to interact with the protocol's liquidity pool. Use `data` to encode pool and token addresses.
2. **Handle Input/Output**: Determine if `givenAmount` is input or output. Calculate the swapped amount using the pool's pricing logic.
3. **Manage Errors**: Use `ISwapExecutorErrors` for invalid parameters or unknown pool types.
4. **Handle Token Approvals**: Manage any required token approvals before swaps.
5. **Support Token Transfers**: Ensure implementation can transfer received tokens to a designated address.
6. **Optimize Gas Usage**: Make the implementation gas-efficient. Assembly isn't necessary.
7. **Ensure Security**: Validate inputs, control access, and prevent reentrancy attacks.

## Example

See the Balancer `SwapExecutor` implementation [here](../../evm/src/balancer-v2/BalancerSwapExecutor.sol).
