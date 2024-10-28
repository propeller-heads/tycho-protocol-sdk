---
description: Understanding how the execution layer works.
---

# Overview

[**SwapExecutor**](swap-executor.md): This component is responsible for performing swaps by interacting with the underlying liquidity pools, handling token approvals, managing input/output amounts, and ensuring gas-efficient and secure execution. Each protocol must implement its own `SwapExecutor` (Solidity contract), tailored to its specific logic and requirements.

[**SwapEncoder**](swap-encoder.md): This component encodes the necessary data structures required for the `SwapExecutor` to perform swaps. It ensures that the swap details, including input/output tokens, pool addresses, and other protocol-specific parameters, are correctly formatted and encoded before being passed to the `SwapExecutor`. Each protocol must implement its own `SwapStructEncoder` Python class and ensure compatibility with its `SwapExecutor`.
