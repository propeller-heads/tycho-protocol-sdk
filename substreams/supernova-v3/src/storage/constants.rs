use hex_literal::hex;

use super::pool_storage::StorageLocation;

// -------- SLOT CONSTANTS --------
// All offsets are from the right (least-significant byte = offset 0),
// matching EVM big-endian packing.

// slot 0 → totalFeeGrowth0Token
pub const SLOT_0: [u8; 32] =
    hex!("0000000000000000000000000000000000000000000000000000000000000000");

// slot 1 → totalFeeGrowth1Token
pub const SLOT_1: [u8; 32] =
    hex!("0000000000000000000000000000000000000000000000000000000000000001");

// slot 2 → globalState (packed 28-byte struct)
pub const SLOT_2: [u8; 32] =
    hex!("0000000000000000000000000000000000000000000000000000000000000002");

// slot 3 → ticks (mapping root — entries computed via keccak)
pub const TICKS_MAP_SLOT: [u8; 32] =
    hex!("0000000000000000000000000000000000000000000000000000000000000003");

// slot 4 → communityFeePending0 + communityFeePending1 + lastFeeTransferTimestamp (packed 30 B)
pub const SLOT_4: [u8; 32] =
    hex!("0000000000000000000000000000000000000000000000000000000000000004");

// slot 5 → pluginFeePending0 + pluginFeePending1 (packed 26 B)
pub const SLOT_5: [u8; 32] =
    hex!("0000000000000000000000000000000000000000000000000000000000000005");

// slot 6 → plugin address (20 B)
pub const SLOT_6: [u8; 32] =
    hex!("0000000000000000000000000000000000000000000000000000000000000006");

// slot 7 → communityVault address (20 B)
pub const SLOT_7: [u8; 32] =
    hex!("0000000000000000000000000000000000000000000000000000000000000007");

// slot 8 → tickTable mapping root
pub const TICK_TABLE_SLOT: [u8; 32] =
    hex!("0000000000000000000000000000000000000000000000000000000000000008");

// slot 9 → nextTickGlobal + prevTickGlobal + liquidity + tickSpacing + tickTreeRoot (packed 29 B)
pub const SLOT_9: [u8; 32] =
    hex!("0000000000000000000000000000000000000000000000000000000000000009");

// slot 12 → reserve0 + reserve1 (packed 32 B: two uint128)
pub const SLOT_12: [u8; 32] =
    hex!("000000000000000000000000000000000000000000000000000000000000000c");

// -------- SLOT 0 → Fee Growth Globals --------

/// `totalFeeGrowth0Token` — full uint256, entire slot 0.
/// Accumulated fee growth per unit of liquidity for token0 (Q128.128).
pub const TOTAL_FEE_GROWTH_0_SLOT: StorageLocation = StorageLocation {
    name: "total_fee_growth_0_token",
    slot: SLOT_0,
    offset: 0,
    number_of_bytes: 32,
    signed: false,
};

/// `totalFeeGrowth1Token` — full uint256, entire slot 1.
/// Accumulated fee growth per unit of liquidity for token1 (Q128.128).
pub const TOTAL_FEE_GROWTH_1_SLOT: StorageLocation = StorageLocation {
    name: "total_fee_growth_1_token",
    slot: SLOT_1,
    offset: 0,
    number_of_bytes: 32,
    signed: false,
};

// -------- SLOT 2 → GlobalState (packed 28 bytes, right-aligned) --------
// Layout (LSB → MSB):
//   [0..19]  price          uint160  20 B
//   [20..22] tick           int24     3 B
//   [23..24] lastFee        uint16    2 B
//   [25]     pluginConfig   uint8     1 B
//   [26..27] communityFee   uint16    2 B
//   [28]     unlocked       bool      1 B
//   [29..31] (unused)                 3 B

/// Current sqrt price as Q64.96 (uint160).
pub const SQRT_PRICE_X96_SLOT: StorageLocation = StorageLocation {
    name: "sqrt_price_x96",
    slot: SLOT_2,
    offset: 0,
    number_of_bytes: 20,
    signed: false,
};

/// Current active tick (int24).
pub const CURRENT_TICK_SLOT: StorageLocation =
    StorageLocation { name: "tick", slot: SLOT_2, offset: 20, number_of_bytes: 3, signed: true };

