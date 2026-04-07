# Running the Supernova V3 Indexer & Simulator

This file is a step-by-step runbook for getting the Supernova V3 substream
indexed and then running interactive swap simulations against any pool. If
you want to know **why** any of these steps exist, see `README.md` next to
this file.

All commands assume the repo root is
`~/Documents/tycho-protocol-sdk` — adjust paths if yours differs.

---

## 0. One-time prerequisites

You only need to do these once per machine.

### 0.1 Toolchain

```bash
# Rust + the wasm target the substream compiles to
rustup target add wasm32-unknown-unknown

# Foundry (for compiling the EVM adapter)
curl -L https://foundry.paradigm.xyz | bash
foundryup

# libpq (needed by tycho-indexer on macOS)
brew install libpq
echo 'export PATH="/opt/homebrew/opt/libpq/bin:$PATH"' >> ~/.zshrc
export PKG_CONFIG_PATH="/opt/homebrew/opt/libpq/lib/pkgconfig:$PKG_CONFIG_PATH"
```

### 0.2 Install `tycho-indexer`

The protocol-testing harness shells out to `tycho-indexer`, so it must be on
your `PATH`.

```bash
cargo install --git https://github.com/propeller-heads/tycho-indexer.git tycho-indexer
which tycho-indexer   # sanity check
```

### 0.3 Environment variables

```bash
export RPC_URL="https://eth-mainnet.g.alchemy.com/v2/<your-key>"
export SUBSTREAMS_API_TOKEN="<your-streamingfast-token>"
```

You can also drop these into a `.env` next to `protocol-testing/` and
`source` it.

---

## 1. Build the substream

```bash
cd ~/Documents/tycho-protocol-sdk/substreams/supernova-v3
make build
```

This produces `../target/wasm32-unknown-unknown/release/supernova_v3.wasm`,
which is the binary referenced from `supernova-v3.yaml`.

If you ever see `Blocking waiting for file lock on build directory`, kill
stale cargo/rustc processes and remove the lock:

```bash
pkill -f cargo; pkill -f rustc
rm -f ~/Documents/tycho-protocol-sdk/substreams/target/.cargo-lock
```

---

## 2. Build the EVM adapter

The simulator runs `SupernovaV3Adapter.sol` inside REVM. The artifact has
to be built (and rebuilt every time you edit the Solidity).

```bash
cd ~/Documents/tycho-protocol-sdk/evm
forge clean    # only needed if you edited the Solidity since the last build
./scripts/buildRuntime.sh \
  -c SupernovaV3Adapter \
  -s "constructor(address)" \
  -a "0x44B7fBd4D87149eFa5347c451E74B9FD18E89c55"
```

The constructor arg is the Algebra factory address.

---

## 3. Start Postgres

The harness expects a Postgres at
`postgres://postgres:mypassword@localhost:5431/tycho_indexer_0` by default.
The repo ships a `docker-compose` for it.

```bash
cd ~/Documents/tycho-protocol-sdk
docker buildx build -f protocol-testing/postgres.Dockerfile \
  -t protocol-testing-db:latest --load .
docker compose -f protocol-testing/docker-compose.yaml up db -d
```

Verify it's listening:

```bash
docker ps | grep protocol-testing-db
```

> If you've previously run the harness against a different `--retention-horizon`
> or a different substream version, **drop the volume** before re-syncing,
> otherwise indexer panics with `NoRelatedEntity` or
> `duplicate key value violates unique constraint`:
>
> ```bash
> docker compose -f protocol-testing/docker-compose.yaml down -v
> ```

---

## 4. Run the indexer + range test

The simplest sanity check — index a fixed block range and verify the
expected components from `integration_test.tycho.yaml`:

```bash
cd ~/Documents/tycho-protocol-sdk/protocol-testing
cargo run -- range --package "supernova-v3"
```

You should see:

- `Using Tycho RPC at http://localhost:4242`
- substream syncing blocks
- a green check on the expected pool component

---

## 5. Run a full sync (index everything from the factory's first block)

