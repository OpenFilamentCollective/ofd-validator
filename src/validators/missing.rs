//! Missing file validator
//!
//! Validates that required JSON files exist in the directory structure

use crate::types::{ValidationError, ValidationLevel, ValidationResult};
use std::path::Path;
use walkdir::WalkDir;

pub struct MissingFileValidator;

impl MissingFileValidator {
    pub fn new() -> Self {
        Self
    }

    pub fn validate_required_files<P: AsRef<Path>>(
        &self,
        data_dir: P,
        stores_dir: P,
    ) -> ValidationResult {
        let data_dir = data_dir.as_ref();
        let stores_dir = stores_dir.as_ref();
        let mut result = ValidationResult::new();

        // Check brand directories
        if data_dir.exists() {
            for brand_entry in WalkDir::new(data_dir).min_depth(1).max_depth(1) {
                let Ok(brand_entry) = brand_entry else { continue };
                if !brand_entry.file_type().is_dir() {
                    continue;
                }
                let brand_dir = brand_entry.path();

                if !brand_dir.join("brand.json").exists() {
                    result.add_error(ValidationError::with_path(
                        ValidationLevel::Error,
                        "Missing File",
                        "Missing brand.json",
                        brand_dir,
                    ));
                }

                // Check material directories
                for material_entry in WalkDir::new(brand_dir).min_depth(1).max_depth(1) {
                    let Ok(material_entry) = material_entry else { continue };
                    if !material_entry.file_type().is_dir() {
                        continue;
                    }
                    let material_dir = material_entry.path();

                    if !material_dir.join("material.json").exists() {
                        result.add_error(ValidationError::with_path(
                            ValidationLevel::Error,
                            "Missing File",
                            "Missing material.json",
                            material_dir,
                        ));
                    }

                    // Check filament directories
                    for filament_entry in WalkDir::new(material_dir).min_depth(1).max_depth(1) {
                        let Ok(filament_entry) = filament_entry else { continue };
                        if !filament_entry.file_type().is_dir() {
                            continue;
                        }
                        let filament_dir = filament_entry.path();

                        if !filament_dir.join("filament.json").exists() {
                            result.add_error(ValidationError::with_path(
                                ValidationLevel::Error,
                                "Missing File",
                                "Missing filament.json",
                                filament_dir,
                            ));
                        }

                        // Check variant directories
                        for variant_entry in WalkDir::new(filament_dir).min_depth(1).max_depth(1) {
                            let Ok(variant_entry) = variant_entry else { continue };
                            if !variant_entry.file_type().is_dir() {
                                continue;
                            }
                            let variant_dir = variant_entry.path();

                            if !variant_dir.join("variant.json").exists() {
                                result.add_error(ValidationError::with_path(
                                    ValidationLevel::Error,
                                    "Missing File",
                                    "Missing variant.json",
                                    variant_dir,
                                ));
                            }

                            if !variant_dir.join("sizes.json").exists() {
                                result.add_error(ValidationError::with_path(
                                    ValidationLevel::Error,
                                    "Missing File",
                                    "Missing sizes.json",
                                    variant_dir,
                                ));
                            }
                        }
                    }
                }
            }
        }

        // Check store directories
        if stores_dir.exists() {
            for store_entry in WalkDir::new(stores_dir).min_depth(1).max_depth(1) {
                let Ok(store_entry) = store_entry else { continue };
                if !store_entry.file_type().is_dir() {
                    continue;
                }
                let store_dir = store_entry.path();

                if !store_dir.join("store.json").exists() {
                    result.add_error(ValidationError::with_path(
                        ValidationLevel::Error,
                        "Missing File",
                        "Missing store.json",
                        store_dir,
                    ));
                }
            }
        }

        result
    }
}

impl Default for MissingFileValidator {
    fn default() -> Self {
        Self::new()
    }
}
