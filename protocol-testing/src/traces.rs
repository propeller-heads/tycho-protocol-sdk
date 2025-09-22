//! Transaction trace analysis with foundry signature decoding.
//!
//! This module provides utilities for analyzing Ethereum transaction traces
//! and decoding method signatures using foundry's comprehensive signature database.

use colored::Colorize;
use foundry_evm::traces::identifier::SignaturesIdentifier;
use serde_json::Value;

/// Decode method selectors using foundry's signature database with scam filtering
pub async fn decode_method_selector(input: &str) -> Option<String> {
    if input.len() < 10 || !input.starts_with("0x") {
        return None;
    }

    let selector_bytes = hex::decode(&input[2..10]).ok()?;
    if selector_bytes.len() != 4 {
        return None;
    }
    let selector: [u8; 4] = selector_bytes.try_into().ok()?;
    let selector_fixed: alloy::primitives::FixedBytes<4> = selector.into();

    // Use foundry's signature identifier
    if let Ok(sig_identifier) = SignaturesIdentifier::new(true) {
        if let Some(signature) = sig_identifier.identify_function(selector_fixed).await {
            let formatted_sig = format!(
                "{}({})",
                signature.name,
                signature
                    .inputs
                    .iter()
                    .map(|p| p.ty.as_str())
                    .collect::<Vec<_>>()
                    .join(",")
            );
            
            // Filter out scam/honeypot signatures
            if is_legitimate_signature(&formatted_sig) {
                return Some(formatted_sig);
            }
        }
    }

    // Return unknown if not found
    Some(format!("{} (unknown)", &input[0..10]))
}

/// Check if a signature looks legitimate (not a scam/honeypot)
fn is_legitimate_signature(signature: &str) -> bool {
    let sig_lower = signature.to_lowercase();

    // Reject obvious scam patterns
    let scam_patterns = [
        "watch_tg", "_tg_", "telegram", "discord", "twitter", "social",
        "invite", "gift", "bonus", "airdrop", "referral", "ref_",
        "_reward", "claim_reward", "_bonus", "_gift", "_invite",
        "honeypot", "rug", "scam", "phish", "sub2juniononyoutube",
        "youtube", "sub2", "junion",
    ];

    for pattern in &scam_patterns {
        if sig_lower.contains(pattern) {
            return false;
        }
    }

    // Reject signatures that are suspiciously long (likely auto-generated scam functions)
    if signature.len() > 80 {
        return false;
    }

    // Reject signatures with too many underscores (common in scam functions)
    let underscore_count = signature.matches('_').count();
    if underscore_count > 3 {
        return false;
    }

    // Reject signatures that look like random hex or encoded data
    if signature.matches(char::is_numeric).count() > signature.len() / 2 {
        return false;
    }

    true
}

/// Trace printing with foundry-style formatting and colors
pub async fn print_call_trace(call: &Value, depth: usize) {
    if depth == 0 {
        println!("{}", "Traces:".cyan().bold());
    }

    if let Some(call_obj) = call.as_object() {
        // Parse trace data
        let call_type = call_obj
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("UNKNOWN");

        let _from = call_obj
            .get("from")
            .and_then(|v| v.as_str())
            .unwrap_or("0x?");

        let to = call_obj
            .get("to")
            .and_then(|v| v.as_str())
            .unwrap_or("0x?");

        let gas_used = call_obj
            .get("gasUsed")
            .and_then(|v| v.as_str())
            .unwrap_or("0x0");

        let _value = call_obj
            .get("value")
            .and_then(|v| v.as_str())
            .unwrap_or("0x0");

        // Convert hex values for display
        let gas_used_dec = if let Some(stripped) = gas_used.strip_prefix("0x") {
            u64::from_str_radix(stripped, 16).unwrap_or(0)
        } else {
            gas_used.parse().unwrap_or(0)
        };

        // Check if call failed
        let has_error = call_obj.get("error").is_some();
        let has_revert = call_obj.get("revertReason").is_some();
        let call_failed = has_error || has_revert;

        // Create tree structure prefix
        let tree_prefix = if depth == 0 { "".to_string() } else { "  ".repeat(depth) + "├─ " };

        // Get input for method signature decoding
        let input = call_obj
            .get("input")
            .and_then(|v| v.as_str())
            .unwrap_or("0x");

        // Decode method signature
        let method_sig = if !input.is_empty() && input != "0x" {
            decode_method_selector(input)
                .await
                .unwrap_or_else(|| "unknown".to_string())
        } else {
            format!("{}()", call_type.to_lowercase())
        };

        // Format the main call line with colors
        let gas_str = format!("[{}]", gas_used_dec);
        let call_part = format!("{}::{}", to, method_sig);

        if call_failed {
            println!("{}{} {}", tree_prefix, gas_str, call_part.red());
        } else {
            println!("{}{} {}", tree_prefix, gas_str, call_part.green());
        }

        // Print return/revert information with proper indentation
        let result_indent = "  ".repeat(depth + 1) + "└─ ← ";

        if let Some(error) = call_obj.get("error") {
            println!("{}{}", result_indent, format!("[Error] {}", error));
        } else if let Some(revert_reason) = call_obj.get("revertReason") {
            println!("{}{}", result_indent, format!("[Revert] {}", revert_reason));
        } else if let Some(output) = call_obj
            .get("output")
            .and_then(|v| v.as_str())
        {
            if !output.is_empty() && output != "0x" {
                println!("{}{}", result_indent, format!("[Return] {}", output));
            } else {
                println!("{}{}", result_indent, "[Return]");
            }
        }

        // Recursively print nested calls
        if let Some(calls) = call_obj.get("calls") {
            if let Some(calls_array) = calls.as_array() {
                for nested_call in calls_array {
                    Box::pin(print_call_trace(nested_call, depth + 1)).await;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[tokio::test]
    async fn test_foundry_signature_decoder() {
        // Test foundry signature resolution
        let transfer_input = "0xa9059cbb000000000000000000000000d8da6bf26964af9d7eed9e03e53415d37aa96045000000000000000000000000000000000000000000000000016345785d8a0000";
        let result = decode_method_selector(transfer_input).await;
        println!("Foundry decoded transfer: {result:?}");

        // Test some other common selector
        let approve_input = "0x095ea7b3000000000000000000000000d8da6bf26964af9d7eed9e03e53415d37aa96045000000000000000000000000000000000000000000000000016345785d8a0000";
        let result = decode_method_selector(approve_input).await;
        println!("Foundry decoded approve: {result:?}");

        // Should return something (either signature or unknown)
        assert!(result.is_some());
    }

    #[tokio::test]
    async fn test_print_call_trace_with_foundry_decoding() {
        // Test trace with ERC20 transfer
        let trace_json = json!({
            "type": "CALL",
            "from": "0x1234567890abcdef1234567890abcdef12345678",
            "to": "0xabcdef1234567890abcdef1234567890abcdef12",
            "gasUsed": "0x5208",
            "value": "0x0",
            "input": "0xa9059cbb000000000000000000000000d8da6bf26964af9d7eed9e03e53415d37aa96045000000000000000000000000000000000000000000000000016345785d8a0000",
            "output": "0x0000000000000000000000000000000000000000000000000000000000000001",
            "calls": []
        });

        // This test mainly ensures the function runs without panicking
        print_call_trace(&trace_json, 0).await;
    }
}
