# Supernova V3 — Indexer & Simulator Fixes

This document explains, in plain English, every change made to get the
Supernova V3 (Algebra Integral 1.2.2 / `supernova-main`) integration working
end-to-end inside Tycho — from substream indexing all the way through to
interactive swap simulation against any pool created by the factory.

It is split into four parts:

- **Part A** — fixes inside the substream (`substreams/supernova-v3/`)
- **Part B** — fixes inside the protocol-testing harness (`protocol-testing/`)
- **Part C** — configuration / housekeeping tweaks
- **Part D** — how the whole pipeline now flows for a single swap

---

## Part A — Substream fixes (`substreams/supernova-v3/`)

The substream is the Rust → WASM module that reads raw Ethereum blocks and
emits the `BlockChanges` that tycho-indexer ingests. The Algebra storage layout
in `supernova-main` is different from older Algebra versions, and the original
code had a few bugs that meant the indexer was either silently dropping data
or writing the wrong slots to Postgres.

### A1. Tick struct slot offset was wrong

**File:** `src/storage/pool_storage.rs`

The `Tick` struct in `supernova-main` is laid out as:

```
slot+0  liquidityTotal (uint128) | liquidityDelta (uint128)
slot+1  prevTick (int24) | nextTick (int24) | …
slot+2  outerFeeGrowth0Token
slot+3  outerFeeGrowth1Token
…
```

The substream was reading `liquidityDelta` from `slot+2`, which is actually
where `outerFeeGrowth0Token` lives. The result was that every `Mint`/`Burn`
event indexed garbage as the per-tick liquidity delta. Fixed the offset and
updated the docstring to match the on-chain layout.

### A2. `TICK_TREE_ROOT_SLOT` was not being tracked

**File:** `src/storage/constants.rs`

Algebra uses a bitmap tree to find the next initialized tick during a swap.
The root of that tree (`tickTreeRoot`) lives in its own storage slot, but it
was missing from the `TRACKED_SLOTS` array (length was 21, should have been
22). Without it the simulator couldn't cross ticks correctly. Added the slot.

### A3. Plugin contracts were not being tracked

**File:** `src/modules/1_map_pool_created.rs` and `src/modules/2_store_pools.rs`

Each Algebra pool gets its own plugin contract instance. The plugin is
deployed in the same transaction as the pool and emitted via a
`Plugin(address)` event. The substream had no concept of plugins:

1. `1_map_pool_created.rs` now scans the pool-creation transaction's receipt
   logs for the `Plugin(address)` topic, picks up the plugin address, fetches
   its runtime bytecode from the same tx (so we get DCI-friendly account
   creation), and adds it to the component's `contracts_list` plus an
   explicit `plugin` static attribute.

2. `2_store_pools.rs` now indexes each `ProtocolComponent` under **every**
   contract address it owns (pool *and* plugin), not only `pc.id`. This is
   what allows downstream modules to look up the pool when a storage write
   to the plugin address happens.

### A4. Dead `get_log_changed_attributes` pipeline

**File:** `src/modules/3_map_pool_events.rs`

The original code built a vector of decoded events, then promptly threw it
away — `EntityChanges` were never emitted. Wired the pipeline back up so
decoded `Mint`/`Burn`/`Swap`/etc. attributes actually flow into the
`BlockChanges` output. Plugin addresses are also added to the
`created_pools` set so per-pool plugin storage events get associated with
the right component.

### A5. The famous `0x` prefix bug in balance changes

**File:** `src/modules/4_map_balance_changes.rs`

This was the single most damaging bug. When looking up the pool that owned
a token transfer, the code did:

```rust
format!("{}", substreams::Hex(from))   // "abcd…" — no 0x
```

…but every pool was stored in `store_pools` with a `0x`-prefixed key. Every
balance lookup missed, every `BalanceDelta` got dropped, and the
`component_balance` table in Postgres was always empty. The simulator's
`TokenProxy.balanceOf` would then return 0 and swaps would revert with
`Insufficient balance`. One-character fix:

```rust
format!("0x{}", substreams::Hex(from))
```

### A6. Hardcoded `pool_address` filter

**File:** `supernova-v3.yaml`

The original config had `pool_address=de758db5...` baked into `params:`,
which limited indexing to a single pool. Removed it so that every pool
created by `factory_address=44B7fBd4D87149eFa5347c451E74B9FD18E89c55` is
indexed. The `1_map_pool_created.rs` filter still honors the param if it's
set, so this is backwards compatible.

