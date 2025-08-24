use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use url::form_urlencoded;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TokenType {
    #[default]
    None,
    Wrapped,
    Underlying,
}

/// Maps a pool token to its counterpart(s) in Balancer V3's liquidity buffer system.
///
/// In Balancer V3, pools can contain both wrapped tokens (like waEthUSDT) and their underlying
/// tokens (like USDT). The liquidity buffer manages these relationships.
///
/// ## Mapping Logic:
/// - **Querying a wrapped token** → Returns underlying token(s)
///   - `token_type = TokenType::Underlying`
///   - `addresses` contains underlying token address(es)
///
/// - **Querying an underlying token** → Returns wrapped token(s)
///   - `token_type = TokenType::Wrapped`  
///   - `addresses` contains wrapped token address(es)
///
/// ## Examples:
/// ```rust
/// // Example 1: waEthUSDT → USDT mapping
/// let waEthUSDT_mapping = MappingToken {
///     token_type: TokenType::Underlying,
///     addresses: vec![hex::decode("dAC17F958D2ee523a2206206994597C13D831ec7 ").unwrap()], // USDT
/// };
///
/// // Example 2: USDC → Multiple wrapped tokens
/// let usdc_mapping = MappingToken {
///     token_type: TokenType::Wrapped,
///     addresses: vec![
///         hex::decode("BEEF01735c132Ada46AA9aA4c54623cAA92A64CB").unwrap(), // steakUSDC
///         hex::decode("D4fa2D31b7968E448877f69A96DE69f5de8cD23E").unwrap(), // waEthUSDC
///     ],
/// };
/// ```
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MappingToken {
    /// Token addresses that this mapping points to (20-byte Ethereum addresses as Vec<u8>)
    pub addresses: Vec<Vec<u8>>,
    /// Type of tokens stored in the addresses field
    pub token_type: TokenType,
}

pub fn json_serialize_mapping_tokens(tokens: &[MappingToken]) -> Vec<u8> {
    serde_json::to_vec(tokens).expect("Failed to serialize MappingToken array to JSON")
}

#[allow(dead_code)]
pub fn json_deserialize_mapping_tokens(bytes: &[u8]) -> Vec<MappingToken> {
    serde_json::from_slice(bytes).expect("Failed to deserialize MappingToken array from JSON")
}

pub struct Params {
    pub buffer_tokens: HashMap<String, String>,
}

impl Params {
    pub fn parse_from_query(input: &str) -> Result<Self> {
        let map: HashMap<String, String> = form_urlencoded::parse(input.as_bytes())
            .map(|(k, v)| (k.into_owned(), v.into_owned()))
            .collect();
        Ok(Params { buffer_tokens: map })
    }

    pub fn get_mapping_token(&self, token: &Vec<u8>) -> Option<MappingToken> {
        // Phase 1: Check if token is a wrapped token (appears as key)
        // If found, collect all underlying tokens (values) that this wrapped token maps to
        let mapping_underlying_addresses: Vec<Vec<u8>> = self
            .buffer_tokens
            .iter()
            .filter_map(|(k, v)| {
                if hex::decode(k)
                    .ok()
                    .as_ref()
                    .map(|b| b == token)
                    .unwrap_or(false)
                {
                    Some(hex::decode(v).expect("Invalid hex in buffer_tokens value"))
                } else {
                    None
                }
            })
            .collect();

        if !mapping_underlying_addresses.is_empty() {
            return Some(MappingToken {
                addresses: mapping_underlying_addresses,
                token_type: TokenType::Underlying,
            });
        }

        // Phase 2: Check if token is an underlying token (appears as value)  
        // If found, collect all wrapped tokens (keys) that map to this underlying token
        let mapping_wrapped_addresses: Vec<Vec<u8>> = self
            .buffer_tokens
            .iter()
            .filter_map(|(k, v)| {
                if hex::decode(v)
                    .ok()
                    .as_ref()
                    .map(|b| b == token)
                    .unwrap_or(false)
                {
                    Some(hex::decode(k).expect("Invalid hex in buffer_tokens key"))
                } else {
                    None
                }
            })
            .collect();

        if !mapping_wrapped_addresses.is_empty() {
            return Some(MappingToken {
                addresses: mapping_wrapped_addresses,
                token_type: TokenType::Wrapped,
            });
        }

        // No mapping found for this token
        None
    }

