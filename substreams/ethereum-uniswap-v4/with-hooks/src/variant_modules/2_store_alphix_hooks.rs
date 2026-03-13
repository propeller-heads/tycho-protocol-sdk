use substreams::store::{StoreNew, StoreSetIfNotExists, StoreSetIfNotExistsInt64};
use substreams_helper::hex::Hexable;
use tycho_substreams::{models::ChangeType, prelude::*};

/// Stores known Alphix hook addresses so they can be matched against pool creations.
///
/// Unlike factory-based hooks (e.g., Euler), Alphix hooks are deployed at known addresses.
/// The `params` string contains a comma-separated list of hook addresses (lowercase, no 0x prefix).
/// This makes it easy to add new hooks by updating the YAML params without code changes.
#[substreams::handlers::store]
pub fn store_alphix_hooks(
    params: String,
    protocol_changes: BlockChanges,
    output: StoreSetIfNotExistsInt64,
) {
    let hook_addresses: Vec<String> = parse_hook_addresses(&params);

    for tx_changes in protocol_changes.changes {
        for component in tx_changes.component_changes {
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
                        // Store the hook address so downstream modules can identify Alphix pools
                        output.set_if_not_exists(0, &hook_address, &1);
                    }
                }
            }
        }
    }
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
