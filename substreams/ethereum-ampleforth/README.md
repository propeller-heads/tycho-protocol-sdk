### Setup

```
# 1) Install Rust
curl https://sh.rustup.rs -sSf | sh

# 2) Add cargo to bash env 

# 3) Install substreams and protocol buf
brew install streamingfast/tap/substreams
brew install buf

# 4) Build
rustup target add wasm32-unknown-unknown
# NOTE: This will run build.rs and generate abi bindings
cargo build --target wasm32-unknown-unknown --release

# 5) Generate protobuf code
substreams protogen substreams.yaml --exclude-paths="sf/substreams,google"
```


### Running

```
substreams auth
export SUBSTREAMS_API_TOKEN=INSERT_TOKEN

substreams gui
substreams run -e mainnet.eth.streamingfast.io:443 substreams.yaml
```
