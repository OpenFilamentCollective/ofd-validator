use regex::Regex;
use serde_json::Value;
use std::sync::LazyLock;

use crate::types::{ValidationError, ValidationResult};

static GTIN_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[0-9]{12,13}$").unwrap());
static EAN_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[0-9]{13}$").unwrap());

/// Validate GTIN/EAN fields in pre-loaded sizes.json entries.
/// Each entry is (path_label, parsed sizes.json Value).
pub fn validate_gtin_ean(sizes_entries: &[(&str, &Value)]) -> ValidationResult {
    let mut result = ValidationResult::default();

    for (path_str, sizes_data) in sizes_entries {
        let sizes_arr = match sizes_data.as_array() {
            Some(a) => a,
            None => continue,
        };

        for (idx, size) in sizes_arr.iter().enumerate() {
            let gtin = size.get("gtin").and_then(|v| v.as_str());
            let ean = size.get("ean").and_then(|v| v.as_str());

            if let Some(gtin_val) = gtin {
                if !GTIN_RE.is_match(gtin_val) {
                    result.add(ValidationError::error(
                        "GTIN",
                        format!("Invalid gtin at $[{}]: must be 12 or 13 digits", idx),
                        Some(path_str.to_string()),
                    ));
                }
            }

            if let Some(ean_val) = ean {
                if !EAN_RE.is_match(ean_val) {
                    result.add(ValidationError::error(
                        "EAN",
                        format!("Invalid ean at $[{}]: must be exactly 13 digits", idx),
                        Some(path_str.to_string()),
                    ));
                }
            }

            // When both present and both 13 digits, they must match
            if let (Some(gtin_val), Some(ean_val)) = (gtin, ean) {
                if gtin_val.len() == 13 && ean_val.len() == 13 && gtin_val != ean_val {
                    result.add(ValidationError::error(
                        "GTIN/EAN",
                        format!(
                            "Mismatch at $[{}]: gtin and ean are both 13 digits but not equal",
                            idx
                        ),
                        Some(path_str.to_string()),
                    ));
                }
            }
        }
    }

    result
}
