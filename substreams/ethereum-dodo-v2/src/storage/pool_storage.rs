use crate::storage::{
    constants::{DPP_PMM_SLOTS, DSP_PMM_SLOTS, DVM_PMM_SLOTS, GSP_PMM_SLOTS},
    utils::read_bytes,
};
use substreams::prelude::BigInt;
use substreams_ethereum::pb::eth::v2::StorageChange;
use tycho_substreams::prelude::{Attribute, ChangeType};

/// `StorageLocation` is a struct that represents a specific location within a contract's storage
/// associated with a name.
///
/// # Fields
///
/// * `name` - A string slice (`&str`) reference representing the unique name associated with this
///   storage location.
/// * `slot` - A fixed-size byte array `[u8; 32]` representing the slot in the contract storage
///   where this data is stored. This acts as a primary identifier for the location of the data.
/// * `offset` - A usize value indicating the offset in bytes from the start of the slot. This
///   allows for fine-grained control and access within a single slot.
/// * `number_of_bytes` - A usize value indicating the size of the data in bytes.
/// ```
#[derive(Clone)]
pub struct StorageLocation<'a> {
    pub name: &'a str,
    pub slot: [u8; 32],
    pub offset: usize,
    pub number_of_bytes: usize,
    pub signed: bool,
}

pub struct DoDoPoolStorage<'a> {
    pub pool_type: &'a str,
    pub storage_changes: &'a Vec<StorageChange>,
}

impl<'a> DoDoPoolStorage<'a> {
    pub fn new(pool_type: &'a str, storage_changes: &'a Vec<StorageChange>) -> DoDoPoolStorage<'a> {
        Self { pool_type, storage_changes }
    }

    pub fn get_changed_attributes(&self) -> Vec<Attribute> {
        let mut attributes = Vec::new();
        let locations = match self.pool_type {
            "gsp" => GSP_PMM_SLOTS.to_vec(),
            "dpp" => DPP_PMM_SLOTS.to_vec(),
            "dsp" => DSP_PMM_SLOTS.to_vec(),
            "dvm" => DVM_PMM_SLOTS.to_vec(),
            _ => {
                vec![]
            }
        };

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
}
