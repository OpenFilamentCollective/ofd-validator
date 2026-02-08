use serde_json::Value;

use crate::types::{ValidationError, ValidationResult};
use crate::util::{cleanse_folder_name, ILLEGAL_CHARACTERS};

/// Validate that a folder name matches the expected value from its JSON data.
pub fn validate_folder_name(
    actual_folder_name: &str,
    json_data: &Value,
    json_key: &str,
    path_label: Option<&str>,
) -> ValidationResult {
    let mut result = ValidationResult::default();
    let path_str = path_label.map(|s| s.to_string());

    let expected_name = match json_data.get(json_key).and_then(|v| v.as_str()) {
        Some(name) => cleanse_folder_name(name),
        None => return result,
    };

    if actual_folder_name != expected_name {
        let has_illegal_chars = expected_name.chars().any(|c| ILLEGAL_CHARACTERS.contains(&c));

        if !has_illegal_chars {
            result.add(ValidationError::error(
                "Folder",
                format!(
                    "Folder name '{}' does not match '{}' value '{}' in JSON",
                    actual_folder_name, json_key, expected_name
                ),
                path_str,
            ));
        }
    }

    result
}
