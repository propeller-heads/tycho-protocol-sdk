# VM Integration

This guide outlines the process for implementing the protocol's behavioral/logical component in a VM integration. You'll need to create a manifest file and implement the corresponding adapter interface.

## Existing Integrations

- Uniswap V2 (`/evm/src/uniswap-v2`)
- Balancer V2 (`/evm/src/balancer-v2`)

## Implementation Process

### Setup

1. Install Foundry:
   ```bash
   curl -L https://foundry.paradigm.xyz | bash
   foundryup
   ```

2. Clone the repository:
   ```bash
   git clone https://github.com/propeller-heads/propeller-protocol-lib
   ```

3. Install dependencies:
   ```bash
   cd ./propeller-protocol-lib/evm/
   forge install
   ```

### Understand the ISwapAdapter

1. Read the [Ethereum Solidity interface](ethereum-solidity.md) documentation.
2. Examine the docstrings in:
   - [ISwapAdapter.sol](https://github.com/propeller-heads/propeller-venue-lib/blob/main/evm/src/interfaces/ISwapAdapter.sol)
   - [ISwapAdapterTypes.sol](https://github.com/propeller-heads/propeller-venue-lib/blob/main/evm/src/interfaces/ISwapAdapterTypes.sol)
3. Generate local docs:
   ```bash
   cd ./evm/
   forge doc
   ```

### Implement the Interface

1. Create your integration directory:
   ```bash
   cp ./evm/src/template ./evm/src/<your-adapter-name>
   ```

2. Implement `ISwapAdapter` in `./evm/src/<your-adapter-name>.sol`.
   (Reference Uniswap V2 and Balancer V2 implementations)

### Test Your Implementation

1. Prepare the test file:
   ```bash
   cp ./evm/test/TemplateSwapAdapter.t.sol ./evm/test/<your-adapter-name>.t.sol
   ```

2. Write comprehensive tests for all implemented functions.

3. Configure fork testing:
   - Set `ETH_RPC_URL` to an Ethereum RPC URL (e.g., Alchemy, Infura)

4. Run tests:
   ```bash
   cd ./evm
   forge test
   ```

For detailed testing guidance, see the [Foundry test guide](https://book.getfoundry.sh/forge/tests) and [Fuzz testing](https://book.getfoundry.sh/forge/fuzz-testing) documentation.
