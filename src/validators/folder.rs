//! Folder name validator
//!
//! Validates that folder names match the corresponding values in JSON files

use crate::types::{ValidationError, ValidationLevel, ValidationResult};
use crate::utils::{cleanse_folder_name, helpers, load_json};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const ILLEGAL_CHARACTERS: &[char] = &[
    '#', '%', '&', '{', '}', '\\', '<', '>', '*', '?', '/', '$', '!', '\'', '"', ':', '@', '`',
    '|', '=',
];

/// Folder name validator
pub struct FolderNameValidator;

impl FolderNameValidator {
    pub fn new() -> Self {
        Self
    }

    /// Validate that a folder name matches the value in its JSON file
    pub fn validate_folder_name<P: AsRef<Path>>(
        &self,
        folder_path: P,
        json_file: &str,
        json_key: &str,
    ) -> ValidationResult {
        let folder_path = folder_path.as_ref();
        let mut result = ValidationResult::new();

        let json_path = folder_path.join(json_file);
        if !json_path.exists() {
            result.add_error(ValidationError::with_path(
                ValidationLevel::Error,
                "Folder",
                format!("Missing {}", json_file),
                folder_path,
            ));
            return result;
        }

        let data = match load_json(&json_path) {
            Ok(Some(d)) => d,
            Ok(None) | Err(_) => return result,
        };

        if let Some(expected_name) = helpers::get_json_string(&data, json_key) {
            let expected_name = cleanse_folder_name(&expected_name);
            let actual_name = folder_path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            if actual_name != expected_name {
                let has_illegal = expected_name.chars().any(|c| ILLEGAL_CHARACTERS.contains(&c));

                if !has_illegal {
                    result.add_error(ValidationError::with_path(
                        ValidationLevel::Error,
                        "Folder",
                        format!(
                            "Folder name '{}' does not match '{}' value '{}' in {}",
                            actual_name, json_key, expected_name, json_file
                        ),
                        folder_path,
                    ));
                }
            }
        }

        result
    }

    pub fn collect_validation_tasks<P: AsRef<Path>>(
        data_dir: P,
        stores_dir: P,
    ) -> Vec<(PathBuf, String, String)> {
        let mut tasks = Vec::new();
        let data_dir = data_dir.as_ref();
        let stores_dir = stores_dir.as_ref();

        // Brand folders
        if data_dir.exists() {
            for brand_entry in WalkDir::new(data_dir).min_depth(1).max_depth(1) {
                let Ok(brand_entry) = brand_entry else { continue };
                if !brand_entry.file_type().is_dir() {
                    continue;
                }
                tasks.push((brand_entry.path().to_path_buf(), "brand.json".to_string(), "id".to_string()));

                // Material folders
                for material_entry in WalkDir::new(brand_entry.path()).min_depth(1).max_depth(1) {
                    let Ok(material_entry) = material_entry else { continue };
                    if !material_entry.file_type().is_dir() {
                        continue;
                    }
                    tasks.push((material_entry.path().to_path_buf(), "material.json".to_string(), "material".to_string()));

                    // Filament folders
                    for filament_entry in WalkDir::new(material_entry.path()).min_depth(1).max_depth(1) {
                        let Ok(filament_entry) = filament_entry else { continue };
                        if !filament_entry.file_type().is_dir() {
                            continue;
                        }
                        tasks.push((filament_entry.path().to_path_buf(), "filament.json".to_string(), "id".to_string()));

                        // Variant folders
                        for variant_entry in WalkDir::new(filament_entry.path()).min_depth(1).max_depth(1) {
                            let Ok(variant_entry) = variant_entry else { continue };
                            if !variant_entry.file_type().is_dir() {
                                continue;
                            }
                            tasks.push((variant_entry.path().to_path_buf(), "variant.json".to_string(), "id".to_string()));
                        }
                    }
                }
            }
        }

        // Store folders
        if stores_dir.exists() {
            for store_entry in WalkDir::new(stores_dir).min_depth(1).max_depth(1) {
                let Ok(store_entry) = store_entry else { continue };
                if !store_entry.file_type().is_dir() {
                    continue;
                }
                tasks.push((store_entry.path().to_path_buf(), "store.json".to_string(), "id".to_string()));
            }
        }

        tasks
    }

    pub fn validate_all<P: AsRef<Path>>(&self, data_dir: P, stores_dir: P) -> ValidationResult {
        let tasks = Self::collect_validation_tasks(data_dir, stores_dir);
        let mut result = ValidationResult::new();

        for (folder_path, json_file, json_key) in tasks {
            result.merge(self.validate_folder_name(&folder_path, &json_file, &json_key));
        }

        result
    }
}

impl Default for FolderNameValidator {
    fn default() -> Self {
        Self::new()
    }
}
