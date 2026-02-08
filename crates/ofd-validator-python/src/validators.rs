use std::path::PathBuf;

use pyo3::prelude::*;

use ofd_validator_core as core;

use crate::types::ValidationResult;

/// Validate GTIN/EAN fields in all sizes.json files.
#[pyfunction]
#[pyo3(signature = (data_dir))]
pub fn validate_gtin_ean(data_dir: &str) -> ValidationResult {
    use walkdir::WalkDir;

    let data_path = PathBuf::from(data_dir);
    let mut sizes_entries = Vec::new();

    for entry in WalkDir::new(&data_path).into_iter().filter_map(|e| e.ok()) {
        if entry.file_name() != "sizes.json" {
            continue;
        }
        if let Some(data) = core::util::load_json(entry.path()) {
            sizes_entries.push((entry.path().to_string_lossy().to_string(), data));
        }
    }

    let refs: Vec<(&str, &serde_json::Value)> = sizes_entries
        .iter()
        .map(|(p, v)| (p.as_str(), v))
        .collect();

    core::validators::validate_gtin_ean(&refs).into()
}

/// Validate store IDs referenced in purchase links.
#[pyfunction]
#[pyo3(signature = (data_dir, stores_dir))]
pub fn validate_store_ids(data_dir: &str, stores_dir: &str) -> ValidationResult {
    use std::collections::HashSet;
    use walkdir::WalkDir;

    let stores_path = PathBuf::from(stores_dir);
    let data_path = PathBuf::from(data_dir);

    // Collect valid store IDs
    let mut valid_store_ids = HashSet::new();
    if let Ok(entries) = std::fs::read_dir(&stores_path) {
        for entry in entries.filter_map(|e| e.ok()) {
            let store_dir = entry.path();
            if !store_dir.is_dir() {
                continue;
            }
            let store_file = store_dir.join("store.json");
            if let Some(data) = core::util::load_json(&store_file) {
                if let Some(id) = data.get("id").and_then(|v| v.as_str()) {
                    valid_store_ids.insert(id.to_string());
                }
            }
        }
    }

    // Collect sizes entries
    let mut sizes_entries = Vec::new();
    for entry in WalkDir::new(&data_path).into_iter().filter_map(|e| e.ok()) {
        if entry.file_name() != "sizes.json" {
            continue;
        }
        if let Some(data) = core::util::load_json(entry.path()) {
            sizes_entries.push((entry.path().to_string_lossy().to_string(), data));
        }
    }

    let refs: Vec<(&str, &serde_json::Value)> = sizes_entries
        .iter()
        .map(|(p, v)| (p.as_str(), v))
        .collect();

    core::validators::validate_store_ids(&valid_store_ids, &refs).into()
}

/// Validate required files exist at each hierarchy level.
#[pyfunction]
#[pyo3(signature = (data_dir, stores_dir))]
pub fn validate_required_files(data_dir: &str, stores_dir: &str) -> ValidationResult {
    let manifest = core::validators::missing_files::build_file_manifest(
        &PathBuf::from(data_dir),
        &PathBuf::from(stores_dir),
    );
    core::validators::validate_required_files(&manifest).into()
}

/// Validate a single logo file.
#[pyfunction]
#[pyo3(signature = (logo_path, logo_name=None))]
pub fn validate_logo_file(logo_path: &str, logo_name: Option<&str>) -> ValidationResult {
    let path = PathBuf::from(logo_path);

    if !path.exists() {
        let mut result = core::ValidationResult::default();
        result.add(core::ValidationError::error(
            "Logo",
            "Logo file not found",
            Some(logo_path.to_string()),
        ));
        return result.into();
    }

    let filename = path.file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_default();

    match std::fs::read(&path) {
        Ok(bytes) => {
            core::validators::validate_logo(&bytes, &filename, logo_name, Some(logo_path)).into()
        }
        Err(e) => {
            let mut result = core::ValidationResult::default();
            result.add(core::ValidationError::error(
                "Logo",
                format!("Failed to read logo file: {}", e),
                Some(logo_path.to_string()),
            ));
            result.into()
        }
    }
}

/// Validate a folder name matches its JSON content.
#[pyfunction]
#[pyo3(signature = (folder_path, json_file, json_key))]
pub fn validate_folder_name(folder_path: &str, json_file: &str, json_key: &str) -> ValidationResult {
    let folder = PathBuf::from(folder_path);
    let json_path = folder.join(json_file);

    if !json_path.exists() {
        let mut result = core::ValidationResult::default();
        result.add(core::ValidationError::error(
            "Folder",
            format!("Missing {}", json_file),
            Some(folder_path.to_string()),
        ));
        return result.into();
    }

    let data = match core::util::load_json(&json_path) {
        Some(v) => v,
        None => return ValidationResult::default(),
    };

    let actual_name = folder.file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_default();

    core::validators::validate_folder_name(&actual_name, &data, json_key, Some(folder_path)).into()
}
