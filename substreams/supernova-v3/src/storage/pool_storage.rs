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

    // ────────────────────────────────────────────────────────────────
    //                    Test helpers
    // ────────────────────────────────────────────────────────────────

    /// Build a `StorageChange` for slot 0 with given old/new bytes.
    fn make_change(old: [u8; 32], new: [u8; 32]) -> StorageChange {
        StorageChange {
            address: vec![],
            key: [0u8; 32].to_vec(),
            old_value: old.to_vec(),
            new_value: new.to_vec(),
            ordinal: 0,
        }
    }

    /// A 32-byte slot key for an integer slot index (e.g. slot 2 → 0x..02).
    fn slot_key(idx: u8) -> [u8; 32] {
        let mut k = [0u8; 32];
        k[31] = idx;
        k
    }

    // ────────────────────────────────────────────────────────────────
    //                  get_changed_attributes
    // ────────────────────────────────────────────────────────────────

    /// When `old_value == new_value`, no attribute should be emitted —
    /// even if the storage key matches a tracked location.
    #[test]
    fn no_emission_when_old_equals_new() {
        let v = [0xaau8; 32];
        let change = make_change(v, v);
        let pool_storage = UniswapPoolStorage::new(std::slice::from_ref(&change));
        let loc = StorageLocation {
            name: "noop",
            slot: [0u8; 32],
            offset: 0,
            number_of_bytes: 32,
            signed: false,
        };
        let attrs = pool_storage.get_changed_attributes(vec![&loc]);
        assert!(attrs.is_empty(), "no attribute should be emitted when slot bytes are unchanged");
    }

    /// Sub-field comparison must work correctly: if the byte change is
    /// outside the field's offset/length window, no attribute is emitted
    /// even though the slot itself changed.
    #[test]
    fn no_emission_when_change_falls_outside_field_window() {
        let mut old_v = [0u8; 32];
        let mut new_v = [0u8; 32];
        // Change byte at LSB offset 15 (slot byte index 16). The field we
        // care about lives at LSB offset 0..3 (slot byte indices 29..32),
        // so this change must NOT trigger an attribute emission.
        old_v[16] = 0x00;
        new_v[16] = 0xff;

        let change = make_change(old_v, new_v);
        let pool_storage = UniswapPoolStorage::new(std::slice::from_ref(&change));
        let loc = StorageLocation {
            name: "low_3_bytes",
            slot: [0u8; 32],
            offset: 0,
            number_of_bytes: 3,
            signed: false,
        };
        let attrs = pool_storage.get_changed_attributes(vec![&loc]);
        assert!(attrs.is_empty(), "field-window read must filter out unrelated slot byte changes");
    }

    /// One slot, multiple sub-field locations: only the field whose bytes
    /// actually changed should be emitted. This is exactly how
    /// `slot 2` (globalState) works during a swap — only `sqrt_price_x96`
    /// and `tick` move, while `community_fee` and `unlocked` stay put.
    #[test]
    fn multi_field_slot_only_changed_field_emitted() {
        // Construct an old/new slot 2 where only the `sqrt_price_x96`
        // bytes (LSB offset 0..20) change. `tick` and the other fields
        // stay constant.
        let mut old_v = [0u8; 32];
        let mut new_v = [0u8; 32];
        // sqrt_price_x96 at LSB offset 0..20 → MSB byte indices 12..32
        for b in old_v.iter_mut().skip(12) {
            *b = 0x11;
        }
        for b in new_v.iter_mut().skip(12) {
            *b = 0x22;
        }
        // tick at LSB offset 20..23 → MSB byte indices 9..12. Same in both.
        // (already 0 in both)

        let change = make_change(old_v, new_v);
        let pool_storage = UniswapPoolStorage::new(std::slice::from_ref(&change));
        let sqrt_price = StorageLocation {
            name: "sqrt_price_x96",
            slot: [0u8; 32],
            offset: 0,
            number_of_bytes: 20,
            signed: false,
        };
        let tick = StorageLocation {
            name: "tick",
            slot: [0u8; 32],
            offset: 20,
            number_of_bytes: 3,
            signed: true,
        };
        let attrs = pool_storage.get_changed_attributes(vec![&sqrt_price, &tick]);
        assert_eq!(attrs.len(), 1, "only sqrt_price_x96 should be emitted");
        assert_eq!(attrs[0].name, "sqrt_price_x96");
    }

    /// Multiple slots changing in one call: every matching location
    /// should be emitted, in the order they were checked. (We don't
    /// guarantee any particular ordering across slots, but every
    /// changed field MUST be present.)
    #[test]
    fn multi_slot_changes_all_emitted() {
        let s0_old = [0u8; 32];
        let s0_new = {
            let mut v = [0u8; 32];
            v[31] = 0x42;
            v
        };
        let s1_old = [0u8; 32];
        let s1_new = {
            let mut v = [0u8; 32];
            v[31] = 0x99;
            v
        };
        let c0 = StorageChange {
            address: vec![],
            key: slot_key(0).to_vec(),
            old_value: s0_old.to_vec(),
            new_value: s0_new.to_vec(),
            ordinal: 0,
        };
        let c1 = StorageChange {
            address: vec![],
            key: slot_key(1).to_vec(),
            old_value: s1_old.to_vec(),
            new_value: s1_new.to_vec(),
            ordinal: 1,
        };
        let changes = [c0, c1];
        let pool_storage = UniswapPoolStorage::new(&changes);
        let l0 = StorageLocation {
            name: "field_0",
            slot: slot_key(0),
            offset: 0,
            number_of_bytes: 1,
            signed: false,
        };
        let l1 = StorageLocation {
            name: "field_1",
            slot: slot_key(1),
            offset: 0,
            number_of_bytes: 1,
            signed: false,
        };
        let attrs = pool_storage.get_changed_attributes(vec![&l0, &l1]);
        assert_eq!(attrs.len(), 2);
        let names: Vec<_> = attrs.iter().map(|a| a.name.as_str()).collect();
        assert!(names.contains(&"field_0"));
        assert!(names.contains(&"field_1"));
    }

    /// A field that goes from a positive value to zero is still a
    /// change and must emit. Edge case: serialised zero must be a
    /// non-empty single zero byte (BigInt zero is special).
    #[test]
    fn zero_value_emits_as_single_zero_byte() {
        let mut old_v = [0u8; 32];
        old_v[31] = 0xff; // old field value = 0xff
        let new_v = [0u8; 32]; // new field value = 0
        let change = make_change(old_v, new_v);
        let pool_storage = UniswapPoolStorage::new(std::slice::from_ref(&change));
        let loc = StorageLocation {
            name: "u8_field",
            slot: [0u8; 32],
            offset: 0,
            number_of_bytes: 1,
            signed: false,
        };
        let attrs = pool_storage.get_changed_attributes(vec![&loc]);
        assert_eq!(attrs.len(), 1);
        assert_eq!(
            attrs[0].value,
            vec![0u8],
            "zero must serialise to a single 0x00 byte (not empty)"
        );
    }

    /// `liquidity` (uint128) at LSB offset 6, length 16, in slot 9.
    /// This pins the exact byte arithmetic the substream uses for one
    /// of the most important runtime fields.
    #[test]
    fn slot9_liquidity_uint128_decoded_at_correct_offset() {
        // Place liquidity = 0x0102030405060708090a0b0c0d0e0f10 at the
        // correct LSB offset 6 (slot byte indices 10..26).
        let mut new_v = [0u8; 32];
        let liq_bytes: [u8; 16] = [
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
            0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10,
        ];
        new_v[10..26].copy_from_slice(&liq_bytes);

        let change = make_change([0u8; 32], new_v);
        let pool_storage = UniswapPoolStorage::new(std::slice::from_ref(&change));
        let loc = StorageLocation {
            name: "liquidity",
            slot: [0u8; 32],
            offset: 6,
            number_of_bytes: 16,
            signed: false,
        };
        let attrs = pool_storage.get_changed_attributes(vec![&loc]);
        assert_eq!(attrs.len(), 1);
        // Reconstruct the BigInt and verify it equals what we packed.
        let expected = BigInt::from_unsigned_bytes_be(&liq_bytes);
        let decoded = BigInt::from_unsigned_bytes_be(&attrs[0].value);
        assert_eq!(decoded, expected);
    }

    /// Most negative int24 (-2^23 = -8388608 = 0x800000) must round-trip
    /// correctly through the signed serialisation path.
    #[test]
    fn int24_min_value_round_trips() {
        let mut new_v = [0u8; 32];
        // int24 min = -8388608 → 0x800000 → at LSB offset 0, slot bytes 29..32
        new_v[29] = 0x80;
        new_v[30] = 0x00;
        new_v[31] = 0x00;
        let change = make_change([0u8; 32], new_v);
        let pool_storage = UniswapPoolStorage::new(std::slice::from_ref(&change));
        let loc = StorageLocation {
            name: "int24_min",
            slot: [0u8; 32],
            offset: 0,
            number_of_bytes: 3,
            signed: true,
        };
        let attrs = pool_storage.get_changed_attributes(vec![&loc]);
        assert_eq!(attrs.len(), 1);
        let decoded = BigInt::from_signed_bytes_be(&attrs[0].value);
        assert_eq!(decoded, BigInt::from(-8_388_608i32));
    }

    /// Most positive int24 (+2^23 - 1 = 8388607 = 0x7fffff).
    #[test]
    fn int24_max_value_round_trips() {
        let mut new_v = [0u8; 32];
        new_v[29] = 0x7f;
        new_v[30] = 0xff;
        new_v[31] = 0xff;
        let change = make_change([0u8; 32], new_v);
        let pool_storage = UniswapPoolStorage::new(std::slice::from_ref(&change));
        let loc = StorageLocation {
            name: "int24_max",
            slot: [0u8; 32],
            offset: 0,
            number_of_bytes: 3,
            signed: true,
        };
        let attrs = pool_storage.get_changed_attributes(vec![&loc]);
        assert_eq!(attrs.len(), 1);
        let decoded = BigInt::from_signed_bytes_be(&attrs[0].value);
        assert_eq!(decoded, BigInt::from(8_388_607i32));
    }

    /// `sqrt_price_x96` is uint160 (20 bytes). When the high bit is
    /// set (price near MAX_SQRT_RATIO), the unsigned serialisation
    /// must NOT be misinterpreted as negative. This is the exact
    /// scenario the C2 fix protects against for the `sqrt_price_x96`
    /// field on a pool whose price has crossed `2^159`.
    #[test]
    fn sqrt_price_x96_high_bit_set_decodes_as_positive() {
        let mut high_bit_value = [0u8; 32];
        // 20 bytes of high-bit-set value: 0xff..ff at LSB offset 0..20
        for b in high_bit_value.iter_mut().skip(12) {
            *b = 0xff;
        }
        let change = make_change([0u8; 32], high_bit_value);
        let pool_storage = UniswapPoolStorage::new(std::slice::from_ref(&change));
        let loc = StorageLocation {
            name: "sqrt_price_x96",
            slot: [0u8; 32],
            offset: 0,
            number_of_bytes: 20,
            signed: false,
        };
        let attrs = pool_storage.get_changed_attributes(vec![&loc]);
        assert_eq!(attrs.len(), 1);
        // Must serialise to exactly 20 bytes (no sign-pad).
        assert_eq!(attrs[0].value.len(), 20);
        let decoded = BigInt::from_unsigned_bytes_be(&attrs[0].value);
        // Round-trip: 20 bytes of 0xff = 2^160 - 1
        let expected = (BigInt::from(1) << 160) - 1;
        assert_eq!(decoded, expected);
    }

    /// Empty `locations` vector should produce empty attributes.
    #[test]
    fn empty_locations_yields_empty_attrs() {
        let change = make_change([0u8; 32], [1u8; 32]);
        let pool_storage = UniswapPoolStorage::new(std::slice::from_ref(&change));
        let attrs = pool_storage.get_changed_attributes(vec![]);
        assert!(attrs.is_empty());
    }

    /// Empty `storage_changes` slice should produce empty attributes
    /// regardless of how many locations are passed.
    #[test]
    fn empty_storage_changes_yields_empty_attrs() {
        let pool_storage = UniswapPoolStorage::new(&[]);
        let loc = StorageLocation {
            name: "anything",
            slot: [0u8; 32],
            offset: 0,
            number_of_bytes: 1,
            signed: false,
        };
        let attrs = pool_storage.get_changed_attributes(vec![&loc]);
        assert!(attrs.is_empty());
    }

    /// A `StorageChange` whose key does NOT match the location's slot
    /// must be ignored, even if its bytes look interesting.
    #[test]
    fn non_matching_slot_key_ignored() {
        let change = StorageChange {
            address: vec![],
            key: slot_key(5).to_vec(), // slot 5
            old_value: vec![0u8; 32],
            new_value: vec![0xffu8; 32],
            ordinal: 0,
        };
        let pool_storage = UniswapPoolStorage::new(std::slice::from_ref(&change));
        let loc = StorageLocation {
            name: "looking_at_slot_2",
            slot: slot_key(2), // ← slot 2, not 5
            offset: 0,
            number_of_bytes: 1,
            signed: false,
        };
        let attrs = pool_storage.get_changed_attributes(vec![&loc]);
        assert!(attrs.is_empty());
    }

    // ────────────────────────────────────────────────────────────────
    //                  decode_slot12_reserves
    // ────────────────────────────────────────────────────────────────

    /// Both reserves zero → both BigInts must be zero (not panic).
    #[test]
    fn decode_slot12_reserves_zero() {
        let slot = [0u8; 32];
        let (r0, r1) = decode_slot12_reserves(&slot);
        assert_eq!(r0, BigInt::from(0));
        assert_eq!(r1, BigInt::from(0));
    }

    /// Only reserve0 set, reserve1 = 0. Make sure we don't accidentally
    /// read the wrong half.
    #[test]
    fn decode_slot12_reserves_only_reserve0() {
        let mut slot = [0u8; 32];
        // reserve0 lives in bytes 16..32 (lower half).
        slot[31] = 0x42;
        let (r0, r1) = decode_slot12_reserves(&slot);
        assert_eq!(r0, BigInt::from(0x42));
        assert_eq!(r1, BigInt::from(0));
    }

    /// Only reserve1 set, reserve0 = 0. Mirror of the test above.
    #[test]
    fn decode_slot12_reserves_only_reserve1() {
        let mut slot = [0u8; 32];
        // reserve1 lives in bytes 0..16 (upper half).
        slot[15] = 0x99;
        let (r0, r1) = decode_slot12_reserves(&slot);
        assert_eq!(r0, BigInt::from(0));
        assert_eq!(r1, BigInt::from(0x99));
    }

    /// Both reserves at uint128 max — proves there's no overflow or
    /// sign-extension surprise inside the helper.
    #[test]
    fn decode_slot12_reserves_both_uint128_max() {
        let slot = [0xffu8; 32];
        let (r0, r1) = decode_slot12_reserves(&slot);
        let max_u128 = (BigInt::from(1) << 128) - 1;
        assert_eq!(r0, max_u128);
        assert_eq!(r1, max_u128);
    }

    /// Real-shaped values inspired by an actual USDC/USDT pool: a few
    /// hundred billion of each token (≈ $1M of liquidity at 1e6 decimals).
    #[test]
    fn decode_slot12_reserves_realistic_stable_pool() {
        // reserve0 = 121_127_485_284 (about 121k USDC at 6 decimals)
        // reserve1 = 801_049_306_036 (about 801k USDT at 6 decimals)
        let r0_target: u64 = 121_127_485_284;
        let r1_target: u64 = 801_049_306_036;

        let mut slot = [0u8; 32];
        // Pack each u64 into the lower 8 bytes of its 16-byte uint128 region.
        // reserve0 lives at slot[16..32]; its u64 LSBs are at slot[24..32].
        slot[24..32].copy_from_slice(&r0_target.to_be_bytes());
        // reserve1 lives at slot[0..16]; its u64 LSBs are at slot[8..16].
        slot[8..16].copy_from_slice(&r1_target.to_be_bytes());

        let (r0, r1) = decode_slot12_reserves(&slot);
        assert_eq!(r0, BigInt::from(r0_target));
        assert_eq!(r1, BigInt::from(r1_target));
    }

    // ────────────────────────────────────────────────────────────────
    //                  add_slot_offset
    // ────────────────────────────────────────────────────────────────

    /// Offset 0 = no-op. The base slot must come back unchanged.
    #[test]
    fn add_slot_offset_zero_is_identity() {
        let base = [0xabu8; 32];
        assert_eq!(add_slot_offset(&base, 0), base);
    }

    /// Offset 1 on a base whose last byte is < 0xff: only the last
    /// byte changes, no carry propagates.
    #[test]
    fn add_slot_offset_one_no_carry() {
        let mut base = [0u8; 32];
        base[31] = 0x10;
        let result = add_slot_offset(&base, 1);
        let mut expected = [0u8; 32];
        expected[31] = 0x11;
        assert_eq!(result, expected);
    }

    /// Offset 1 on a base whose last byte is 0xff: a single carry
    /// must propagate to byte 30, and the previous byte must increment.
    #[test]
    fn add_slot_offset_one_with_carry() {
        let mut base = [0u8; 32];
        base[31] = 0xff;
        base[30] = 0x00;
        let result = add_slot_offset(&base, 1);
        let mut expected = [0u8; 32];
        expected[31] = 0x00;
        expected[30] = 0x01;
        assert_eq!(result, expected);
    }

    /// Offset = max u8 (255) and base ending in 0x01: result should
    /// roll the last byte to 0x00 and the second-to-last to 0x01.
    /// (0x01 + 0xff = 0x100 → write 0x00, carry 1.)
    #[test]
    fn add_slot_offset_max_u8_with_carry_chain() {
        let mut base = [0u8; 32];
        base[31] = 0x01;
        base[30] = 0x00;
        let result = add_slot_offset(&base, 0xff);
        let mut expected = [0u8; 32];
        expected[31] = 0x00;
        expected[30] = 0x01;
        assert_eq!(result, expected);
    }

    /// Offset 3 — exercising the "skip ticks slot+0/+1/+2 to land on
    /// outerFeeGrowth1Token at +3" pattern. Must produce a slot whose
    /// trailing byte equals (base last byte + 3).
    #[test]
    fn add_slot_offset_three_lands_on_correct_subslot() {
        let mut base = [0u8; 32];
        base[31] = 0x07;
        let result = add_slot_offset(&base, 3);
        assert_eq!(result[31], 0x0a);
        // All other bytes untouched.
        for (i, b) in result.iter().enumerate() {
            if i == 31 {
                continue;
            }
            assert_eq!(*b, 0, "byte {i} should be untouched");
        }
    }

    // ────────────────────────────────────────────────────────────────
    //                  get_full_slot_changes
    // ────────────────────────────────────────────────────────────────

    /// `get_full_slot_changes` should emit one attribute per tracked
    /// whole-slot whose value actually changed. Slots not in the
    /// tracked list must be silently ignored.
    #[test]
    fn full_slot_changes_emits_only_tracked_changed_slots() {
        // Slot 2 (tracked) → changed.
        let s2 = StorageChange {
            address: vec![],
            key: slot_key(2).to_vec(),
            old_value: vec![0u8; 32],
            new_value: {
                let mut v = vec![0u8; 32];
                v[31] = 0x42;
                v
            },
            ordinal: 0,
        };
        // Slot 12 (tracked) → changed.
        let s12 = StorageChange {
            address: vec![],
            key: slot_key(0x0c).to_vec(),
            old_value: vec![0u8; 32],
            new_value: vec![0x99u8; 32],
            ordinal: 1,
        };
        // Slot 100 (NOT tracked) → changed. Should be ignored.
        let s100 = StorageChange {
            address: vec![],
            key: slot_key(100).to_vec(),
            old_value: vec![0u8; 32],
            new_value: vec![0xffu8; 32],
            ordinal: 2,
        };
        // Slot 9 (tracked) → unchanged. Should NOT be emitted.
        let s9_noop = StorageChange {
            address: vec![],
            key: slot_key(9).to_vec(),
            old_value: vec![0xaau8; 32],
            new_value: vec![0xaau8; 32],
            ordinal: 3,
        };
        let changes = [s2, s12, s100, s9_noop];
        let pool_storage = UniswapPoolStorage::new(&changes);
        let attrs = pool_storage.get_full_slot_changes();
        let names: Vec<_> = attrs.iter().map(|a| a.name.as_str()).collect();
        assert_eq!(attrs.len(), 2, "exactly the two changed-tracked slots should be emitted, got: {names:?}");
        assert!(names.contains(&"slot_2"));
        assert!(names.contains(&"slot_12"));
        assert!(!names.contains(&"slot_9"), "unchanged tracked slot must not be emitted");
    }

    /// Empty input → empty output.
    #[test]
    fn full_slot_changes_empty_input_yields_empty_output() {
        let pool_storage = UniswapPoolStorage::new(&[]);
        assert!(pool_storage.get_full_slot_changes().is_empty());
    }

    // ────────────────────────────────────────────────────────────────
    //                  get_ticks_changes
    // ────────────────────────────────────────────────────────────────

    /// Single tick: the helper must compute the correct `ticks[idx]`
    /// mapping slot via keccak256(idx . TICKS_MAP_SLOT) + 1 (for the
    /// liquidityDelta sub-slot), and emit a `ticks/{idx}/net-liquidity`
    /// attribute when that slot changes.
    #[test]
    fn ticks_changes_single_tick_emits_when_changed() {
        // tick index = 100. Compute the expected liquidity-delta slot.
        let tick_idx = BigInt::from(100);
        let map_slot =
            crate::storage::utils::calc_map_slot(
                &crate::storage::utils::left_pad_from_bigint(&tick_idx),
                &crate::storage::constants::TICKS_MAP_SLOT,
            );
        // liquidityDelta lives at map_slot + 1, offset 0, length 16, signed.
        let liquidity_delta_slot = add_slot_offset(&map_slot, 1);

        // Construct a storage change at that exact slot, where the
        // lower 16 bytes (LSB offset 0..16 → MSB byte indices 16..32)
        // change from 0 to a small int128 value.
        let mut new_v = vec![0u8; 32];
        // Place +12345 = 0x...3039 in the int128 region.
        new_v[30] = 0x30;
        new_v[31] = 0x39;
        let change = StorageChange {
            address: vec![],
            key: liquidity_delta_slot.to_vec(),
            old_value: vec![0u8; 32],
            new_value: new_v,
            ordinal: 0,
        };

        let pool_storage = UniswapPoolStorage::new(std::slice::from_ref(&change));
        let attrs = pool_storage.get_ticks_changes(vec![&tick_idx]);
        assert_eq!(attrs.len(), 1);
        assert_eq!(attrs[0].name, "ticks/100/net-liquidity");
        let decoded = BigInt::from_signed_bytes_be(&attrs[0].value);
        assert_eq!(decoded, BigInt::from(12345));
    }

    /// Multiple ticks, only one of which has its liquidity-delta slot
    /// in the storage changes. We expect exactly one attribute back.
    #[test]
    fn ticks_changes_multiple_ticks_only_changed_emitted() {
        let tick_a = BigInt::from(50);
        let tick_b = BigInt::from(250);

        // Only tick_b's slot is in the storage changes.
        let map_b = crate::storage::utils::calc_map_slot(
            &crate::storage::utils::left_pad_from_bigint(&tick_b),
            &crate::storage::constants::TICKS_MAP_SLOT,
        );
        let slot_b = add_slot_offset(&map_b, 1);
        let mut new_v = vec![0u8; 32];
        new_v[31] = 0x07; // small positive delta = 7
        let change = StorageChange {
            address: vec![],
            key: slot_b.to_vec(),
            old_value: vec![0u8; 32],
            new_value: new_v,
            ordinal: 0,
        };

        let pool_storage = UniswapPoolStorage::new(std::slice::from_ref(&change));
        let attrs = pool_storage.get_ticks_changes(vec![&tick_a, &tick_b]);
        assert_eq!(attrs.len(), 1);
        assert_eq!(attrs[0].name, "ticks/250/net-liquidity");
    }

    /// Negative tick index — must keccak-hash the SAME way as the
    /// EVM does for negative-int mapping keys. Algebra ticks are
    /// `int24`, sign-extended to 32 bytes when used as a mapping key.
    #[test]
    fn ticks_changes_negative_tick_index() {
        let tick = BigInt::from(-100);
        let padded = crate::storage::utils::left_pad_from_bigint(&tick);
        // Sanity check the padding: top byte should be 0xff (sign extension).
        assert_eq!(padded[0], 0xff);
        let map_slot = crate::storage::utils::calc_map_slot(
            &padded,
            &crate::storage::constants::TICKS_MAP_SLOT,
        );
        let liquidity_delta_slot = add_slot_offset(&map_slot, 1);

        // Place a negative int128 = -1 = 0xff..ff at LSB offset 0..16
        // → slot byte indices 16..32 all 0xff.
        let mut new_v = vec![0u8; 32];
        for i in 16..32 {
            new_v[i] = 0xff;
        }
        let change = StorageChange {
            address: vec![],
            key: liquidity_delta_slot.to_vec(),
            old_value: vec![0u8; 32],
            new_value: new_v,
            ordinal: 0,
        };

        let pool_storage = UniswapPoolStorage::new(std::slice::from_ref(&change));
        let attrs = pool_storage.get_ticks_changes(vec![&tick]);
        assert_eq!(attrs.len(), 1);
        assert_eq!(attrs[0].name, "ticks/-100/net-liquidity");
        let decoded = BigInt::from_signed_bytes_be(&attrs[0].value);
        assert_eq!(decoded, BigInt::from(-1));
    }

    /// Empty tick list → empty output.
    #[test]
    fn ticks_changes_empty_input_yields_empty_output() {
        let change = make_change([0u8; 32], [1u8; 32]);
        let pool_storage = UniswapPoolStorage::new(std::slice::from_ref(&change));
        let attrs = pool_storage.get_ticks_changes(vec![]);
        assert!(attrs.is_empty());
    }
}
