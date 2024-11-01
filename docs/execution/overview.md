---
description: Understanding how the execution layer works.
---

# Overview

[**SwapExecutor**](swap-executor.md): The `SwapExecutor` provides a unified interface for executing swaps. It handles interactions with the underlying liquidity pools, manages token approvals, controls input/output amounts, and ensures gas-efficient and secure execution. Each protocol must implement its own `SwapExecutor` (a Solidity contract) tailored to its specific logic and requirements.

[**SwapEncoder**](swap-encoder.md): The `SwapEncoder` encodes protocol components and swap parameters into valid calldata for the corresponding `SwapExecutor` contract. It ensures that swap details—including input/output tokens, pool addresses, and other protocol-specific parameters—are accurately formatted and encoded before being sent to the `SwapExecutor`. Each protocol must implement its own `SwapStructEncoder` Python class to maintain compatibility with its `SwapExecutor`.