/// Most recent dynamic fee value (uint16, in hundredths of a bip).
pub const LAST_FEE_SLOT: StorageLocation = StorageLocation {
    name: "last_fee",
    slot: SLOT_2,
    offset: 23,
    number_of_bytes: 2,
    signed: false,
};

/// Plugin configuration flags bitmap (uint8).
pub const PLUGIN_CONFIG_SLOT: StorageLocation = StorageLocation {
    name: "plugin_config",
    slot: SLOT_2,
    offset: 25,
    number_of_bytes: 1,
    signed: false,
};

/// Community fee share in hundredths of a percent (uint16).
pub const COMMUNITY_FEE_SLOT: StorageLocation = StorageLocation {
    name: "community_fee",
    slot: SLOT_2,
    offset: 26,
    number_of_bytes: 2,
    signed: false,
};

/// Reentrancy lock flag (bool / uint8).
pub const UNLOCKED_SLOT: StorageLocation = StorageLocation {
    name: "unlocked",
    slot: SLOT_2,
    offset: 28,
    number_of_bytes: 1,
    signed: false,
};

// -------- SLOT 4 → Community Fees Pending (packed 30 bytes) --------
// Layout (LSB → MSB):
//   [0..12]  communityFeePending0      uint104  13 B
//   [13..25] communityFeePending1      uint104  13 B
//   [26..29] lastFeeTransferTimestamp  uint32    4 B

/// Accumulated community fee for token0 not yet transferred (uint104).
pub const COMMUNITY_FEE_PENDING_0_SLOT: StorageLocation = StorageLocation {
    name: "community_fee_pending_0",
    slot: SLOT_4,
    offset: 0,
    number_of_bytes: 13,
    signed: false,
};

/// Accumulated community fee for token1 not yet transferred (uint104).
pub const COMMUNITY_FEE_PENDING_1_SLOT: StorageLocation = StorageLocation {
    name: "community_fee_pending_1",
    slot: SLOT_4,
    offset: 13,
    number_of_bytes: 13,
    signed: false,
};

/// Timestamp of the last community-fee transfer (uint32).
pub const LAST_FEE_TRANSFER_TIMESTAMP_SLOT: StorageLocation = StorageLocation {
    name: "last_fee_transfer_timestamp",
    slot: SLOT_4,
    offset: 26,
    number_of_bytes: 4,
    signed: false,
};

// -------- SLOT 5 → Plugin Fees Pending (packed 26 bytes) --------
// Layout (LSB → MSB):
//   [0..12]  pluginFeePending0  uint104  13 B
//   [13..25] pluginFeePending1  uint104  13 B

/// Accumulated plugin fee for token0 (uint104).
pub const PLUGIN_FEE_PENDING_0_SLOT: StorageLocation = StorageLocation {
    name: "plugin_fee_pending_0",
    slot: SLOT_5,
    offset: 0,
    number_of_bytes: 13,
    signed: false,
};

/// Accumulated plugin fee for token1 (uint104).
pub const PLUGIN_FEE_PENDING_1_SLOT: StorageLocation = StorageLocation {
    name: "plugin_fee_pending_1",
    slot: SLOT_5,
    offset: 13,
    number_of_bytes: 13,
    signed: false,
};

// -------- SLOT 6 → Plugin Address --------

/// Plugin contract address (address / uint160, right-aligned in 32-byte slot).
pub const PLUGIN_ADDRESS_SLOT: StorageLocation = StorageLocation {
    name: "plugin",
    slot: SLOT_6,
    offset: 0,
    number_of_bytes: 20,
    signed: false,
};

// -------- SLOT 7 → Community Vault Address --------

/// communityVault address — receives community fees (address / uint160).
pub const COMMUNITY_VAULT_SLOT: StorageLocation = StorageLocation {
    name: "community_vault",
    slot: SLOT_7,
    offset: 0,
    number_of_bytes: 20,
    signed: false,
};

// -------- SLOT 9 → Packed liquidity / tick pointers (29 bytes) --------
// Layout (LSB → MSB):
//   [0..2]   nextTickGlobal  int24    3 B
//   [3..5]   prevTickGlobal  int24    3 B
//   [6..21]  liquidity       uint128 16 B
//   [22..24] tickSpacing     int24    3 B
//   [25..28] tickTreeRoot    uint32   4 B

/// Next initialized tick above the current tick (int24).
pub const NEXT_TICK_GLOBAL_SLOT: StorageLocation = StorageLocation {
    name: "next_tick_global",
    slot: SLOT_9,
    offset: 0,
    number_of_bytes: 3,
    signed: true,
};

