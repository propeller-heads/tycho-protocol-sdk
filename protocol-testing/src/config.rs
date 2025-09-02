use std::collections::HashMap;

use colored::Colorize;
use serde::{Deserialize, Serialize};
use similar::{ChangeTag, TextDiff};
use tycho_core::{dto::ProtocolComponent, Bytes};

/// Represents a ProtocolComponent with its main attributes
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProtocolComponentExpectation {
    pub id: String,
    pub tokens: Vec<Bytes>,
    #[serde(default)]
    pub static_attributes: HashMap<String, Bytes>,
    pub creation_tx: Bytes,
}

/// Represents a ProtocolComponent with test configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProtocolComponentWithTestConfig {
    #[serde(flatten)]
    pub base: ProtocolComponentExpectation,
    #[serde(default = "default_false")]
    pub skip_simulation: bool,
}

impl ProtocolComponentExpectation {
    pub fn compare(&self, other: &ProtocolComponent, colorize_output: bool) -> Option<String> {
        let mut diffs = Vec::new();

        // Compare id
        if self.id != other.id {
            let diff = self.format_diff("id", &self.id, &other.id, colorize_output);
            diffs.push(format!("Field 'id' mismatch for {}:\n{}", self.id, diff));
        }

        // Compare tokens
        if self.tokens != other.tokens {
            let self_tokens = format!("{:?}", self.tokens);
            let other_tokens = format!("{:?}", other.tokens);
            let diff = self.format_diff("tokens", &self_tokens, &other_tokens, colorize_output);
            diffs.push(format!("Field 'tokens' mismatch for {}:\n{}", self.id, diff));
        }

        // Compare static_attributes
        for (key, value) in &self.static_attributes {
            let other_value = other.static_attributes.get(key);
            match other_value {
                Some(other_value) => {
                    if value != other_value {
                        let self_value = format!("{:?}", value);
                        let other_value = format!("{:?}", other_value);
                        let diff = self.format_diff(
                            "static_attributes",
                            &self_value,
                            &other_value,
                            colorize_output,
                        );
                        diffs.push(format!(
                            "Field 'static_attributes' mismatch for {}:\n{}",
                            self.id, diff
                        ));
                    }
                }
                None => {
                    diffs.push(format!(
                        "Field 'static_attributes' mismatch for {}: Key '{}' not found",
                        self.id, key
                    ));
                }
            }
        }
        // Compare creation_tx
        if self.creation_tx != other.creation_tx {
            let self_tx = format!("{}", self.creation_tx.clone());
            let other_tx = format!("{}", other.creation_tx.clone());
            let diff = self.format_diff("creation_tx", &self_tx, &other_tx, colorize_output);
            diffs.push(format!("Field 'creation_tx' mismatch for {}:\n{}", self.id, diff));
        }

        if diffs.is_empty() {
            None
        } else {
            Some(diffs.join("\n"))
        }
    }
    fn format_diff(&self, _field_name: &str, left: &str, right: &str, colorize: bool) -> String {
        let diff = TextDiff::from_lines(left, right);

        let mut result = String::new();
        for change in diff.iter_all_changes() {
            let formatted = match change.tag() {
                ChangeTag::Delete => {
                    if colorize {
                        format!("{}", format!("-{}", change.value().trim_end()).red())
                    } else {
                        format!("-{}", change.value().trim_end())
                    }
                }
                ChangeTag::Insert => {
                    if colorize {
                        format!("{}", format!("+{}", change.value().trim_end()).green())
                    } else {
                        format!("+{}", change.value().trim_end())
                    }
                }
                ChangeTag::Equal => {
                    format!(" {}", change.value().trim_end())
                }
            };
            result.push_str(&formatted);
            result.push('\n');
        }

        result
    }
}

fn default_false() -> bool {
    false
}

/// Configuration for an individual test
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct IntegrationTest {
    pub name: String,
    pub start_block: u64,
    pub stop_block: u64,
    pub initialized_accounts: Option<Vec<String>>,
    pub expected_components: Vec<ProtocolComponentWithTestConfig>,
}

/// Main integration test configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct IntegrationTestsConfig {
    pub substreams_yaml_path: String,
    pub adapter_contract: String,
    pub adapter_build_signature: Option<String>,
    pub adapter_build_args: Option<String>,
    pub initialized_accounts: Option<Vec<String>>,
    pub skip_balance_check: bool,
    pub protocol_type_names: Vec<String>,
    pub tests: Vec<IntegrationTest>,
}
