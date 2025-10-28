use substreams_ethereum::pb::eth::v2::{self as eth};
use tycho_substreams::{block_storage::get_block_storage_changes, prelude::*};

#[substreams::handlers::map]
pub fn map_dci_protocol_changes(
    block: eth::Block,
    protocol_changes: BlockChanges,
) -> Result<BlockChanges, substreams::errors::Error> {
    // Add storage changes (required by DCI)
    let block_storage_changes = get_block_storage_changes(&block);

    let mut updated_changes = protocol_changes;
    updated_changes.storage_changes = block_storage_changes;

    Ok(updated_changes)
}
