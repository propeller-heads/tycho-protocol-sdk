specVersion: v0.1.0
package:
  name: "ethereum_sky"
  version: v0.1.0

protobuf:
  files:
    - tycho/evm/v1/vm.proto
    - tycho/evm/v1/common.proto
    - tycho/evm/v1/utils.proto
    - contract.proto
  importPaths:
    - ../../proto
    - proto

binaries:
  default:
    type: wasm/rust-v1
    file: ../target/wasm32-unknown-unknown/release/ethereum_sky.wasm

modules:
  - name: map_components
    kind: map
    initialBlock: 20663734
    inputs:
      - params: string
      - source: sf.ethereum.type.v2.Block
    output:
      type: proto:tycho.evm.v1.BlockTransactionProtocolComponents

  - name: store_components
    kind: store
    initialBlock: 20663734
    updatePolicy: set
    valueType: string
    inputs:
      - map: map_components

  - name: map_relative_balances
    kind: map
    initialBlock: 20663734
    inputs:
      - params: string
      - source: sf.ethereum.type.v2.Block
      - store: store_components
    output:
      type: proto:tycho.evm.v1.BlockBalanceDeltas

  - name: store_balances
    kind: store
    initialBlock: 20663734
    updatePolicy: add
    valueType: bigint
    inputs:
      - params: string
      - source: sf.ethereum.type.v2.Block
      - map: map_relative_balances

  - name: map_protocol_changes
    kind: map
    initialBlock: 20663734
    inputs:
      - params: string
      - source: sf.ethereum.type.v2.Block
      - map: map_components
      - map: map_relative_balances
      - store: store_components
      - store: store_balances
        mode: deltas
    output:
      type: proto:tycho.evm.v1.BlockChanges

network: mainnet

params:
  mainnet: >-
    83F20F44975D03b1b09e64809B757c47f942BEeA,  # sDAI
    6B175474E89094C44Da98b954EedeAC495271d0F,  # DAI
    3225737a9Bbb6473CB4a45b7244ACa2BeFdB276A,  # DAI-USDS Converter
    f6e72Db5454dd049d0788e411b06CfAF16853042,  # DAI Lite PSM
    A188EEC8F81263234dA3622A406892F3D630f98c,  # USDS PSM Wrapper
    a3931d71877C0E7a3148CB7Eb4463524FEc27fbD,  # sUSDS
    BDcFCA946b6CDd965f99a839e4435Bcdc1bc470B,  # MKR-SKY Converter
    dC035D45d973E3EC169d2276DDab16f1e407384F,  # USDS
    A0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48,  # USDC
    9f8F72aA9304c8B593d555F12eF6589cC3A579A2,  # MKR
    56072C95FAA701256059aa122697B133aDEd9279,  # SKY
    37305B1cD40574E4C5Ce33f8e8306Be057fD7341   # DAI Lite PSM USDC Holder