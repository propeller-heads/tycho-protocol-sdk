use anyhow::{anyhow, Result};
use serde::Deserialize;
use substreams_helper::hex::Hexable;
use tycho_substreams::{
    entrypoint::create_entrypoint,
    prelude::{entry_point_params::TraceData, *},
};

#[derive(Debug, Deserialize)]
pub struct Params {
    pub hook_address: String,
    pub oracle_address: String,
}

impl Params {
    pub fn parse_from_query(input: &str) -> Result<Self> {
        serde_qs::from_str(input).map_err(|e| anyhow!("Failed to parse query params: {}", e))
    }
}

#[substreams::handlers::map]
pub fn map_universal_enriched_protocol_changes(
    params: String,
    protocol_changes: BlockChanges,
) -> Result<BlockChanges, substreams::errors::Error> {
    let params = Params::parse_from_query(&params)?;
    let universal_hook_address = params.hook_address.as_str();
    let universal_oracle_address = params.oracle_address.as_str();

    let enriched_changes = _enrich_protocol_changes(
        universal_hook_address,
        universal_oracle_address,
        protocol_changes,
    );

    Ok(enriched_changes)
}

pub fn _enrich_protocol_changes(
    universal_hook_address: &str,
    universal_oracle_address: &str,
    mut protocol_changes: BlockChanges,
) -> BlockChanges {
    // Process each transaction's changes
    for tx_changes in &mut protocol_changes.changes {
        for component_change in &mut tx_changes.component_changes {
            if component_change.change == i32::from(ChangeType::Creation) {
                // Check if this component has a hooks attribute
                if let Some(hooks_attr) = component_change
                    .static_att
                    .iter()
                    .find(|attr| attr.name == "hooks")
                {
                    let hook_address = hooks_attr.value.to_hex();
                    if hook_address.to_lowercase() == universal_hook_address {
                        // Add the hook_identifier static attribute
                        component_change
                            .static_att
                            .push(Attribute {
                                name: "hook_identifier".to_string(),
                                value: "universal_v1".as_bytes().to_vec(),
                                change: ChangeType::Creation.into(),
                            });
                        let component_id_bytes = match hex::decode(
                            component_change
                                .id
                                .trim_start_matches("0x"),
                        ) {
                            Ok(bytes) if bytes.len() == 32 => bytes,
                            _ => continue,
                        };
                        let target =
                            match hex::decode(universal_oracle_address.trim_start_matches("0x")) {
                                Ok(bytes) if bytes.len() == 20 => bytes,
                                _ => continue,
                            };
                        let mut calldata = Vec::with_capacity(4 + 32);
                        calldata.extend_from_slice(
                            hex::decode("bc60a7aa")
                                .unwrap()
                                .as_slice(), // safeGetPrice(bytes32)
                        );
                        calldata.extend_from_slice(&component_id_bytes);
                        let trace_data = TraceData::Rpc(RpcTraceData { caller: None, calldata });
                        let (entrypoint, entrypoint_param) = create_entrypoint(
                            target,
                            "safeGetPrice(bytes32)".into(),
                            component_change.id.clone(),
                            trace_data,
                        );
                        tx_changes.entrypoints.push(entrypoint);
                        tx_changes
                            .entrypoint_params
                            .push(entrypoint_param);
                    }
                }
            }
        }
    }

    protocol_changes
}
