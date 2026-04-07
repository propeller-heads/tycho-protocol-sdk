use std::{collections::HashMap, error::Error as StdError, fmt, str::FromStr};

use tokio::time::{sleep, Duration};
use tracing::{debug, info, warn};
use tycho_simulation::{
    tycho_client::{
        feed::synchronizer::Snapshot,
        rpc::{HttpRPCClientOptions, RPCClient},
        HttpRPCClient, SnapshotParameters,
    },
    tycho_common::{
        dto::{
            Chain, EntryPointWithTracingParams, PaginationParams, ProtocolComponent,
            ProtocolComponentsRequestBody, ResponseToken, TracedEntryPointRequestBody,
            TracingResult, VersionParam,
        },
        models::{token::Token, ComponentId},
        Bytes,
    },
};

/// Custom error type for RPC operations
#[derive(Debug)]
pub enum RpcError {
    ClientError(String),
}

impl fmt::Display for RpcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RpcError::ClientError(msg) => write!(f, "RPC client error: {msg}"),
        }
    }
}

impl StdError for RpcError {}

impl From<Box<dyn StdError>> for RpcError {
    fn from(error: Box<dyn StdError>) -> Self {
        RpcError::ClientError(error.to_string())
    }
}

impl From<tycho_simulation::tycho_client::RPCError> for RpcError {
    fn from(error: tycho_simulation::tycho_client::RPCError) -> Self {
        RpcError::ClientError(error.to_string())
    }
}

/// Client for interacting with the Tycho RPC server
pub struct TychoClient {
    http_client: HttpRPCClient,
}

impl TychoClient {
    pub fn new(host: &str, auth_key: Option<String>) -> Result<Self, RpcError> {
        let options = HttpRPCClientOptions::new().with_auth_key(auth_key);
        let http_client =
            HttpRPCClient::new(host, options).map_err(|e| RpcError::ClientError(e.to_string()))?;
        Ok(Self { http_client })
    }

    /// Gets protocol components from the RPC server
    pub async fn get_protocol_components(
        &self,
        protocol_system: &str,
        chain: Chain,
    ) -> Result<Vec<ProtocolComponent>, RpcError> {
        let request = ProtocolComponentsRequestBody::system_filtered(protocol_system, None, chain);

        let chunk_size = 100;
        let concurrency = 1;

        let response = self
            .http_client
            .get_protocol_components_paginated(&request, Some(chunk_size), concurrency)
            .await?;

        Ok(response.protocol_components)
    }

    pub async fn get_tokens(
        &self,
        chain: Chain,
        min_quality: Option<i32>,
        max_days_since_last_trade: Option<u64>,
    ) -> Result<HashMap<Bytes, Token>, RpcError> {
        debug!("Loading tokens from Tycho...");

        let concurrency = 1;

        #[allow(clippy::mutable_key_type)]
        let res = self
            .http_client
            .get_all_tokens(chain, min_quality, max_days_since_last_trade, Some(3_000), concurrency)
            .await?
            .into_iter()
            .map(|token| {
                let mut token_clone: ResponseToken = token.clone();
                // Set default gas if empty
                // TODO: Check if this interferes with simulation logic
                if token_clone.gas.is_empty() {
                    token_clone.gas = vec![Some(44000_u64)];
                }
                (
                    token_clone.address.clone(),
                    token_clone
                        .try_into()
                        .expect("Failed to convert token"),
                )
            })
            .collect::<HashMap<_, Token>>();

        Ok(res)
    }

    /// Gets traced entry points from the RPC server
    pub async fn get_traced_entry_points(
        &self,
        protocol_system: &str,
        component_ids: Vec<String>,
        chain: Chain,
    ) -> Result<HashMap<String, Vec<(EntryPointWithTracingParams, TracingResult)>>, RpcError> {
        let request_body = TracedEntryPointRequestBody {
            protocol_system: protocol_system.to_string(),
            chain,
            pagination: PaginationParams { page: 0, page_size: 100 },
            component_ids: Some(component_ids),
        };

        let traced_entry_points = self
            .http_client
            .get_traced_entry_points(&request_body)
            .await?;

        Ok(traced_entry_points.traced_entry_points)
    }

