---
description: Implementing a SwapEncoder for a Protocol
---

# Swap Encoder

## Overview

The `SwapStructEncoder` interface encodes data for swaps, which the `SwapExecutor` uses to interact with liquidity pools. It structures swap details, including input/output tokens, pool addresses, and protocol-specific parameters.

Each protocol must implement its own `SwapStructEncoder` to ensure correct data encoding for the `SwapExecutor`.

## Development Environment Setup

1. Access Propeller packages: Run `aws codeartifact login --tool pip --repository protosim --domain propeller` so you can access the propeller packages.

2. Create the dev environment `conda env create -f propeller-swap-encoders/environment_dev.yaml`

3. Activate it with `conda activate propeller-swap-encoders`

4. Install dependencies with `pip install -r propeller-swap-encoders/requirements.txt`

{% hint style="info" %}
Should you get a pyyaml installation error execute the following command: `pip install "cython<3.0.0" && pip install --no-build-isolation pyyaml==5.4.1`
{% endhint %}

You can import the abstract class `SwapStructEncoder` from `propeller-solver-core` in your python code like:
```python
from core.encoding.interface import SwapStructEncoder
```

## Key Methods

The `SwapStructEncoder` interface can be found [here](https://github.com/propeller-heads/defibot/blob/7ea38b92e60e182471f513c2aeef0370c4b3766a/propeller-solver-core/core/encoding/interface.py#L31).

### encode_swap_struct

**Purpose**: Encode swap details into a bytes object for `SwapExecutor` use.

**Parameters**:
- `swap`: Dictionary containing swap details:
  - `pool_id: str`: Liquidity pool identifier
  - `sell_token: EthereumToken`: Token to be sold
  - `buy_token: EthereumToken`: Token to be bought
  - `split: float`: How the swap should be split between pools
  - `sell_amount: int`: Amount of sell token to use
  - `buy_amount: int`: Amount of buy token to receive
  - `token_approval_needed: bool`: Indicates if token approval is needed
  - `pool_tokens`: Optional tuple with additional pool-specific token data
  - `pool_type: str`: Type of pool for swap execution
- `receiver`: Address to receive output tokens
- `encoding_context`: Additional context for encoding (details [here](https://github.com/propeller-heads/defibot/blob/7ea38b92e60e182471f513c2aeef0370c4b3766a/propeller-solver-core/core/encoding/interface.py#L9))
- `*kwargs`: Additional protocol-specific parameters

**Returns**: Bytes object containing encoded swap data

## Implementation Steps

1. **Define Protocol-Specific Encoding Logic**: Implement `encode_swap_struct` to encode swap details for your specific protocol.

2. **Ensure Compatibility with SwapExecutor**: Verify that the encoded data is compatible with the protocol's `SwapExecutor` implementation.

3. **Thorough Testing**: Test the encoding process with various swap scenarios to ensure accuracy and compatibility with the `SwapExecutor`.

## Example Implementation

See the example implementation of a `SwapExecutor` for Balancer here and test here.

{% hint style="warning" %}
Ensure your implementation adheres to the protocol's specific requirements and is thoroughly tested before deployment.
{% endhint}
