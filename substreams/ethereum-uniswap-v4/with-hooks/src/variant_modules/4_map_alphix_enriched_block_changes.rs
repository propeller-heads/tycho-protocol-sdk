use substreams_helper::hex::Hexable;
use tycho_substreams::prelude::*;

/// Enriches protocol changes by adding `hook_identifier: "alphix_v1"` to pools
/// that use Alphix hooks.
///
/// Unlike factory-based hooks (e.g., Euler), Alphix hooks are deployed at known addresses.
/// The `params` string contains a comma-separated list of hook addresses.
/// This module checks pool creations directly against the configured addresses,
/// eliminating the need for an intermediate store.
#[substreams::handlers::map]
pub fn map_alphix_enriched_block_changes(
    params: String,
    block_changes: BlockChanges,
) -> Result<BlockChanges, substreams::errors::Error> {
    let hook_addresses = parse_hook_addresses(&params);
    Ok(enrich_block_changes(block_changes, &hook_addresses))
}

pub fn enrich_block_changes(
    mut protocol_changes: BlockChanges,
    hook_addresses: &[String],
) -> BlockChanges {
    for tx_changes in &mut protocol_changes.changes {
        for component in &mut tx_changes.component_changes {
            if component.change == i32::from(ChangeType::Creation) {
                if let Some(hooks_attr) = component
                    .static_att
                    .iter()
                    .find(|attr| attr.name == "hooks")
                {
                    let hook_address = hooks_attr.value.to_hex();

                    if hook_addresses
                        .iter()
                        .any(|addr| addr == &hook_address)
                    {
                        component
                            .static_att
                            .push(Attribute {
                                name: "hook_identifier".to_string(),
                                value: "alphix_v1".as_bytes().to_vec(),
                                change: ChangeType::Creation.into(),
                            });
                    }
                }
            }
        }
    }

    protocol_changes
}

/// Parse comma-separated hook addresses from params string.
/// Accepts addresses with or without 0x prefix, normalizes to `0x`-prefixed lowercase
/// to match the output of `Hexable::to_hex()`.
pub fn parse_hook_addresses(params: &str) -> Vec<String> {
    params
        .split(',')
        .map(|s| {
            let trimmed = s.trim().trim_start_matches("0x").to_lowercase();
            format!("0x{}", trimmed)
        })
        .filter(|s| s.len() > 2) // filter out empty entries that become just "0x"
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hook_addresses() {
        let params = "0x831CfDf7c0E194f5369f204b3DD2481B843d60c0,0x0e4b892Df7C5Bcf5010FAF4AA106074e555660C0";
        let addresses = parse_hook_addresses(params);
        assert_eq!(addresses.len(), 2);
        assert_eq!(addresses[0], "0x831cfdf7c0e194f5369f204b3dd2481b843d60c0");
        assert_eq!(addresses[1], "0x0e4b892df7c5bcf5010faf4aa106074e555660c0");
    }

    #[test]
    fn test_parse_single_address() {
        let params = "5e645C3D580976Ca9e3fe77525D954E73a0Ce0C0";
        let addresses = parse_hook_addresses(params);
        assert_eq!(addresses.len(), 1);
        assert_eq!(addresses[0], "0x5e645c3d580976ca9e3fe77525d954e73a0ce0c0");
    }

    #[test]
    fn test_parse_empty() {
        let addresses = parse_hook_addresses("");
        assert_eq!(addresses.len(), 0);
    }
}
