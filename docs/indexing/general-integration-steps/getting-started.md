---
description: Practical guide to write an indexing component.
---

# 2. Getting Started

Before integrating, ensure you have a thorough understanding of the protocol’s structure and behavior. Key areas to focus on include:

1. **Contracts and Their Roles**: Identify the contracts involved in the protocol and the specific roles they play. Understand how they impact the behavior of the component you're integrating.
2. **Conditions for State Changes**: Determine which conditions, such as oracle updates or particular method calls, trigger state changes (e.g. price updates) in the protocol.
3. **Component Addition and Removal**: Check how components are added or removed within the protocol. Many protocols either use a factory contract to deploy new components or provision new components directly through specific method calls.

Once you have a clear understanding of the protocol's mechanics, you can proceed with the implementation.

### Next Steps

1. **Proceed with** [**Substream Package Structure**](3.-substream-package-structure.md): This document will provide you with a clear understanding of the deliverables and how to structure your package. Then, begin implementing the recommended handlers or adjust the structure to fit your specific needs.
2. **Read About** [**Tracking Components**](../common-problems-and-patterns/tracking-components.md): This is essential for protocols that use factory contracts, as it outlines how to track new components effectively.
3. [**Normalize Relative ERC20 Balances**](../common-problems-and-patterns/normalizing-relative-erc20-balances.md): If you’re working with relative balance changes instead of absolute values, familiarize yourself with the suggested approach for normalizing balances.
4. [**Track Contract Storage**](../common-problems-and-patterns/tracking-contract-storage.md): For cases where you need to monitor the storage of specific contracts, consider using the techniques in the _Tracking Contract Storage_ guide.
5. [**Set Up and Run Tests**](4.-testing.md): Make use of the comprehensive testing suite for end-to-end testing of your Substreams modules.
