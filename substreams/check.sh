set -e

PACKAGES=(
  base-aerodrome-slipstreams
  ethereum-balancer-v2
  ethereum-balancer-v3
  ethereum-cowamm
  ethereum-curve
  ethereum-erc4626
  ethereum-maverick-v2
  ethereum-pancakeswap-v3
  ethereum-rocketpool
  ethereum-template-factory
  ethereum-template-singleton
  ethereum-uniswap-v2
  ethereum-uniswap-v3
  ethereum-uniswap-v3-logs-only
  ethereum-uniswap-v4-no-hooks
  ethereum-uniswap-v4-shared
  ethereum-uniswap-v4-with-hooks
  tycho-substreams
  unichain-velodrome
)

for package in "${PACKAGES[@]}"; do
  echo "Running checks for package: $package"
  cargo +nightly fmt --package "$package" -- --check
  cargo +nightly clippy --package "$package" -- -D warnings
done

cargo build --target wasm32-unknown-unknown --all-targets --all-features
cargo test --workspace --all-targets --all-features