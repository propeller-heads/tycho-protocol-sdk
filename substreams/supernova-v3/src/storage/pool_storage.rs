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
                        // Serialise round-trip: signed → signed bytes (preserves
                        // sign extension); unsigned → big-endian magnitude bytes
                        // WITHOUT the leading 0x00 sign byte that
                        // `to_signed_bytes_be()` would prepend for values whose
                        // high bit is set. This matters for fields like
                        // `sqrt_price_x96` and `liquidity` that can legitimately
                        // exceed 2^159 / 2^127, where the wrong serialisation
                        // would change the on-wire byte length and confuse any
                        // consumer that decodes with `from_unsigned_bytes_be`.
                        let value_bytes: Vec<u8> = if storage_location.signed {
                            BigInt::from_signed_bytes_be(new_data).to_signed_bytes_be()
                        } else {
                            // `to_bytes_be()` returns `(Sign, Vec<u8>)`. For an
                            // unsigned value built from raw bytes, the sign is
                            // always Plus (or NoSign for zero), so we just take
                            // the magnitude. We pad with a leading zero only if
                            // the original buffer was empty (defensive).
                            let bi = BigInt::from_unsigned_bytes_be(new_data);
                            let (_, mag) = bi.to_bytes_be();
                            if mag.is_empty() { vec![0u8] } else { mag }
                        };
                        attributes.push(Attribute {
                            name: storage_location.name.to_string(),
                            value: value_bytes,
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

/// Decode the packed `(reserve0, reserve1)` pair from Algebra Integral
/// pool slot 12 (`_reserves`).
///
/// # Layout
///
/// `_reserves` is two `uint128` fields packed LSB-first by the Solidity
/// compiler:
///
/// ```text
///     uint128 reserve0;   // bits   0..127  (LSB)
///     uint128 reserve1;   // bits 128..255  (MSB)
/// ```
///
/// When the slot is read MSB-first (as EVM storage values are typically
/// returned and stored), the byte layout becomes:
///
/// ```text
///     [ 0..16)  reserve1  (upper 128 bits — left half of the slot)
///     [16..32)  reserve0  (lower 128 bits — right half of the slot)
/// ```
///
/// This is the **single source of truth** for slot-12 decoding. The
/// protocol-testing harness's `tycho_rpc.rs::get_snapshots` mirrors the
/// same layout in its reserve-reconciliation overlay; both must be
/// updated together if Algebra ever reorders the fields. The pinned
/// unit test below (`decode_slot12_reserves_unpacks_lsb_first`) is the
/// regression guard.
#[allow(dead_code)] // referenced by cross-crate consumers (protocol-testing harness) and pinned by the unit test below
pub fn decode_slot12_reserves(slot12_value: &[u8; 32]) -> (substreams::scalar::BigInt, substreams::scalar::BigInt) {
    let reserve1 = substreams::scalar::BigInt::from_unsigned_bytes_be(&slot12_value[0..16]);
    let reserve0 = substreams::scalar::BigInt::from_unsigned_bytes_be(&slot12_value[16..32]);
    (reserve0, reserve1)
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
    // We never expect to overflow byte 0 of a keccak-derived slot in
    // practice (probability ~2^-256), but assert it under debug builds so
    // a future bug in calc_map_slot doesn't silently corrupt addressing.
    debug_assert_eq!(carry, 0, "add_slot_offset overflow past slot byte 0");
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use substreams_ethereum::pb::eth::v2::StorageChange;

    /// Regression test for the unsigned/signed serialisation bug
    /// (review finding C2). When an unsigned field has its high bit set
    /// — e.g. a 16-byte uint128 starting with 0xff — the previous
    /// implementation called `to_signed_bytes_be()` after constructing
    /// the BigInt with `from_unsigned_bytes_be()`, which prepended a
    /// 0x00 sign byte and changed the on-wire byte length from 16 to 17.
    ///
    /// This test pins the new behaviour: an unsigned field is serialised
    /// as the magnitude bytes only, with the original byte length preserved.
    #[test]
    fn unsigned_field_high_bit_set_serialises_without_sign_pad() {
        // 32-byte slot whose lower 16 bytes are 0xffff…ff (max uint128).
        let mut new_value = vec![0u8; 16];
        new_value.extend(std::iter::repeat(0xffu8).take(16));
        let old_value = vec![0u8; 32];

        let location_name: &str = "test_uint128";
        let location_slot: [u8; 32] = [0u8; 32]; // slot 0
        let storage_change = StorageChange {
            address: vec![],
            key: location_slot.to_vec(),
            old_value,
            new_value: new_value.clone(),
            ordinal: 0,
        };
        let pool_storage = UniswapPoolStorage::new(std::slice::from_ref(&storage_change));
        let location = StorageLocation {
            name: location_name,
            slot: location_slot,
            offset: 0,
            number_of_bytes: 16,
            signed: false,
        };
        let attrs = pool_storage.get_changed_attributes(vec![&location]);
        assert_eq!(attrs.len(), 1);
        // 16 bytes of magnitude — NOT 17 (no leading 0x00 sign byte).
        assert_eq!(
            attrs[0].value.len(),
            16,
            "unsigned uint128 must serialise to exactly 16 bytes (no sign-pad)"
        );
        assert!(
            attrs[0].value.iter().all(|b| *b == 0xff),
            "value bytes should be all 0xff for max uint128"
        );
    }

    /// Pin the slot-12 reserve byte layout. If Algebra ever reorders
    /// `reserve0`/`reserve1` in `ReservesManager`, this test must fail
    /// loudly so we update both the substream's balance emission and
    /// the harness's reconciliation in lockstep.
    #[test]
    fn decode_slot12_reserves_unpacks_lsb_first() {
        // Construct a slot value where reserve0 = 0x010203... (lower 16 bytes)
        // and reserve1 = 0xa1b2c3... (upper 16 bytes).
        let mut slot = [0u8; 32];
        // reserve1 = 0xa1...b0 in the upper half (slot bytes 0..16)
        for (i, b) in slot.iter_mut().enumerate().take(16) {
            *b = 0xa0 + i as u8;
        }
        // reserve0 = 0x01...10 in the lower half (slot bytes 16..32)
        for (i, b) in slot.iter_mut().enumerate().skip(16) {
            *b = 0x01 + (i - 16) as u8;
        }
        let (r0, r1) = decode_slot12_reserves(&slot);

        // reserve0 was written into bytes 16..32 = [0x01, 0x02, ..., 0x10].
        let expected_r0 = BigInt::from_unsigned_bytes_be(&slot[16..32]);
        // reserve1 was written into bytes 0..16 = [0xa0, 0xa1, ..., 0xaf].
        let expected_r1 = BigInt::from_unsigned_bytes_be(&slot[0..16]);

        assert_eq!(r0, expected_r0, "reserve0 must come from slot bytes 16..32 (lower 128 bits)");
        assert_eq!(r1, expected_r1, "reserve1 must come from slot bytes 0..16 (upper 128 bits)");
        // Sanity: the two halves are not equal (so a swapped helper would fail).
        assert_ne!(r0, r1);
    }

    /// Signed fields still round-trip correctly. A negative int24
    /// (e.g. tick = -2) must be sign-extended on the way back out.
    #[test]
    fn signed_field_negative_value_round_trips() {
        // int24 = -2 → big-endian: 0xffffe (3 bytes). Place at slot offset 20
        // (where Algebra packs `tick`).
        let mut new_value = vec![0u8; 32];
        // Slot byte indices [9..12) hold the int24 tick (= LSB offset 20..23).
        new_value[9] = 0xff;
        new_value[10] = 0xff;
        new_value[11] = 0xfe;
        let storage_change = StorageChange {
            address: vec![],
            key: [0u8; 32].to_vec(),
            old_value: vec![0u8; 32],
            new_value,
            ordinal: 0,
        };
        let pool_storage = UniswapPoolStorage::new(std::slice::from_ref(&storage_change));
        let location = StorageLocation {
            name: "tick",
            slot: [0u8; 32],
            offset: 20,
            number_of_bytes: 3,
            signed: true,
        };
        let attrs = pool_storage.get_changed_attributes(vec![&location]);
        assert_eq!(attrs.len(), 1);
        let decoded = BigInt::from_signed_bytes_be(&attrs[0].value);
        assert_eq!(decoded, BigInt::from(-2));
    }
}