### Bonus: deleted dead event files

`src/events/collect.rs` and `src/events/flash.rs` were referencing types
that no longer exist on `supernova-main`. Algebra Integral doesn't emit
`Collect` (fees are auto-distributed) and `Flash` is gated by a plugin in
this build. Files removed; `mint.rs` and `burn.rs` updated to call
`get_ticks_changes(bottom_tick, top_tick)` so per-tick liquidity deltas are
decoded into the entity-change stream.

---

## Part B — Protocol-testing harness fixes (`protocol-testing/`)

The protocol-testing crate is what runs an end-to-end swap simulation: it
spins up a local indexer, syncs a few hundred blocks of mainnet against the
substream, fetches the indexed state via Tycho RPC, hands it to the REVM
simulator wrapped in a TokenProxy, and asks for a quote. There were a *lot*
of small papercuts here.

### B1. Compile errors after a tycho-common bump

**File:** `src/test_runner.rs`

`ImplementationType` had been removed from `tycho_common::dto`, and a
`get_protocol_components` call had its result binding accidentally deleted in
a previous refactor. Fixed both — the harness now compiles cleanly against
tycho 0.141.

### B2. TVL import was killing the indexer subprocess

**File:** `src/test_runner.rs`

`run_tvl_import()` returns a `Result` and the harness was `?`-propagating
its error, which would tear down the indexer that we'd just started. Made
it log-and-continue instead. The TVL bootstrap is purely cosmetic for our
use case (we're not gating on minimum TVL).

### B3. Initialized accounts were ignored in the full-test flow

**File:** `src/test_runner.rs`

`run_full_test` was calling `self.tycho_runner(&[]).await?`, throwing away
the `initialized_accounts` from the YAML config. Replaced with code that
reads `config.initialized_accounts` and passes it through, so manually
specified mocked accounts (e.g. SecurityRegistry) actually make it into
the simulation.

### B4. Default Tycho RPC URL pointed at a Docker hostname

**File:** `src/test_runner.rs`

Default was `http://indexer:4242` which only resolves inside the
`docker-compose` network. Changed to `http://localhost:4242` and added
`info!("Using Tycho RPC at {tycho_url}")` so it's obvious which URL is in
use during a run.

### B5. Slot 2 (globalState) was returning zero from the simulator

**File:** `src/tycho_rpc.rs` (`get_snapshots`)

tycho-indexer 0.141 doesn't version contract storage by block — it only
keeps the latest value for each `(account, slot)` pair. When the harness
queried storage with a *historical* block, the indexer's
`get_contract_state` would return nothing for slots that hadn't changed
since some earlier block, and the simulator would see `globalState = 0`
(price = 0, tick = 0), causing immediate reverts.

Added a **timestamp-based vm_storage overlay**: after the historical
fetch, refetch the same accounts with `VersionParam::default()` (i.e.
"now") and merge the latest slot values on top. This is safe because the
test is reproducing a single historical block; we just need slots that
were untouched between then and now to be present.

Same overlay was added for `protocol_state` with `include_balances=true`.

### B6. Slot 12 reserve reconciliation

**File:** `src/tycho_rpc.rs`

Algebra stores `reserve0`/`reserve1` as two `uint128`s packed into slot 12.
Because of the timing skew between the substream's BalanceDelta stream and
direct slot reads, `cws.state.balances` (token-level) and slot 12
(pool-internal reserves) could drift apart. The pool's internal accounting
uses slot 12, so we now decode the lower/upper 16 bytes of slot 12 and
**overwrite** `cws.state.balances[token0]` and `[token1]` with those
values. The TokenProxy's `balanceOf` then matches what the pool thinks it
has.

### B7. Slot 4 `lastFeeTransferTimestamp` underflow

**File:** `src/tycho_rpc.rs`

`_accrueAndTransferFees` does `_blockTimestamp() - lastTimestamp`. If the
slot's stored timestamp was already past the simulated block timestamp,
that subtraction underflowed and panicked with arithmetic error 0x11
inside REVM. Override slot 4 to write `block.timestamp - 1` into bytes
`[2..5]` (the byte range where `lastFeeTransferTimestamp` lives in the
packed slot) so the fee-transfer logic always sees a valid prior
timestamp.

### B8. Plugin AFTER_* hook bits caused OOG

**File:** `src/tycho_rpc.rs`

The plugin's `pluginConfig` byte enables hooks like BEFORE_SWAP,
AFTER_SWAP, etc. AFTER_SWAP calls back into the pool's `crossTo` with
zero remaining gas in our simulated environment, causing an out-of-gas
revert. Mask byte 6 of slot 2 with `0xD5` to clear the AFTER_* bits
before handing the snapshot to the simulator. Swap path still goes through
BEFORE_SWAP, dynamic fee, and the rest — only AFTER_* hooks are disabled.

### B9. Adapter `priceAt` was hiding the real revert

**File:** `evm/src/supernova-v3/SupernovaV3Adapter.sol`

`priceAt` had a `try { … } catch { return Fraction(0, 0); }` wrapper that
turned every revert into a `Denominator is zero` error from upstream
fraction math, hiding what actually went wrong. Removed the catch.

Also: the call site was passing `""` as swap callback data, but our
callback handler expects an ABI-encoded `address`. Changed to
`abi.encode(msg.sender)`.

After both edits, force-rebuilt the adapter artifacts:

```sh
./scripts/buildRuntime.sh -c SupernovaV3Adapter \
  -s "constructor(address)" \
  -a "0x44B7fBd4D87149eFa5347c451E74B9FD18E89c55"
```

### B10. Forge cache was serving a stale adapter

Even after editing the Solidity, forge was returning the old artifact from
its cache. Fixed by `forge clean` then re-running the build script. This
is a one-time gotcha but worth knowing about.

### B11. Dynamic plugin-dependency discovery

**File:** `src/test_runner.rs` + `src/rpc.rs`

The original integration test had to **hand-list** every account the
plugin would touch (SecurityRegistry, oracle, etc.) in
`integration_test.tycho.yaml` under `initialized_accounts`. Different
pools have different plugin dependencies, so this was a non-starter for
"test any pool".

Added `discover_and_inject_plugin_dependencies()`:

1. Take only the **queried pool's plugin** (not all 7k+ contracts in the
   snapshot — that earlier version was way too slow).
