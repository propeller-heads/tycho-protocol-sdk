# ethereum-etherfi substream tests

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

## Integration_test.tycho.yaml using constructor()

Since in `ethereum-sfraxeth` we had issues when casting addresses into interface, I'll try to rerun the integration test by first modifying the adapter so that the weeth address is hardcoded and does not need to be passaed

**EtherfiAdapter.sol** changes

```
constructor() {
    weEth = IWeEth(address(0xCd5fE23C85820F7B72D0926FC9b05b43E359b7ee));
    eEth = weEth.eETH();
    liquidityPool = eEth.liquidityPool();
}
```

**Integration_test.tycho.yaml**

```
# Name of the substreams config file in your substreams module. Usually "./substreams.yaml"
substreams_yaml_path: ./substreams.yaml

# Name of the adapter contract, usually: ProtocolSwapAdapter
adapter_contract: "EtherfiAdapter"

# Constructor signature of the Adapter contract
adapter_build_signature: "constructor()"

# A comma separated list of args to be passed to the constructor of the Adapter contract
adapter_build_args:

# Whether or not the testing script should skip checking balances of the protocol components.
skip_balance_check: false

# A list of accounts that need to be indexed to run the tests properly.
# Usually used when there is a global component required by all pools and created before the tested range of blocks.
initialized_accounts:
  - "0x308861A430be4cce5502d0A12724771Fc6DaF216" # Liquidity pool component for initialization

# A list of protocol types names created by your Substreams module.
protocol_type_names:
  - "etherfi_liquidity_pool"
  - "etherfi_weeth_pool"

# A list of tests.
tests:
  # Test for Etherfi Liquidity Pool creation
  - name: test_liquidity_pool_creation
    # Indexed block range
    start_block: 17664317
    stop_block: 17664500
    expected_components:
      - id: "0x308861A430be4cce5502d0A12724771Fc6DaF216" # Liquidity Pool Address
        tokens:
          - "0x35fA164735182de50811E8e2E824cFb9B6118ac2" # eETH
          - "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE" # ETH
        creation_tx: "0x491b823bc15ced4c54f0ed5a235d39e478f8aae3ad02eb553924b40ad9859e10"
        skip_simulation: false

  # Test for WeETH pool creation
  - name: test_weeth_pool_creation
    start_block: 17664317
    stop_block: 17664500
    expected_components:
      - id: "0xCd5fE23C85820F7B72D0926FC9b05b43E359b7ee" # WeETH Address
        tokens:
          - "0x35fA164735182de50811E8e2E824cFb9B6118ac2" # eETH
          - "0xCd5fE23C85820F7B72D0926FC9b05b43E359b7ee" # WeETH
        creation_tx: "0xa034bdf7ec3b407125fcfbb786d908b0bcfd9976f2fbaf489776ba58b9db61ac"
        skip_simulation: false

```

### Test results

`python ./testing/src/runner/cli.py --package "ethereum-etherfi" --vm-traces`

❗️ test_liquidity_pool_creation failed: '0x308861a430be4cce5502d0a12724771fc6daf216' not found in protocol components.

❗️ test_weeth_pool_creation failed: '0xcd5fe23c85820f7b72d0926fc9b05b43e359b7ee' not found in protocol components.

## Integration_test.tycho.yaml using implementation addresses instead of proxyes

```
# Name of the substreams config file in your substreams module. Usually "./substreams.yaml"
substreams_yaml_path: ./substreams.yaml

# Name of the adapter contract, usually: ProtocolSwapAdapter
adapter_contract: "EtherfiAdapter"

# Constructor signature of the Adapter contract
adapter_build_signature: "constructor()"

# A comma separated list of args to be passed to the constructor of the Adapter contract
adapter_build_args:

# Whether or not the testing script should skip checking balances of the protocol components.
skip_balance_check: false

# A list of accounts that need to be indexed to run the tests properly.
# Usually used when there is a global component required by all pools and created before the tested range of blocks.
initialized_accounts:
  - "0x308861A430be4cce5502d0A12724771Fc6DaF216" # Liquidity pool component for initialization

# A list of protocol types names created by your Substreams module.
protocol_type_names:
  - "etherfi_liquidity_pool"
  - "etherfi_weeth_pool"

# A list of tests.
tests:
  # Test for Etherfi Liquidity Pool creation
  - name: test_liquidity_pool_creation
    # Indexed block range
    start_block: 20832700 # Actual deployment block: 20832727
    stop_block: 20832800
    expected_components:
      - id: "0xa8A8Be862BA6301E5949ABDE93b1D892C14FfB1F" # Liquidity Pool Implementation Address
        tokens:
          - "0x35fA164735182de50811E8e2E824cFb9B6118ac2" # eETH
          - "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE" # ETH
        creation_tx: "0x95c2e5081af5a591d4899689fa385b550a49775cc705cf1eb0b40bd14974568b"
        skip_simulation: false

  # Test for WeETH pool creation
  - name: test_weeth_pool_creation
    start_block: 18517500 # Actual deployment block: 18517517
    stop_block: 18517600
    expected_components:
      - id: "0xe629ee84C1Bd9Ea9c677d2D5391919fCf5E7d5D9" # WeETH Implementation Address
        tokens:
          - "0x35fA164735182de50811E8e2E824cFb9B6118ac2" # eETH
          - "0xCd5fE23C85820F7B72D0926FC9b05b43E359b7ee" # WeETH
        creation_tx: "0x455235ddb3d8c00f4cb805ae76f6c11d017bf71f32940459fce6269530b0c011"
        skip_simulation: false

```
### Test results

`python ./testing/src/runner/cli.py --package "ethereum-etherfi" --vm-traces`

❗️ test_liquidity_pool_creation failed: '0x308861a430be4cce5502d0a12724771fc6daf216' not found in protocol components.

❗️ test_weeth_pool_creation failed: '0xcd5fe23c85820f7b72d0926fc9b05b43e359b7ee' not found in protocol components.