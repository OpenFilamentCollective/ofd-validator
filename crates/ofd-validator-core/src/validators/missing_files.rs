use crate::types::{ValidationError, ValidationResult};

/// Describes the file manifest for a variant directory.
pub struct VariantEntry {
    pub path: String,
    pub has_variant_json: bool,
    pub has_sizes_json: bool,
}

/// Describes the file manifest for a filament directory.
pub struct FilamentEntry {
    pub path: String,
    pub has_filament_json: bool,
    pub variants: Vec<VariantEntry>,
}

/// Describes the file manifest for a material directory.
pub struct MaterialEntry {
    pub path: String,
    pub has_material_json: bool,
    pub filaments: Vec<FilamentEntry>,
}

/// Describes the file manifest for a brand directory.
pub struct BrandEntry {
    pub path: String,
    pub has_brand_json: bool,
    pub materials: Vec<MaterialEntry>,
}

/// Describes the file manifest for a store directory.
pub struct StoreEntry {
    pub path: String,
    pub has_store_json: bool,
}

/// Complete file manifest for the dataset.
pub struct FileManifest {
    pub brands: Vec<BrandEntry>,
    pub stores: Vec<StoreEntry>,
}

/// Build a FileManifest by walking the filesystem.
#[cfg(feature = "filesystem")]
pub fn build_file_manifest(data_dir: &std::path::Path, stores_dir: &std::path::Path) -> FileManifest {
    let mut brands = Vec::new();

    if let Ok(brand_entries) = std::fs::read_dir(data_dir) {
        for brand_entry in brand_entries.filter_map(|e| e.ok()) {
            let brand_dir = brand_entry.path();
            if !brand_dir.is_dir() {
                continue;
            }

            let mut materials = Vec::new();

            if let Ok(material_entries) = std::fs::read_dir(&brand_dir) {
                for material_entry in material_entries.filter_map(|e| e.ok()) {
                    let material_dir = material_entry.path();
                    if !material_dir.is_dir() {
                        continue;
                    }

                    let mut filaments = Vec::new();

                    if let Ok(filament_entries) = std::fs::read_dir(&material_dir) {
                        for filament_entry in filament_entries.filter_map(|e| e.ok()) {
                            let filament_dir = filament_entry.path();
                            if !filament_dir.is_dir() {
                                continue;
                            }

                            let mut variants = Vec::new();

                            if let Ok(variant_entries) = std::fs::read_dir(&filament_dir) {
                                for variant_entry in variant_entries.filter_map(|e| e.ok()) {
                                    let variant_dir = variant_entry.path();
                                    if !variant_dir.is_dir() {
                                        continue;
                                    }

                                    variants.push(VariantEntry {
                                        path: variant_dir.to_string_lossy().to_string(),
                                        has_variant_json: variant_dir.join("variant.json").exists(),
                                        has_sizes_json: variant_dir.join("sizes.json").exists(),
                                    });
                                }
                            }

                            filaments.push(FilamentEntry {
                                path: filament_dir.to_string_lossy().to_string(),
                                has_filament_json: filament_dir.join("filament.json").exists(),
                                variants,
                            });
                        }
                    }

                    materials.push(MaterialEntry {
                        path: material_dir.to_string_lossy().to_string(),
                        has_material_json: material_dir.join("material.json").exists(),
                        filaments,
                    });
                }
            }

            brands.push(BrandEntry {
                path: brand_dir.to_string_lossy().to_string(),
                has_brand_json: brand_dir.join("brand.json").exists(),
                materials,
            });
        }
    }

    let mut stores = Vec::new();
    if let Ok(store_entries) = std::fs::read_dir(stores_dir) {
        for store_entry in store_entries.filter_map(|e| e.ok()) {
            let store_dir = store_entry.path();
            if !store_dir.is_dir() {
                continue;
            }

            stores.push(StoreEntry {
                path: store_dir.to_string_lossy().to_string(),
                has_store_json: store_dir.join("store.json").exists(),
            });
        }
    }

    FileManifest { brands, stores }
}

/// Validate required files exist based on the file manifest.
pub fn validate_required_files(manifest: &FileManifest) -> ValidationResult {
    let mut result = ValidationResult::default();

    for brand in &manifest.brands {
        if !brand.has_brand_json {
            result.add(ValidationError::error(
                "Missing File",
                "Missing brand.json",
                Some(brand.path.clone()),
            ));
        }

        for material in &brand.materials {
            if !material.has_material_json {
                result.add(ValidationError::error(
                    "Missing File",
                    "Missing material.json",
                    Some(material.path.clone()),
                ));
            }

            for filament in &material.filaments {
                if !filament.has_filament_json {
                    result.add(ValidationError::error(
                        "Missing File",
                        "Missing filament.json",
                        Some(filament.path.clone()),
                    ));
                }

                for variant in &filament.variants {
                    if !variant.has_variant_json {
                        result.add(ValidationError::error(
                            "Missing File",
                            "Missing variant.json",
                            Some(variant.path.clone()),
                        ));
                    }

                    if !variant.has_sizes_json {
                        result.add(ValidationError::error(
                            "Missing File",
                            "Missing sizes.json",
                            Some(variant.path.clone()),
                        ));
                    }
                }
            }
        }
    }

    for store in &manifest.stores {
        if !store.has_store_json {
            result.add(ValidationError::error(
                "Missing File",
                "Missing store.json",
                Some(store.path.clone()),
            ));
        }
    }

    result
}
