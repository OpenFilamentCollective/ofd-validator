//! Common helper functions for validation
//!
//! This module provides utility functions used across multiple validators,
//! such as JSON loading, path manipulation, and string processing.

use serde_json::Value;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum HelperError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Load JSON from a file with error handling
///
/// Returns Ok(Some(value)) if successful, Ok(None) if file doesn't exist,
/// or Err if there's a parsing error.
pub fn load_json<P: AsRef<Path>>(path: P) -> Result<Option<Value>, HelperError> {
    let path = path.as_ref();

    if !path.exists() {
        return Ok(None);
    }

    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let value: Value = serde_json::from_reader(reader)?;
    Ok(Some(value))
}

/// Clean folder name by replacing slashes and stripping whitespace
///
/// Matches the Python implementation in validators.py:cleanse_folder_name()
pub fn cleanse_folder_name(name: &str) -> String {
    name.replace('/', " ").trim().to_string()
}

/// Get a string value from a JSON object
pub fn get_json_string(value: &Value, key: &str) -> Option<String> {
    value.get(key)?.as_str().map(|s| s.to_string())
}

/// Get a string value from nested JSON path (e.g., "field.subfield")
pub fn get_nested_string(value: &Value, path: &str) -> Option<String> {
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = value;

    for part in &parts[..parts.len() - 1] {
        current = current.get(part)?;
    }

    current.get(parts.last()?)?.as_str().map(|s| s.to_string())
}

/// Check if a file has a specific extension
pub fn has_extension<P: AsRef<Path>>(path: P, extensions: &[&str]) -> bool {
    path.as_ref()
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| extensions.iter().any(|&e| e.eq_ignore_ascii_case(ext)))
        .unwrap_or(false)
}

/// Get the parent directory of a path, returning None if it's the root
pub fn get_parent_dir<P: AsRef<Path>>(path: P) -> Option<std::path::PathBuf> {
    path.as_ref().parent().map(|p| p.to_path_buf())
}

/// Check if a path is within a directory (recursive check)
pub fn is_within_dir<P: AsRef<Path>, D: AsRef<Path>>(path: P, dir: D) -> bool {
    path.as_ref().starts_with(dir.as_ref())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_cleanse_folder_name() {
        assert_eq!(cleanse_folder_name("test/name"), "test name");
        assert_eq!(cleanse_folder_name("  spaces  "), "spaces");
        assert_eq!(cleanse_folder_name("normal"), "normal");
    }

    #[test]
    fn test_load_json() {
        // Create a temporary JSON file
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, r#"{{"key": "value"}}"#).unwrap();

        let result = load_json(temp_file.path()).unwrap();
        assert!(result.is_some());

        let json = result.unwrap();
        assert_eq!(get_json_string(&json, "key"), Some("value".to_string()));
    }

    #[test]
    fn test_load_json_nonexistent() {
        let result = load_json("/nonexistent/file.json").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_get_json_string() {
        let json: Value = serde_json::json!({"name": "test", "count": 42});
        assert_eq!(get_json_string(&json, "name"), Some("test".to_string()));
        assert_eq!(get_json_string(&json, "count"), None); // Not a string
        assert_eq!(get_json_string(&json, "missing"), None);
    }

    #[test]
    fn test_has_extension() {
        assert!(has_extension("/path/to/file.json", &["json"]));
        assert!(has_extension("/path/to/file.JSON", &["json"])); // Case insensitive
        assert!(has_extension("/path/to/file.png", &["png", "jpg"]));
        assert!(!has_extension("/path/to/file.txt", &["json", "png"]));
        assert!(!has_extension("/path/to/file", &["json"]));
    }

    #[test]
    fn test_is_within_dir() {
        assert!(is_within_dir("/a/b/c", "/a/b"));
        assert!(is_within_dir("/a/b/c", "/a"));
        assert!(!is_within_dir("/a/b", "/a/b/c"));
        assert!(!is_within_dir("/x/y", "/a/b"));
    }
}
