mod folder_name;
mod gtin;
mod json_validator;
mod logo_validator;
mod missing_files;
mod store_id;

pub use folder_name::validate_folder_name_impl;
pub use gtin::validate_gtin_ean_impl;
pub use json_validator::validate_json_file_impl;
pub use logo_validator::validate_logo_file_impl;
pub use missing_files::validate_required_files_impl;
pub use store_id::validate_store_ids_impl;

use pyo3::prelude::*;
use std::path::PathBuf;

use crate::types::ValidationResult;

/// Validate GTIN/EAN fields in all sizes.json files.
#[pyfunction]
#[pyo3(signature = (data_dir))]
pub fn validate_gtin_ean(data_dir: &str) -> ValidationResult {
    validate_gtin_ean_impl(&PathBuf::from(data_dir))
}

/// Validate store IDs referenced in purchase links.
#[pyfunction]
#[pyo3(signature = (data_dir, stores_dir))]
pub fn validate_store_ids(data_dir: &str, stores_dir: &str) -> ValidationResult {
    validate_store_ids_impl(&PathBuf::from(data_dir), &PathBuf::from(stores_dir))
}

/// Validate required files exist at each hierarchy level.
#[pyfunction]
#[pyo3(signature = (data_dir, stores_dir))]
pub fn validate_required_files(data_dir: &str, stores_dir: &str) -> ValidationResult {
    validate_required_files_impl(&PathBuf::from(data_dir), &PathBuf::from(stores_dir))
}

/// Validate a single logo file.
#[pyfunction]
#[pyo3(signature = (logo_path, logo_name=None))]
pub fn validate_logo_file(logo_path: &str, logo_name: Option<&str>) -> ValidationResult {
    validate_logo_file_impl(&PathBuf::from(logo_path), logo_name)
}

/// Validate a folder name matches its JSON content.
#[pyfunction]
#[pyo3(signature = (folder_path, json_file, json_key))]
pub fn validate_folder_name(folder_path: &str, json_file: &str, json_key: &str) -> ValidationResult {
    validate_folder_name_impl(&PathBuf::from(folder_path), json_file, json_key)
}
