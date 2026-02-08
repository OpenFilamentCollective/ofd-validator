use std::collections::HashSet;
use serde_json::Value;

use crate::types::{ValidationError, ValidationResult};

/// Validate store IDs referenced in sizes.json purchase_links.
/// `valid_store_ids` is the set of known store IDs from store.json files.
/// `sizes_entries` is a list of (path_label, parsed sizes.json Value).
pub fn validate_store_ids(
    valid_store_ids: &HashSet<String>,
    sizes_entries: &[(&str, &Value)],
) -> ValidationResult {
    let mut result = ValidationResult::default();

    for (path_str, sizes_data) in sizes_entries {
        let sizes_arr = match sizes_data.as_array() {
            Some(a) => a,
            None => continue,
        };

        for (size_idx, size) in sizes_arr.iter().enumerate() {
            let purchase_links = match size.get("purchase_links").and_then(|v| v.as_array()) {
                Some(links) => links,
                None => continue,
            };

            for (link_idx, link) in purchase_links.iter().enumerate() {
                if let Some(store_id) = link.get("store_id").and_then(|v| v.as_str()) {
                    if !valid_store_ids.contains(store_id) {
                        result.add(ValidationError::error(
                            "StoreID",
                            format!(
                                "Invalid store_id '{}' at $[{}].purchase_links[{}]",
                                store_id, size_idx, link_idx
                            ),
                            Some(path_str.to_string()),
                        ));
                    }
                }
            }
        }
    }

    result
}