    pub fn get_underlying_token(&self, wrapped_token: &Vec<u8>) -> Option<Vec<u8>> {
        self.get_mapping_token(wrapped_token)
            .and_then(|mapping| {
                if mapping.token_type == TokenType::Underlying {
                    mapping.addresses.first().cloned()
                } else {
                    None
                }
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_from_query() {
        let input = "9D39A5DE30e57443BfF2A8307A4256c8797A3497=4c9EDD5852cd905f086C759E8383e09bff1E68B3&5F9D59db355b4A60501544637b00e94082cA575b=4c9EDD5852cd905f086C759E8383e09bff1E68B3";

        let params = Params::parse_from_query(input).expect("Failed to parse query string");

        let mut expected_map = HashMap::new();
        expected_map.insert(
            "9D39A5DE30e57443BfF2A8307A4256c8797A3497".to_string(),
            "4c9EDD5852cd905f086C759E8383e09bff1E68B3".to_string(),
        );
        expected_map.insert(
            "5F9D59db355b4A60501544637b00e94082cA575b".to_string(),
            "4c9EDD5852cd905f086C759E8383e09bff1E68B3".to_string(),
        );

        assert_eq!(params.buffer_tokens, expected_map);
    }

    #[test]
    fn test_get_mapping_token() {
        let mut buffer_tokens = HashMap::new();

        buffer_tokens.insert(
            "9D39A5DE30e57443BfF2A8307A4256c8797A3497".to_lowercase(),
            "4c9EDD5852cd905f086C759E8383e09bff1E68B3".to_lowercase(),
        );
        buffer_tokens.insert(
            "5F9D59db355b4A60501544637b00e94082cA575b".to_lowercase(),
            "4c9EDD5852cd905f086C759E8383e09bff1E68B3".to_lowercase(),
        );

        let params = Params { buffer_tokens };

        let underlying_token = hex::decode("4c9EDD5852cd905f086C759E8383e09bff1E68B3").unwrap();
        let result = params
            .get_mapping_token(&underlying_token)
            .unwrap();
        assert_eq!(result.token_type, TokenType::Wrapped);
        assert_eq!(result.addresses.len(), 2);
        let expected = vec![
            hex::decode("9D39A5DE30e57443BfF2A8307A4256c8797A3497").unwrap(),
            hex::decode("5F9D59db355b4A60501544637b00e94082cA575b").unwrap(),
        ];
        for addr in &result.addresses {
            assert!(expected.contains(addr));
        }

        let wrapped_token = hex::decode("9D39A5DE30e57443BfF2A8307A4256c8797A3497").unwrap();
        let result = params
            .get_mapping_token(&wrapped_token)
            .unwrap();
        assert_eq!(result.token_type, TokenType::Underlying);
        assert_eq!(result.addresses.len(), 1);
        assert_eq!(
            result.addresses[0],
            hex::decode("4c9EDD5852cd905f086C759E8383e09bff1E68B3").unwrap()
        );

        let unknown_token = hex::decode("deadbeef").unwrap();
        assert!(params
            .get_mapping_token(&unknown_token)
            .is_none());
    }

    #[test]
    fn test_get_mapping_token_multiple_matches() {
        let mut buffer_tokens = HashMap::new();

        buffer_tokens.insert("111111111111".to_string(), "aaaaaaaaaaaa".to_string());
        buffer_tokens.insert("222222222222".to_string(), "aaaaaaaaaaaa".to_string());
        buffer_tokens.insert("333333333333".to_string(), "bbbbbbbbbbbb".to_string());

        let params = Params { buffer_tokens };

        let underlying_token = hex::decode("aaaaaaaaaaaa").unwrap();
        let result = params
            .get_mapping_token(&underlying_token)
            .unwrap();
        assert_eq!(result.token_type, TokenType::Wrapped);
        assert_eq!(result.addresses.len(), 2);
        let expected =
            vec![hex::decode("111111111111").unwrap(), hex::decode("222222222222").unwrap()];
        for addr in &result.addresses {
            assert!(expected.contains(addr));
        }

        let underlying_token = hex::decode("bbbbbbbbbbbb").unwrap();
        let result = params
            .get_mapping_token(&underlying_token)
            .unwrap();
        assert_eq!(result.token_type, TokenType::Wrapped);
        assert_eq!(result.addresses.len(), 1);
        assert_eq!(result.addresses[0], hex::decode("333333333333").unwrap());
    }

    #[test]
    fn test_json_serialize_deserialize_mapping_tokens() {
        let tokens = vec![
            MappingToken {
                addresses: vec![
                    hex::decode("0a0b0c0d0e0f").unwrap(),
                    hex::decode("010203040506").unwrap(),
                ],
                token_type: TokenType::Wrapped,
            },
            MappingToken {
                addresses: vec![hex::decode("0f0e0d0c0b0a").unwrap()],
                token_type: TokenType::Underlying,
            },
        ];

        let serialized = json_serialize_mapping_tokens(&tokens);
        let deserialized = json_deserialize_mapping_tokens(&serialized);
        assert_eq!(tokens.len(), deserialized.len());

        for (orig, de) in tokens.iter().zip(deserialized.iter()) {
            assert_eq!(orig.token_type, de.token_type);
            assert_eq!(orig.addresses, de.addresses);
        }
    }
}
