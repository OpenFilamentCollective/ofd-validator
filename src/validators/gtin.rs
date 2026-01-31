//! GTIN/EAN validator
//!
//! Validates GTIN and EAN fields in sizes.json files

use crate::types::{ValidationError, ValidationLevel, ValidationResult};
use crate::utils::load_json;
use regex::Regex;
use std::path::Path;
use walkdir::WalkDir;

lazy_static::lazy_static! {
    static ref GTIN_RE: Regex = Regex::new(r"^[0-9]{12,13}$").unwrap();
    static ref EAN_RE: Regex = Regex::new(r"^[0-9]{13}$").unwrap();
}

pub struct GTINValidator;

impl GTINValidator {
    pub fn new() -> Self {
        Self
    }

    pub fn validate_gtin_ean<P: AsRef<Path>>(&self, data_dir: P) -> ValidationResult {
        let data_dir = data_dir.as_ref();
        let mut result = ValidationResult::new();

        if !data_dir.exists() {
            return result;
        }

        for entry in WalkDir::new(data_dir) {
            let Ok(entry) = entry else { continue };
            if entry.file_name() != "sizes.json" {
                continue;
            }

            let Ok(Some(sizes_data)) = load_json(entry.path()) else { continue };
            let Some(sizes_array) = sizes_data.as_array() else { continue };

            for (idx, size) in sizes_array.iter().enumerate() {
                let gtin = size.get("gtin").and_then(|v| v.as_str());
                let ean = size.get("ean").and_then(|v| v.as_str());

                // Validate GTIN if present
                if let Some(gtin_val) = gtin {
                    if !GTIN_RE.is_match(gtin_val) {
                        result.add_error(ValidationError::with_path(
                            ValidationLevel::Error,
                            "GTIN",
                            format!("Invalid gtin at $[{}]: must be 12 or 13 digits", idx),
                            entry.path(),
                        ));
                    }
                }

                // Validate EAN if present
                if let Some(ean_val) = ean {
                    if !EAN_RE.is_match(ean_val) {
                        result.add_error(ValidationError::with_path(
                            ValidationLevel::Error,
                            "EAN",
                            format!("Invalid ean at $[{}]: must be exactly 13 digits", idx),
                            entry.path(),
                        ));
                    }
                }

                // Check consistency when both present
                if let (Some(gtin_val), Some(ean_val)) = (gtin, ean) {
                    if gtin_val.len() == 13 && ean_val.len() == 13 && gtin_val != ean_val {
                        result.add_error(ValidationError::with_path(
                            ValidationLevel::Error,
                            "GTIN/EAN",
                            format!("Mismatch at $[{}]: gtin and ean are both 13 digits but not equal", idx),
                            entry.path(),
                        ));
                    }
                }
            }
        }

        result
    }
}

impl Default for GTINValidator {
    fn default() -> Self {
        Self::new()
    }
}
