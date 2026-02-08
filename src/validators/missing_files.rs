use std::path::Path;

use crate::types::{ValidationError, ValidationResult};

pub fn validate_required_files_impl(data_dir: &Path, stores_dir: &Path) -> ValidationResult {
    let mut result = ValidationResult::default();

    // Check brand directories
    if let Ok(brands) = std::fs::read_dir(data_dir) {
        for brand_entry in brands.filter_map(|e| e.ok()) {
            let brand_dir = brand_entry.path();
            if !brand_dir.is_dir() {
                continue;
            }

            let brand_path_str = brand_dir.to_string_lossy().to_string();
            if !brand_dir.join("brand.json").exists() {
                result.add(ValidationError::error(
                    "Missing File",
                    "Missing brand.json",
                    Some(brand_path_str.clone()),
                ));
            }

            // Check material directories
            if let Ok(materials) = std::fs::read_dir(&brand_dir) {
                for material_entry in materials.filter_map(|e| e.ok()) {
                    let material_dir = material_entry.path();
                    if !material_dir.is_dir() {
                        continue;
                    }

                    let material_path_str = material_dir.to_string_lossy().to_string();
                    if !material_dir.join("material.json").exists() {
                        result.add(ValidationError::error(
                            "Missing File",
                            "Missing material.json",
                            Some(material_path_str.clone()),
                        ));
                    }

                    // Check filament directories
                    if let Ok(filaments) = std::fs::read_dir(&material_dir) {
                        for filament_entry in filaments.filter_map(|e| e.ok()) {
                            let filament_dir = filament_entry.path();
                            if !filament_dir.is_dir() {
                                continue;
                            }

                            let filament_path_str = filament_dir.to_string_lossy().to_string();
                            if !filament_dir.join("filament.json").exists() {
                                result.add(ValidationError::error(
                                    "Missing File",
                                    "Missing filament.json",
                                    Some(filament_path_str.clone()),
                                ));
                            }

                            // Check variant directories
                            if let Ok(variants) = std::fs::read_dir(&filament_dir) {
                                for variant_entry in variants.filter_map(|e| e.ok()) {
                                    let variant_dir = variant_entry.path();
                                    if !variant_dir.is_dir() {
                                        continue;
                                    }

                                    let variant_path_str =
                                        variant_dir.to_string_lossy().to_string();

                                    if !variant_dir.join("variant.json").exists() {
                                        result.add(ValidationError::error(
                                            "Missing File",
                                            "Missing variant.json",
                                            Some(variant_path_str.clone()),
                                        ));
                                    }

                                    if !variant_dir.join("sizes.json").exists() {
                                        result.add(ValidationError::error(
                                            "Missing File",
                                            "Missing sizes.json",
                                            Some(variant_path_str.clone()),
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Check store directories
    if let Ok(stores) = std::fs::read_dir(stores_dir) {
        for store_entry in stores.filter_map(|e| e.ok()) {
            let store_dir = store_entry.path();
            if !store_dir.is_dir() {
                continue;
            }

            let store_path_str = store_dir.to_string_lossy().to_string();
            if !store_dir.join("store.json").exists() {
                result.add(ValidationError::error(
                    "Missing File",
                    "Missing store.json",
                    Some(store_path_str),
                ));
            }
        }
    }

    result
}
