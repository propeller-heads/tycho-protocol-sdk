//! Transaction trace analysis and method signature decoding.
//!
//! This module provides utilities for analyzing Ethereum transaction traces
//! and decoding method signatures using the 4byte.directory database.
//! It helps understand what methods were called during transaction execution
//! and provides human-readable output for debugging and analysis.

use serde_json::Value;
use tokio::time::{timeout, Duration};

/// Decode method selectors using 4byte.directory
pub async fn decode_method_selector(input: &str) -> Option<String> {
    if input.len() < 10 || !input.starts_with("0x") {
        return None;
    }

    let selector = &input[0..10];
    if let Some(signature) = fetch_method_signature(selector).await {
        return Some(signature);
    }

    // Fallback: return the selector with unknown label
    Some(format!("{selector} (unknown)"))
}

/// Fetch method signature from 4byte.directory
async fn fetch_method_signature(selector: &str) -> Option<String> {
    // First check for well-known signatures
    if let Some(sig) = get_well_known_signature(selector) {
        return Some(sig);
    }

    let url = format!("https://www.4byte.directory/api/v1/signatures/?hex_signature={selector}");

    let response = timeout(Duration::from_millis(1500), async {
        reqwest::get(&url)
            .await?
            .json::<serde_json::Value>()
            .await
    })
    .await;

    if let Ok(Ok(json)) = response {
        if let Some(results) = json
            .get("results")
            .and_then(|r| r.as_array())
        {
            // First pass: look for exact matches to well-known patterns
            let mut candidates = Vec::new();
            for result in results.iter() {
                if let Some(signature) = result
                    .get("text_signature")
                    .and_then(|s| s.as_str())
                {
                    if is_legitimate_signature(signature) {
                        candidates.push(signature);
                    }
                }
            }

            // Prioritize common ERC20/DeFi function patterns
            if let Some(best_sig) = find_best_signature(&candidates) {
                return Some(best_sig.to_string());
            }
        }
    }

    None
}

/// Get well-known method signatures for common selectors
fn get_well_known_signature(selector: &str) -> Option<String> {
    match selector {
        // ERC20 Standard
        "0xa9059cbb" => Some("transfer(address,uint256)".to_string()),
        "0x095ea7b3" => Some("approve(address,uint256)".to_string()),
        "0x23b872dd" => Some("transferFrom(address,address,uint256)".to_string()),
        "0x70a08231" => Some("balanceOf(address)".to_string()),
        "0x18160ddd" => Some("totalSupply()".to_string()),
        "0xdd62ed3e" => Some("allowance(address,address)".to_string()),
        "0x06fdde03" => Some("name()".to_string()),
        "0x95d89b41" => Some("symbol()".to_string()),
        "0x313ce567" => Some("decimals()".to_string()),
        
        // Common DeFi functions
        "0x40c10f19" => Some("mint(address,uint256)".to_string()),
        "0x42966c68" => Some("burn(uint256)".to_string()),
        "0x79cc6790" => Some("burnFrom(address,uint256)".to_string()),
        "0x8da5cb5b" => Some("owner()".to_string()),
        "0xf2fde38b" => Some("transferOwnership(address)".to_string()),
        "0x715018a6" => Some("renounceOwnership()".to_string()),
        
        // Balancer V2 specific
        "0x52bbbe29" => Some("swap((bytes32,uint8,address,address,uint256,bytes),((address,bool,address,bool),uint256,uint256),uint256,uint256)".to_string()),
        "0x9d2c110c" => Some("onSwap((uint8,address,address,uint256,bytes32,uint256,address,address,bytes),uint256,uint256)".to_string()),
        "0x48bd7dfd" => Some("execute(bytes32,address)".to_string()),
        
        // Multicall patterns
        "0xac9650d8" => Some("multicall(bytes[])".to_string()),
        "0x5ae401dc" => Some("multicall(uint256,bytes[])".to_string()),
        
        _ => None,
    }
}

