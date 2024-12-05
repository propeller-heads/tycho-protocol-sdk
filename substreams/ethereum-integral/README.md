# Protocol Components

- RelayerProxy
  - Track both Instant buy and sells for proxy balance state
  - Additionally, need events that the Delay contracts start pull from relayer and trading with pair
- UniswapPair (from Factory)
  - Need Buy and Sell events to keep track of balances of the pairs (normal)
