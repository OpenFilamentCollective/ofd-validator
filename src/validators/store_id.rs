use std::collections::HashSet;
use std::path::Path;
use walkdir::WalkDir;

use crate::types::{ValidationError, ValidationResult};
use crate::util::load_json;

pub fn validate_store_ids_impl(data_dir: &Path, stores_dir: &Path) -> ValidationResult {
    let mut result = ValidationResult::default();

    // Collect valid store IDs
    let mut valid_store_ids = HashSet::new();
    if let Ok(entries) = std::fs::read_dir(stores_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let store_dir = entry.path();
            if !store_dir.is_dir() {
                continue;
            }
            let store_file = store_dir.join("store.json");
            if let Some(data) = load_json(&store_file) {
                if let Some(id) = data.get("id").and_then(|v| v.as_str()) {
                    valid_store_ids.insert(id.to_string());
                }
            }
        }
    }

    // Validate references in sizes.json files
    for entry in WalkDir::new(data_dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_name() != "sizes.json" {
            continue;
        }

        let sizes_file = entry.path();
        let sizes_data = match load_json(sizes_file) {
            Some(v) => v,
            None => continue,
        };

        let sizes_arr = match sizes_data.as_array() {
            Some(a) => a,
            None => continue,
        };

        let path_str = sizes_file.to_string_lossy().to_string();

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
                            Some(path_str.clone()),
                        ));
                    }
                }
            }
        }
    }

    result
}