Use this when you want to interactively swap against **any** pool the
factory has ever created. It runs the substream from `initialBlock` (set in
`supernova-v3.yaml`) up to the latest block, then keeps the indexer alive
so the interactive command can talk to it.

```bash
cd ~/Documents/tycho-protocol-sdk/protocol-testing
cargo run -- full --package "supernova-v3"
```

Useful flags:

- `--initial-block <N>` — start from a later block to skip ahead.
- `--reuse-last-sync` — resume from whatever's already in the DB instead of
  wiping it. **Don't** use this if you've changed the substream code since
  the last run.
- `--vm-simulation-traces` — print REVM traces (verbose, but invaluable
  when debugging a revert).

Leave this process running in its own terminal — step 6 needs the indexer
it spawns.

---

## 6. Run an interactive swap simulation

In a **second terminal**, with the `full` run from step 5 still alive:

```bash
cd ~/Documents/tycho-protocol-sdk/protocol-testing

cargo run -- interactive \
  --package "supernova-v3" \
  --pool       0x2beb35e78c9427899353c41c96bcc96c5647ec63 \
  --token-in   0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48 \
  --token-out  0xdac17f958d2ee523a2206206994597c13d831ec7 \
  --amount     1000000
```

Notes:

- `--pool` can be **any** address indexed by the factory — plugin
  dependencies are discovered at runtime, so you don't need to pre-list
  them in YAML.
- `--amount` is the **raw** amount in token base units (so for USDC with
  6 decimals, `1000000` = `1 USDC`). If you get `Insufficient balance`,
  the pool just doesn't have enough liquidity at the test block — try a
  smaller number.
- Quotes are returned synchronously in the terminal where you ran the
  `interactive` command.

---

## 7. Inspecting Postgres while the indexer runs

The harness's Postgres lives on **port 5431** (`protocol-testing-db-1`).
Don't confuse it with any other tycho DB you might have on 5432/5433.

```bash
docker exec -it protocol-testing-db-1 \
  psql -U postgres -d tycho_indexer_0
```

Useful queries once inside `psql`:

```sql
-- How many components were indexed?
SELECT COUNT(*) FROM protocol_component
WHERE protocol_system_id =
  (SELECT id FROM protocol_system WHERE name = 'supernova-v3-vm');

-- Storage rows for a specific pool
SELECT COUNT(*) FROM contract_storage cs
JOIN account a ON a.id = cs.account_id
WHERE a.address = '\x2beb35e78c9427899353c41c96bcc96c5647ec63';

-- Per-component balances (this should NOT be empty — if it is, the
-- substream's `0x` prefix bug regressed)
SELECT * FROM component_balance LIMIT 5;
```

---

## 8. Tearing down

```bash
# Stop the running indexer (Ctrl-C in the terminal where step 5 is)

# Stop and wipe Postgres
cd ~/Documents/tycho-protocol-sdk
docker compose -f protocol-testing/docker-compose.yaml down -v
```

---

## TL;DR — happy path

```bash
# one terminal
cd ~/Documents/tycho-protocol-sdk/substreams/supernova-v3 && make build
cd ~/Documents/tycho-protocol-sdk/evm && ./scripts/buildRuntime.sh \
  -c SupernovaV3Adapter -s "constructor(address)" \
  -a "0x44B7fBd4D87149eFa5347c451E74B9FD18E89c55"
docker compose -f ~/Documents/tycho-protocol-sdk/protocol-testing/docker-compose.yaml up db -d

cd ~/Documents/tycho-protocol-sdk/protocol-testing
export RPC_URL=...
export SUBSTREAMS_API_TOKEN=...
cargo run -- full --package "supernova-v3"

# second terminal
cd ~/Documents/tycho-protocol-sdk/protocol-testing
cargo run -- interactive --package "supernova-v3" \
  --pool 0x2beb35e78c9427899353c41c96bcc96c5647ec63 \
  --token-in 0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48 \
  --token-out 0xdac17f958d2ee523a2206206994597c13d831ec7 \
  --amount 1000000
```
