//! Store ID validator
//!
//! Validates that store_id references in purchase_links are valid

use crate::types::{ValidationError, ValidationLevel, ValidationResult};
use crate::utils::{helpers, load_json};
use std::collections::HashSet;
use std::path::Path;
use walkdir::WalkDir;

pub struct StoreIdValidator;

impl StoreIdValidator {
    pub fn new() -> Self {
        Self
    }

    pub fn validate_store_ids<P: AsRef<Path>>(
        &self,
        data_dir: P,
        stores_dir: P,
    ) -> ValidationResult {
        let data_dir = data_dir.as_ref();
        let stores_dir = stores_dir.as_ref();
        let mut result = ValidationResult::new();

        // Collect valid store IDs
        let mut valid_store_ids = HashSet::new();
        if stores_dir.exists() {
            for entry in WalkDir::new(stores_dir).min_depth(2).max_depth(2) {
                let Ok(entry) = entry else { continue };
                if entry.file_name() != "store.json" {
                    continue;
                }
                if let Ok(Some(data)) = load_json(entry.path()) {
                    if let Some(id) = helpers::get_json_string(&data, "id") {
                        valid_store_ids.insert(id);
                    }
                }
            }
        }

        // Validate references in sizes.json files
        if data_dir.exists() {
            for entry in WalkDir::new(data_dir) {
                let Ok(entry) = entry else { continue };
                if entry.file_name() != "sizes.json" {
                    continue;
                }

                let Ok(Some(sizes_data)) = load_json(entry.path()) else { continue };
                let Some(sizes_array) = sizes_data.as_array() else { continue };

                for (size_idx, size) in sizes_array.iter().enumerate() {
                    let Some(links) = size.get("purchase_links").and_then(|v| v.as_array()) else { continue };

                    for (link_idx, link) in links.iter().enumerate() {
                        if let Some(store_id) = link.get("store_id").and_then(|v| v.as_str()) {
                            if !valid_store_ids.contains(store_id) {
                                result.add_error(ValidationError::with_path(
                                    ValidationLevel::Error,
                                    "StoreID",
                                    format!("Invalid store_id '{}' at $[{}].purchase_links[{}]", store_id, size_idx, link_idx),
                                    entry.path(),
                                ));
                            }
                        }
                    }
                }
            }
        }

        result
    }
}

impl Default for StoreIdValidator {
    fn default() -> Self {
        Self::new()
    }
}
