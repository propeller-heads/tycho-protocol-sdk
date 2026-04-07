use crate::storage::utils;
use tycho_substreams::prelude::{Attribute, ChangeType};

use substreams::scalar::BigInt;
use substreams_ethereum::pb::eth::v2::StorageChange;

use super::{
    constants::{
        TICKS_MAP_SLOT,
        SLOT_0, SLOT_1, SLOT_2, SLOT_4, SLOT_5, SLOT_6, SLOT_7, SLOT_9, SLOT_12,
    },
    utils::read_bytes,
};

/// `StorageLocation` describes a sub-field inside a 32-byte EVM storage slot.
///
/// Offsets are **from the right** (least-significant byte = offset 0), matching
/// Solidity's right-aligned, little-endian-within-slot packing convention.
///
/// # Fields
/// * `name`            — unique attribute key written to the Tycho DB
/// * `slot`            — 32-byte storage slot number
/// * `offset`          — byte offset from the rightmost byte of the slot
/// * `number_of_bytes` — width of the field in bytes
/// * `signed`          — if true, decode as a signed (two's complement) integer
#[derive(Clone)]
pub struct StorageLocation<'a> {
    pub name: &'a str,
    pub slot: [u8; 32],
    pub offset: usize,
    pub number_of_bytes: usize,
    pub signed: bool,
}

pub struct UniswapPoolStorage<'a> {
    pub storage_changes: &'a [StorageChange],
}