/// Previous initialized tick at or below the current tick (int24).
pub const PREV_TICK_GLOBAL_SLOT: StorageLocation = StorageLocation {
    name: "prev_tick_global",
    slot: SLOT_9,
    offset: 3,
    number_of_bytes: 3,
    signed: true,
};

/// Current in-range liquidity (uint128).
pub const LIQUIDITY_SLOT: StorageLocation = StorageLocation {
    name: "liquidity",
    slot: SLOT_9,
    offset: 6,
    number_of_bytes: 16,
    signed: false,
};

/// Tick spacing — minimum granularity of ticks (int24).
pub const TICK_SPACING_SLOT: StorageLocation = StorageLocation {
    name: "tick_spacing",
    slot: SLOT_9,
    offset: 22,
    number_of_bytes: 3,
    signed: true,
};

/// Root word of the packed binary tick tree used for O(log n) tick search (uint32).
/// Packed in the same slot as tickSpacing (slot 9).
pub const TICK_TREE_ROOT_SLOT: StorageLocation = StorageLocation {
    name: "tick_tree_root",
    slot: SLOT_9,
    offset: 25,
    number_of_bytes: 4,
    signed: false,
};

// -------- SLOT 12 → Reserves (packed 32 bytes: two uint128) --------
// Layout (LSB → MSB):
//   [0..15]  reserve0  uint128  16 B
//   [16..31] reserve1  uint128  16 B

/// Pool's internal tracked balance of token0 (uint128).
pub const RESERVE_0_SLOT: StorageLocation = StorageLocation {
    name: "reserve0",
    slot: SLOT_12,
    offset: 0,
    number_of_bytes: 16,
    signed: false,
};

/// Pool's internal tracked balance of token1 (uint128).
pub const RESERVE_1_SLOT: StorageLocation = StorageLocation {
    name: "reserve1",
    slot: SLOT_12,
    offset: 16,
    number_of_bytes: 16,
    signed: false,
};

// -------- TRACKED SLOTS --------
// All variables that can change during a Swap, Mint, Burn, or Initialize event.
// These are passed to `get_changed_attributes()` to produce on-chain state deltas.
//
// Slots touched per operation:
//   Swap:       0, 1, 2, 4, 5, 6(read), 7(read), 9, 12
//   Mint/Burn:  2, 3(ticks — handled separately), 9, 12
//   Initialize: 2, 9
//
// We track all writable slots here; read-only addresses (plugin/communityVault)
// are included because they can be updated by governance and we need their
// current values in the DB for the simulation engine.

pub const TRACKED_SLOTS: [StorageLocation; 22] = [
    // Slot 0 — fee growth globals (updated every swap)
    TOTAL_FEE_GROWTH_0_SLOT,
    TOTAL_FEE_GROWTH_1_SLOT,
    // Slot 2 — globalState fields (core swap state)
    SQRT_PRICE_X96_SLOT,
    CURRENT_TICK_SLOT,
    LAST_FEE_SLOT,
    PLUGIN_CONFIG_SLOT,
    COMMUNITY_FEE_SLOT,
    UNLOCKED_SLOT,
    // Slot 4 — community fee accrual (updated every swap via _changeReserves)
    COMMUNITY_FEE_PENDING_0_SLOT,
    COMMUNITY_FEE_PENDING_1_SLOT,
    LAST_FEE_TRANSFER_TIMESTAMP_SLOT,
    // Slot 5 — plugin fee accrual (updated every swap via _changeReserves)
    PLUGIN_FEE_PENDING_0_SLOT,
    PLUGIN_FEE_PENDING_1_SLOT,
    // Slot 6 & 7 — addresses (rarely change but needed for simulation)
    PLUGIN_ADDRESS_SLOT,
    COMMUNITY_VAULT_SLOT,
    // Slot 9 — hot liquidity + tick pointer slot (updated on tick cross).
    // tickTreeRoot shares this slot (offset 25) and is updated on every tick add/remove.
    NEXT_TICK_GLOBAL_SLOT,
    PREV_TICK_GLOBAL_SLOT,
    LIQUIDITY_SLOT,
    TICK_SPACING_SLOT,
    TICK_TREE_ROOT_SLOT,
    // Slot 12 — reserves (updated every swap)
    RESERVE_0_SLOT,
    RESERVE_1_SLOT,
];