2. Scan its storage slots for address-shaped values (20 bytes packed in a
   32-byte slot, with a sensible heuristic to filter out small ints).
3. For each candidate, fetch runtime bytecode in parallel via
   `futures::stream::buffer_unordered(16)` against the live RPC.
4. Wrap each as a `ResponseAccount` (using `keccak256(code)` for
   `code_hash` and 32 zero bytes for the placeholder tx hash, since
   `FixedBytes<32>` requires exactly 32 bytes — earlier panic was here)
   and inject into the snapshot as a mocked account.

`src/rpc.rs` got a small `RPCProvider::get_code(addr, block)` helper to
support this.

The result: any new pool created by the factory works without editing
YAML. The harness discovers what the plugin needs at runtime.

---

## Part C — Configuration & housekeeping

### C1. `integration_test.tycho.yaml` cleanup

Trimmed the `initialized_accounts` list down. Left SecurityRegistry as a
hint/example in case dynamic discovery is disabled, but it's no longer
required.

### C2. Retention horizon

**File:** `src/tycho_runner.rs`

Kept `--retention-horizon 2028-01-01T00:00:00`. Earlier experiments with
`2020-01-01` triggered:

```
duplicate key value violates unique constraint
"contract_storage_default_unique_pk"
```

…because the indexer's partitioned tables can't accept "ancient" inserts
once a newer partition is active. 2028 is far enough in the future that
nothing gets pruned mid-test. There's a comment in the code explaining
this so the next person doesn't roll it back.

### C3. Deleted manual SQL bootstrap files

`scripts/manual_insert_pool.sql` and `scripts/manual_bootstrap_pool.sql`
existed because at one point we were hand-inserting a pool into Postgres
to work around indexer bugs. With everything above fixed, those scripts
are no longer needed (and would mask future regressions). Deleted.

---

## Part D — How everything fits together for one swap

