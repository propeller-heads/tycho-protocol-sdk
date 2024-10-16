# ethereum-etherfi substreams changes

It seems the substreams require some fixes and new logic to handle the implementation contracts and proxy contracts.

**Liquidity Pool Proxy**

- Address: 0x308861A430be4cce5502d0A12724771Fc6DaF216
- Deployment tx hash: 0x491b823bc15ced4c54f0ed5a235d39e478f8aae3ad02eb553924b40ad9859e10
- Deployment block: 17664317
- Standard: EIP-1967 Transparent Proxy

**Liquidity Pool Implementation**

- Address: 0xa8A8Be862BA6301E5949ABDE93b1D892C14FfB1F
- Deployment tx hash: 0x95c2e5081af5a591d4899689fa385b550a49775cc705cf1eb0b40bd14974568b
- Deployment block: 20832727

**weETH Proxy**

- Address: 0xCd5fE23C85820F7B72D0926FC9b05b43E359b7ee
- Deployment tx hash: 0xa034bdf7ec3b407125fcfbb786d908b0bcfd9976f2fbaf489776ba58b9db61ac
- Deployment block: 17664336
- Standard: EIP-1967 Transparent Proxy

**weETH Implementation**

- Address: 0xe629ee84C1Bd9Ea9c677d2D5391919fCf5E7d5D9
- Deployment tx hash: 0x455235ddb3d8c00f4cb805ae76f6c11d017bf71f32940459fce6269530b0c011
- Deployment block: 18517517

**eETH Proxy**

- Address: 0x35fA164735182de50811E8e2E824cFb9B6118ac2
- Deployment tx hash: 0xf6763c707b90b260bba114fce9a141aa4a923327539ded9d4d4ae4395b2200ff
- Deployment block: 17664324
- Standard: EIP-1967 Transparent Proxy

**eETH Implementation**

- Address: 0x1B47A665364bC15C28B05f449B53354d0CefF72f
- Deployment tx hash: 0x13b30c3b456189b04b2049d2c94285443f881497f3221a509a7d98facc06a5f7
- Deployment block: 18549702

## substreams.yaml changes explaination

1. **New Module `map_implementation_addresses`**:

   - Added a new map module (`map_implementation_addresses`) to identify and track implementation addresses for proxies.
   - This module processes each block and identifies if an address is a proxy, extracts its implementation, and outputs the implementation information.

2. **New Store `store_implementation_addresses`**:

   - A store named `store_implementation_addresses` is added to retain the implementation addresses for the proxies.
   - It stores the relationship between proxy addresses and their corresponding implementation addresses.

3. **Updating `map_relative_balances`**:

   - Updated `map_relative_balances` to also utilize `store_implementation_addresses` to determine if an address corresponds to an implementation, allowing proper handling of proxy balances.

4. **Updating `map_protocol_changes`**:
   - Included `store_implementation_addresses` to be an input for `map_protocol_changes`, ensuring that protocol changes can take the proxy and implementation relation into account.