    pub async fn get_snapshots(
        &self,
        chain: Chain,
        block_number: u64,
        protocol_system: &str,
        components: &HashMap<ComponentId, ProtocolComponent>,
        contract_ids: &[Bytes],
        entrypoints: &HashMap<String, Vec<(EntryPointWithTracingParams, TracingResult)>>,
        block_timestamp_secs: Option<u32>,
    ) -> Result<Snapshot, RpcError> {
        let params =
            SnapshotParameters::new(chain, protocol_system, components, contract_ids, block_number)
                .entrypoints(entrypoints);

        let chunk_size = 100;
        let concurrency = 1;

        let mut response = self
            .http_client
            .get_snapshots(&params, Some(chunk_size), concurrency)
            .await?;

        // Workaround for tycho-indexer 0.141.0 storage versioning behavior:
        //
        // The block-based version inside `get_snapshots` constructs
        // `VersionParam { block: Some(block_number), timestamp: None }`. The indexer then
        // resolves block_number → its on-chain timestamp T, and queries
        // `WHERE valid_from <= T AND valid_to > T` against the partitioned tables. Because
        // tycho-indexer overwrites prior values in-place (it does not archive into daily
        // partitions unless `--retention-horizon` is in the past, which currently breaks
        // routing — see comments in tycho_runner.rs), each (account, slot) and (component,
        // token) row has `valid_from` set to the on-chain timestamp of the *most recent*
        // write. If `block_number` is even one block stale relative to that, the row's
        // `valid_from` is greater than T and the row is excluded → simulator sees zero
        // for slot 2 (globalState), slot 12 (reserves), AND missing balances for the pool's
        // tokens (which the simulator injects into TokenProxy stubs).
        //
        // Workaround: re-fetch contract state AND protocol state with
        // `VersionParam::default()` (which uses `Utc::now()` as the timestamp). The current
        // row's `valid_to` is the indexer's infinity sentinel (262142-12-31), so
        // `valid_from <= now AND valid_to > now` always matches. We then overlay the
        // results onto the snapshot's `vm_storage` (raw contract state) and
        // `states[*].state.balances` / `attributes` (decoded protocol state).
        let now_version = VersionParam::default();

        if !contract_ids.is_empty() {
            let refreshed = self
                .http_client
                .get_contract_state_paginated(
                    chain,
                    contract_ids,
                    protocol_system,
                    &now_version,
                    Some(chunk_size),
                    concurrency,
                )
                .await?;
            for account in refreshed.accounts {
                let slot_count = account.slots.len();
                let addr_hex = format!("0x{}", hex::encode(&account.address));
                response
                    .vm_storage
                    .insert(account.address.clone(), account);
                info!(addr = %addr_hex, slot_count, "Overlayed account vm_storage");
            }
            debug!(
                contracts = contract_ids.len(),
                "Overlayed snapshot vm_storage with timestamp-based contract_state"
            );
        }

        // Refresh protocol_state (attributes + balances) for every component in the
        // snapshot. The simulator reads `state.balances` to populate its TokenProxy
        // overwrites — without this overlay, balances are empty and the swap math
        // sees `balanceOf(pool) == 0`.
        if !components.is_empty() {
            let component_ids: Vec<String> = components.keys().cloned().collect();
            let refreshed_states = self
                .http_client
                .get_protocol_states_paginated(
                    chain,
                    &component_ids,
                    protocol_system,
                    true, // include_balances
                    &now_version,
                    Some(chunk_size),
                    concurrency,
                )
                .await?;
            let by_id: HashMap<String, _> = refreshed_states
                .states
                .into_iter()
                .map(|s| (s.component_id.clone(), s))
                .collect();
            let mut updated = 0usize;
            for (cid, cws) in response.states.iter_mut() {
                if let Some(fresh) = by_id.get(cid) {
                    cws.state = fresh.clone();
                    updated += 1;
                }
            }
            debug!(
                components = component_ids.len(),
                updated,
                "Overlayed snapshot protocol_state with timestamp-based balances/attributes"
            );
        }

        // Reconcile component balances with the pool's on-chain stored reserves.
        //
        // For Algebra Integral / Supernova v3 the pool's `_updateReserves()` reads the
        // ERC20 balance via `balanceOf(pool)` AND compares it against the internally-stored
        // reserve in slot 12 (packed `reserve0|reserve1`). The simulator injects the ERC20
        // balance via `tycho_simulation::evm::protocol::vm::erc20_token::TokenProxy` based
        // on the snapshot's `state.balances` (which comes from `component_balance`,
        // populated by Transfer-event aggregation in the substream). These two sources
        // can drift by a few wei when the indexer is between blocks, and any drift causes
        // the pool's swap math to underflow with `panic 0x11` (ArithmeticOver/Underflow).
        //
        // To eliminate the drift entirely, override the snapshot's component balances
        // with the values decoded from the pool's slot 12. The pool's stored reserve IS
        // the source of truth for both `_updateReserves()` and the swap math; aligning
        // the simulator's TokenProxy balance with this value guarantees consistency.
        //
        // Slot 12 layout (Algebra `ReservesManager`):
        //   bytes 16..31 (offset 0,  16 B) → reserve0  (uint128)
        //   bytes  0..15 (offset 16, 16 B) → reserve1  (uint128)
        for (cid, cws) in response.states.iter_mut() {
            // The component id is the pool address; vm_storage is keyed by Bytes(address).
            let pool_addr = match Bytes::from_str(cid) {
                Ok(a) => a,
                Err(_) => continue,
            };
            let acc = match response.vm_storage.get(&pool_addr) {
                Some(a) => a,
                None => continue,
            };
            // Slot 12 = 0x0c right-padded to 32 bytes.
            let slot12_key = Bytes::from(
                hex::decode(
                    "000000000000000000000000000000000000000000000000000000000000000c",
                )
                .unwrap(),
            );
            let slot12_val = match acc.slots.get(&slot12_key) {
                Some(v) if v.len() == 32 => v.clone(),
                _ => continue,
            };
            // reserve0 = lower 16 bytes (right end), reserve1 = upper 16 bytes (left end).
            let reserve1_bytes: Vec<u8> = slot12_val[0..16].to_vec();
            let reserve0_bytes: Vec<u8> = slot12_val[16..32].to_vec();
            // Tokens in the component are ordered token0, token1.
            let token0 = match cws.component.tokens.first() {
                Some(t) => t.clone(),
                None => continue,
            };
            let token1 = match cws.component.tokens.get(1) {
                Some(t) => t.clone(),
                None => continue,
            };
            cws.state.balances.insert(token0.clone(), Bytes::from(reserve0_bytes.clone()));
            cws.state.balances.insert(token1.clone(), Bytes::from(reserve1_bytes.clone()));
            info!(
                pool = %cid,
                reserve0 = %hex::encode(&reserve0_bytes),
                reserve1 = %hex::encode(&reserve1_bytes),
                "Reconciled component balances from pool slot 12 reserves"
            );
        }

        // Override slot 4's `lastFeeTransferTimestamp` for every pool component so that
        // `_blockTimestamp() - lastTimestamp` evaluates to a small positive value LESS
        // than `FEE_TRANSFER_FREQUENCY` (= 5 seconds).
        //
        // Two problems we're avoiding:
        //   (a) Underflow in `_accrueAndTransferFees` (panic 0x11): if the on-chain
        //       lastTimestamp written by the indexer is greater than the simulator's
        //       BlockEnv timestamp, the unchecked uint32 subtraction reverts.
        //   (b) Triggering the fee-transfer code path: if the diff is ≥ 5, the pool calls
        //       `_transferFees` which transfers ERC20 to communityVault and invokes
        //       `plugin.handlePluginFee(...)`. Both of these depend on contracts whose
        //       state we don't have in the snapshot, so they revert.
        //
        // Setting `lastTimestamp = block.timestamp - 1` makes the diff exactly `1`, which
        // is `< 5` and `>= 0`, satisfying both constraints — fees just accrue in pending.
        //
        // Pool's slot 4 packs (LSB → MSB):
        //   bytes [0..12]  (slot byte 19..31): communityFeePending0
        //   bytes [13..25] (slot byte 6..18):  communityFeePending1
        //   bytes [26..29] (slot byte 2..5):   lastFeeTransferTimestamp (uint32)
        //
        // We only modify the timestamp bytes (slot byte indices 2..5), preserving fees.
        // Collect pool addresses once for both slot 4 and slot 2 overrides.
        let pool_addrs: Vec<Bytes> = response
            .states
            .keys()
            .filter_map(|cid| Bytes::from_str(cid).ok())
            .collect();

        if let Some(ts) = block_timestamp_secs {
            // Use ts.saturating_sub(1) so we never wrap around when ts == 0.
            let safe_ts = ts.saturating_sub(1);
            let ts_bytes = safe_ts.to_be_bytes();
            let slot4_key = Bytes::from(
                hex::decode(
                    "0000000000000000000000000000000000000000000000000000000000000004",
                )
                .unwrap(),
            );
            for pool_addr in &pool_addrs {
                let Some(acc) = response.vm_storage.get_mut(pool_addr) else {
                    continue;
                };
                if let Some(slot4_val) = acc.slots.get(&slot4_key).cloned() {
                    if slot4_val.len() == 32 {
                        let mut new_val: Vec<u8> = slot4_val.to_vec();
                        // Slot byte indices 2..5 hold the uint32 lastFeeTransferTimestamp.
                        new_val[2] = ts_bytes[0];
                        new_val[3] = ts_bytes[1];
                        new_val[4] = ts_bytes[2];
                        new_val[5] = ts_bytes[3];
                        acc.slots.insert(slot4_key.clone(), Bytes::from(new_val));
                        info!(
                            pool = %format!("0x{}", hex::encode(pool_addr)),
                            lastFeeTransferTimestamp = safe_ts,
                            "Overrode slot 4 lastFeeTransferTimestamp = block.timestamp - 1"
                        );
                    }
                }
            }
        }

        // Override slot 2 (globalState) to clear the after-hook bits in pluginConfig.
        //
        // Bit map of pluginConfig (uint8):
        //   bit 0 (0x01) BEFORE_SWAP_FLAG          ← keep (plugin's beforeSwap is needed
        //                                              to compute dynamic fee)
        //   bit 1 (0x02) AFTER_SWAP_FLAG           ← CLEAR (calls IAlgebraVirtualPool.crossTo
        //                                              with too little gas, OOG reverts)
        //   bit 2 (0x04) BEFORE_POSITION_MODIFY    ← keep (we don't modify positions)
        //   bit 3 (0x08) AFTER_POSITION_MODIFY     ← CLEAR (defensive)
        //   bit 4 (0x10) BEFORE_FLASH              ← keep
        //   bit 5 (0x20) AFTER_FLASH               ← CLEAR (defensive)
        //   bit 6 (0x40) AFTER_INIT                ← keep
        //   bit 7 (0x80) DYNAMIC_FEE               ← keep (fee comes from plugin)
        //
        // Mask = 0xFF & ~(0x02 | 0x08 | 0x20) = 0xD5
        //
        // Slot 2 packs `GlobalState` LSB→MSB:
        //   [0..19]  price          uint160  20 B
        //   [20..22] tick           int24     3 B
        //   [23..24] lastFee        uint16    2 B
        //   [25]     pluginConfig   uint8     1 B
        //   [26..27] communityFee   uint16    2 B
        //   [28]     unlocked       bool      1 B
        //
        // In a 32-byte big-endian slot, offset N from the right = slot byte index (31 - N).
        // pluginConfig is at offset 25 → slot byte index 6.
        let slot2_key = Bytes::from(
            hex::decode(
                "0000000000000000000000000000000000000000000000000000000000000002",
            )
            .unwrap(),
        );
        const AFTER_HOOK_MASK: u8 = !(0x02 | 0x08 | 0x20); // = 0xD5
        for pool_addr in &pool_addrs {
            let Some(acc) = response.vm_storage.get_mut(pool_addr) else {
                continue;
            };
            if let Some(slot2_val) = acc.slots.get(&slot2_key).cloned() {
                if slot2_val.len() == 32 {
                    let mut new_val: Vec<u8> = slot2_val.to_vec();
                    let old_cfg = new_val[6];
                    let new_cfg = old_cfg & AFTER_HOOK_MASK;
                    if new_cfg != old_cfg {
                        new_val[6] = new_cfg;
                        acc.slots.insert(slot2_key.clone(), Bytes::from(new_val));
                        info!(
                            pool = %format!("0x{}", hex::encode(pool_addr)),
                            old_cfg = format!("0x{:02x}", old_cfg),
                            new_cfg = format!("0x{:02x}", new_cfg),
                            "Cleared AFTER_* plugin hook bits in slot 2 pluginConfig"
                        );
                    }
                }
            }
        }

        Ok(response)
    }

    /// Waits for the protocol to be synced and have components available
    pub async fn wait_for_protocol_sync(
        &self,
        protocol_system: &str,
        chain: Chain,
    ) -> Result<(), RpcError> {
        loop {
            match self
                .get_protocol_components(protocol_system, chain)
                .await
            {
                Ok(components) => {
                    info!("Found {} components for protocol {}", components.len(), protocol_system);
                    if !components.is_empty() {
                        return Ok(());
                    }
                    info!(
                        "Protocol {} found but no components available yet, waiting...",
                        protocol_system
                    );
                }
                Err(e) => {
                    warn!(
                        "Failed to get protocol components for {}: {}. Retrying in 15 minutes...",
                        protocol_system, e
                    );
                }
            }

            sleep(Duration::from_secs(15 * 60)).await;
        }
    }
}