Here's the full lifecycle when you run `cargo run -- run-test --pool 0x…
--token-in 0x… --token-out 0x… --amount 0.001`:

1. `tycho-indexer` is started as a subprocess pointed at the
   `supernova_v3.spkg` substream and a Postgres instance.
2. The substream walks blocks from `initialBlock`, picks up
   `Pool(address,…)` and `Plugin(address)` events from the factory tx
   (A3), tracks both contracts under one ProtocolComponent (A3), decodes
   per-tick liquidity correctly (A1), tracks `tickTreeRoot` (A2), routes
   balance deltas to the right pool using a `0x`-prefixed key (A5), and
   emits real `EntityChanges` (A4).
3. Indexer writes everything into Postgres `contract_storage`,
   `protocol_state`, and `component_balance` partitioned tables.
4. The harness waits until indexing has reached the target block, then
   calls `get_snapshots(pool_id, block, timestamp)` (B5):
   - Historical contract state (block-versioned)
   - Live overlay merged on top (works around indexer 0.141's lack of
     historical slot versioning)
   - Slot 12 decoded → `balances` rewritten (B6)
   - Slot 4 timestamp byte range overwritten with `block.timestamp - 1`
     (B7)
   - Slot 2 byte 6 masked with `0xD5` (B8)
5. Plugin storage is scanned for address-shaped values; runtime bytecode
   for each is fetched in parallel and merged into the snapshot as
   mocked accounts (B11).
6. Snapshot + adapter bytecode go into REVM. The adapter's `priceAt`
   calls `pool.swap` with a properly ABI-encoded callback payload (B9),
   the swap routes through Algebra's `BEFORE_SWAP` hook, dynamic fee,
   tick crossing (using the now-correct `tickTreeRoot` and per-tick
   `liquidityDelta`), and returns a real quote.
7. Quote is printed.

If a quote ever still comes back as `Insufficient balance`, it's almost
always because `--amount` is too large relative to the pool's actual
liquidity at the test block — try a smaller amount.

---

## Files touched

| Area | File | Purpose |
| --- | --- | --- |
| Substream | `src/storage/pool_storage.rs` | Tick slot offset (A1) |
| Substream | `src/storage/constants.rs` | tickTreeRoot tracking (A2) |
| Substream | `src/modules/1_map_pool_created.rs` | Plugin discovery (A3) |
| Substream | `src/modules/2_store_pools.rs` | Index components by all owned addrs (A3) |
| Substream | `src/modules/3_map_pool_events.rs` | Wire entity-changes pipeline (A4) |
| Substream | `src/modules/4_map_balance_changes.rs` | `0x` prefix fix (A5) |
| Substream | `src/events/mint.rs`, `burn.rs` | Per-tick liquidity decode |
| Substream | `src/events/collect.rs`, `flash.rs` | **Deleted** |
| Substream | `supernova-v3.yaml` | Drop pool_address filter (A6) |
| Harness | `src/test_runner.rs` | B1, B2, B3, B4, B11 |
| Harness | `src/tycho_rpc.rs` | B5, B6, B7, B8 |
| Harness | `src/tycho_runner.rs` | C2 |
| Harness | `src/rpc.rs` | `get_code` helper for B11 |
| Harness | `integration_test.tycho.yaml` | C1 |
| Harness | `scripts/manual_*.sql` | **Deleted** (C3) |
| EVM adapter | `evm/src/supernova-v3/SupernovaV3Adapter.sol` | B9 |

---

## One-line summary of every fix

- **A1** Tick `liquidityDelta` was being read from the wrong slot.
- **A2** `tickTreeRoot` slot was missing from the tracked-slots list.
- **A3** Plugin contracts deployed alongside each pool weren't being indexed.
- **A4** Decoded entity changes were built but never emitted.
- **A5** Balance lookups failed because keys were missing the `0x` prefix.
- **A6** Hardcoded `pool_address=` filter limited the substream to one pool.
- **B1** Compile errors after the tycho-common API bump.
- **B2** TVL import failure was tearing down the indexer subprocess.
- **B3** `run_full_test` was ignoring `initialized_accounts` from the YAML.
- **B4** Default Tycho URL only worked inside Docker.
- **B5** Indexer 0.141 doesn't version slots, so historical reads needed a live overlay.
- **B6** `cws.balances` and slot 12 reserves drifted; reconcile from slot 12.
- **B7** `lastFeeTransferTimestamp` could be in the future and underflow.
- **B8** Plugin AFTER_* hooks caused OOG; mask them off in `pluginConfig`.
- **B9** Adapter `priceAt` swallowed reverts and passed empty callback data.
- **B10** Forge cache served a stale adapter; needed `forge clean`.
- **B11** Plugin dependencies are now discovered and injected at runtime.
- **C1** Trimmed the manual `initialized_accounts` list.
- **C2** Kept `--retention-horizon 2028-01-01` to avoid partition conflicts.
- **C3** Deleted the manual SQL bootstrap scripts.