/// Find the best signature from a list of candidates
fn find_best_signature<'a>(candidates: &'a [&'a str]) -> Option<&'a str> {
    if candidates.is_empty() {
        return None;
    }

    // Priority patterns (most to least preferred)
    let priority_patterns = [
        // Standard ERC20 functions
        "transfer(", "approve(", "transferFrom(", "balanceOf(", "allowance(",
        "totalSupply(", "name(", "symbol(", "decimals(",
        // Common DeFi patterns
        "swap(", "mint(", "burn(", "deposit(", "withdraw(",
        "owner(", "initialize(", "execute(",
        // Generic patterns (lower priority)
        "get", "set", "add", "remove", "update",
    ];

    // Find highest priority match
    for pattern in &priority_patterns {
        for &candidate in candidates {
            if candidate.starts_with(pattern) {
                return Some(candidate);
            }
        }
    }

    // Prefer shorter, simpler signatures
    candidates.iter()
        .min_by_key(|sig| sig.len())
        .copied()
}

/// Check if a signature looks legitimate (not a scam/honeypot)
fn is_legitimate_signature(signature: &str) -> bool {
    let sig_lower = signature.to_lowercase();

    // Reject obvious scam patterns
    let scam_patterns = [
        "watch_tg", "_tg_", "telegram", "discord", "twitter", "social",
        "invite", "gift", "bonus", "airdrop", "referral", "ref_",
        "_reward", "claim_reward", "_bonus", "_gift", "_invite",
        "honeypot", "rug", "scam", "phish",
    ];

    for pattern in &scam_patterns {
        if sig_lower.contains(pattern) {
            return false;
        }
    }

    // Reject signatures that are suspiciously long (likely auto-generated scam functions)
    if signature.len() > 100 {
        return false;
    }

    // Reject signatures with too many underscores (common in scam functions)
    let underscore_count = signature.matches('_').count();
    if underscore_count > 4 {
        return false;
    }

    // Reject signatures that look like random hex or encoded data
    if signature.matches(char::is_numeric).count() > signature.len() / 2 {
        return false;
    }

    true
}

/// Enhanced trace printing with method selector decoding using 4byte.directory
pub async fn print_call_trace(call: &Value, depth: usize) {
    if depth == 0 {
        println!("Traces:");
    }

    let indent = "  ".repeat(depth);

    if let Some(call_obj) = call.as_object() {
        // Print call information
        let call_type = call_obj
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("UNKNOWN");

        let from = call_obj
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

        let value = call_obj
            .get("value")
            .and_then(|v| v.as_str())
            .unwrap_or("0x0");

        // Convert hex values for display
        let gas_used_dec = if let Some(stripped) = gas_used.strip_prefix("0x") {
            u64::from_str_radix(&stripped[2..], 16).unwrap_or(0)
        } else {
            gas_used.parse().unwrap_or(0)
        };

        // Format in enhanced style with method signature decoding
        println!(
            "{indent}[{depth}] {call_type} {from} -> {to} (gas: {gas_used_dec}, value: {value})"
        );

        // Print input data with method selector decoding using 4byte.directory
        if let Some(input) = call_obj
            .get("input")
            .and_then(|v| v.as_str())
        {
            if !input.is_empty() && input != "0x" {
                if let Some(decoded_method) = decode_method_selector(input).await {
                    println!("{indent}    │ Method: {decoded_method}");
                }
                println!("{indent}    │ Input: {input}");
            }
        }

        // Print output
        if let Some(output) = call_obj
            .get("output")
            .and_then(|v| v.as_str())
        {
            if !output.is_empty() && output != "0x" {
                println!("{indent}    │ Output: {output}");
            }
        }

        // Print error/revert information
        if let Some(error) = call_obj.get("error") {
            println!("{indent}    │ ERROR: {error}");
        }

        if let Some(revert_reason) = call_obj.get("revertReason") {
            println!("{indent}    │ REVERT: {revert_reason}");
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
    async fn test_4byte_signature_decoder() {
        // Test 4byte signature resolution
        let transfer_input = "0xa9059cbb000000000000000000000000d8da6bf26964af9d7eed9e03e53415d37aa96045000000000000000000000000000000000000000000000000016345785d8a0000";
        let result = decode_method_selector(transfer_input).await;
        println!("4byte decoded transfer: {result:?}");

        // Test some other common selector
        let approve_input = "0x095ea7b3000000000000000000000000d8da6bf26964af9d7eed9e03e53415d37aa96045000000000000000000000000000000000000000000000000016345785d8a0000";
        let result = decode_method_selector(approve_input).await;
        println!("4byte decoded approve: {result:?}");

        // Should return something (either signature or unknown)
        assert!(result.is_some());
    }

    #[tokio::test]
    async fn test_legitimate_signature_filtering() {
        // Test legitimate signature
        assert!(is_legitimate_signature("transfer(address,uint256)"));
        assert!(is_legitimate_signature("swap(uint256,uint256,address,bytes)"));

        // Test scam signatures
        assert!(!is_legitimate_signature("watch_tg_invmru_119a5a98(address,uint256)"));
        assert!(!is_legitimate_signature("telegram_airdrop_bonus(address,uint256)"));
        assert!(!is_legitimate_signature("claimReward_giftCode_12345(address,uint256)"));

        // Test overly long signatures (likely auto-generated scams)
        assert!(!is_legitimate_signature("someVeryLongFunctionNameThatIsProbablyGeneratedByScammersToHideRealFunctionality(address,uint256)"));
    }

    #[tokio::test]
    async fn test_print_call_trace_with_decoding() {
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