impl<'a> UniswapPoolStorage<'a> {
    pub fn new(storage_changes: &'a [StorageChange]) -> UniswapPoolStorage<'a> {
        Self { storage_changes }
    }

    /// Iterates through storage changes and checks for modifications in the provided list of
    /// storage locations. For each change, it compares the old and new values at the specified
    /// offset and length for that location. If a change is detected, it's added to the returned
    /// `Attribute` list.
    ///
    /// Arguments:
    ///     locations: Vec<&StorageLocation> - A vector of references to StorageLocation objects
    /// that define the slots, offsets, and lengths to be checked for changes.
    ///
    /// Returns:
    ///     `Vec<Attribute>`: A vector containing Attributes for each change detected in the tracked
    /// slots. Returns an empty vector if no changes are detected.
    pub fn get_changed_attributes(&self, locations: Vec<&StorageLocation>) -> Vec<Attribute> {
        let mut attributes = Vec::new();

        // For each storage change, check if it changes a tracked slot.
        // If it does, add the attribute to the list of attributes
        for change in self.storage_changes {
            for storage_location in locations.iter() {
                // Check if the change slot matches the tracked slot
                if change.key == storage_location.slot {
                    let old_data = read_bytes(
                        &change.old_value,
                        storage_location.offset,
                        storage_location.number_of_bytes,
                    );
                    let new_data = read_bytes(
                        &change.new_value,
                        storage_location.offset,
                        storage_location.number_of_bytes,
                    );

                    // Check if there is a change in the data
                    if old_data != new_data {
                        let value = match storage_location.signed {
                            true => BigInt::from_signed_bytes_be(new_data),
                            false => BigInt::from_unsigned_bytes_be(new_data),
                        };
                        attributes.push(Attribute {
                            name: storage_location.name.to_string(),
                            value: value.to_signed_bytes_be(),
                            change: ChangeType::Update.into(),
                        });
                    }
                }
            }
        }

        attributes
    }

    /// Returns the **full 32-byte raw value** of each swap-critical slot whenever
    /// that slot appears in the storage changes.  This is used on pool creation /
    /// first-seen to snapshot the complete slot state into the Tycho DB so that the
    /// VM simulation engine can reconstruct the contract storage without an RPC call.
    ///
    /// Slots tracked (all touched during a swap):
    ///   0  — totalFeeGrowth0Token
    ///   1  — totalFeeGrowth1Token
    ///   2  — globalState (price, tick, lastFee, pluginConfig, communityFee, unlocked)
    ///   4  — communityFeePending0/1 + lastFeeTransferTimestamp
    ///   5  — pluginFeePending0/1
    ///   6  — plugin address
    ///   7  — communityVault address
    ///   9  — nextTickGlobal, prevTickGlobal, liquidity, tickSpacing, tickTreeRoot
    ///   12 — reserve0, reserve1
    pub fn get_full_slot_changes(&self) -> Vec<Attribute> {
        // The canonical list of whole-slot attribute names, in slot order.
        const WHOLE_SLOTS: [([u8; 32], &str); 9] = [
            (SLOT_0,  "slot_0"),
            (SLOT_1,  "slot_1"),
            (SLOT_2,  "slot_2"),
            (SLOT_4,  "slot_4"),
            (SLOT_5,  "slot_5"),
            (SLOT_6,  "slot_6"),
            (SLOT_7,  "slot_7"),
            (SLOT_9,  "slot_9"),
            (SLOT_12, "slot_12"),
        ];

        let mut attributes = Vec::new();

        for change in self.storage_changes {
            for (slot, name) in &WHOLE_SLOTS {
                if change.key == *slot && change.old_value != change.new_value {
                    attributes.push(Attribute {
                        name: name.to_string(),
                        value: change.new_value.clone(),
                        change: ChangeType::Update.into(),
                    });
                }
            }
        }

        attributes
    }

    /// Iterates over a list of tick indexes and checks for modifications in the list of
    /// storage changes. If a relevant change is detected, it's added to the returned `Attribute`
    /// list.
    ///
    /// Arguments:
    ///     ticks_idx: `Vec<&BigInt>` - A vector of references to tick indexes as BigInt objects.
    ///
    /// Returns:
    ///     `Vec<Attribute>`: A vector containing Attributes for each change detected. Returns an
    /// empty vector if no changes are detected.
    ///
    /// # Tick struct layout (Algebra supernova-main, libraries/TickManagement.sol)
    ///
    /// ```
    /// struct Tick {
    ///   uint256 liquidityTotal;         // slot+0  (32 B)
    ///   int128  liquidityDelta;         // slot+1  offset  0 (16 B)
    ///   int24   prevTick;               // slot+1  offset 16 (3 B)
    ///   int24   nextTick;               // slot+1  offset 19 (3 B)
    ///   uint256 outerFeeGrowth0Token;   // slot+2  (32 B)
    ///   uint256 outerFeeGrowth1Token;   // slot+3  (32 B)
    /// }
    /// ```
    ///
    /// We track `liquidityDelta` (int128, offset 0, 16 B at slot+1) which is the
    /// quantity the simulation engine needs for price-impact calculations.
    pub fn get_ticks_changes(&self, ticks_idx: Vec<&BigInt>) -> Vec<Attribute> {
        let mut storage_locs = Vec::new();
        let mut tick_names = Vec::new();

        // First, create all the names and push them into tick_names.
        // We need this to keep the references to the names alive until we call
        // `get_changed_attributes()`
        for tick_idx in ticks_idx.iter() {
            tick_names.push(format!("ticks/{tick_idx}/net-liquidity"));
        }

        // Then, iterate over ticks_idx and tick_names simultaneously
        for (tick_idx, tick_name) in ticks_idx.iter().zip(tick_names.iter()) {
            // The mapping base slot for `ticks` is slot 3.
            // Each Tick struct occupies 4 sequential slots per entry.
            // Layout: slot+0 liquidityTotal, slot+1 liquidityDelta|prevTick|nextTick,
            //         slot+2 outerFeeGrowth0Token, slot+3 outerFeeGrowth1Token.
            let tick_base_slot =
                utils::calc_map_slot(&utils::left_pad_from_bigint(tick_idx), &TICKS_MAP_SLOT);

            // liquidityDelta is at tick_base_slot + 1, offset 0, 16 bytes (int128)
            let liquidity_delta_slot = add_slot_offset(&tick_base_slot, 1);

            storage_locs.push(StorageLocation {
                name: tick_name,
                slot: liquidity_delta_slot,
                offset: 0,
                number_of_bytes: 16,
                signed: true,
            });
        }

        self.get_changed_attributes(storage_locs.iter().collect())
    }
}

/// Compute `slot + offset` for sequential struct sub-slots.
/// Algebra tick data spans 4 consecutive slots per tick entry; this lets us
/// address `liquidityDelta` (sub-slot index 2 of the mapping entry).
fn add_slot_offset(base: &[u8; 32], offset: u8) -> [u8; 32] {
    let mut result = *base;
    let mut carry = offset as u16;
    for i in (0..32).rev() {
        let sum = result[i] as u16 + carry;
        result[i] = sum as u8;
        carry = sum >> 8;
        if carry == 0 {
            break;
        }
    }
    result
}
