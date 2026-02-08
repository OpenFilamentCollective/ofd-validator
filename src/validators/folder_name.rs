use std::path::Path;

use crate::types::{ValidationError, ValidationResult};
use crate::util::{cleanse_folder_name, load_json, ILLEGAL_CHARACTERS};

pub fn validate_folder_name_impl(
    folder_path: &Path,
    json_file: &str,
    json_key: &str,
) -> ValidationResult {
    let mut result = ValidationResult::default();
    let path_str = folder_path.to_string_lossy().to_string();

    let json_path = folder_path.join(json_file);
    if !json_path.exists() {
        result.add(ValidationError::error(
            "Folder",
            format!("Missing {}", json_file),
            Some(path_str),
        ));
        return result;
    }

    let data = match load_json(&json_path) {
        Some(v) => v,
        None => return result,
    };

    let expected_name = match data.get(json_key).and_then(|v| v.as_str()) {
        Some(name) => cleanse_folder_name(name),
        None => return result,
    };

    let actual_name = match folder_path.file_name() {
        Some(name) => name.to_string_lossy().to_string(),
        None => return result,
    };

    if actual_name != expected_name {
        let has_illegal_chars = expected_name.chars().any(|c| ILLEGAL_CHARACTERS.contains(&c));

        if !has_illegal_chars {
            result.add(ValidationError::error(
                "Folder",
                format!(
                    "Folder name '{}' does not match '{}' value '{}' in {}",
                    actual_name, json_key, expected_name, json_file
                ),
                Some(path_str),
            ));
        }
    }

    result
}
