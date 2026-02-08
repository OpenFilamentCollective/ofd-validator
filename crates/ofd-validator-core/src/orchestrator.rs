use std::collections::HashSet;
use serde_json::Value;
use rayon::prelude::*;

use crate::schema_cache::SchemaCache;
use crate::types::ValidationResult;
use crate::validators;
use crate::validators::missing_files::FileManifest;

/// A pre-loaded dataset ready for validation.
/// All file contents are already in memory — no filesystem access during validation.
pub struct DataSet {
    /// (path_label, schema_name, parsed JSON)
    pub json_entries: Vec<(String, String, Value)>,
    /// (path_label, filename, raw bytes, logo_name from JSON)
    pub logo_entries: Vec<(String, String, Vec<u8>, Option<String>)>,
    /// (path_label, actual_folder_name, json_data from the folder's JSON file, json_key)
    pub folder_entries: Vec<(String, String, Value, String)>,
    /// (path_label, parsed sizes.json)
    pub sizes_entries: Vec<(String, Value)>,
    /// Set of valid store IDs from store.json files
    pub valid_store_ids: HashSet<String>,
    /// File manifest for missing-files validation
    pub file_manifest: FileManifest,
    /// Compiled schema cache
    pub schema_cache: SchemaCache,
}

#[cfg(feature = "filesystem")]
impl DataSet {
    /// Build a DataSet by walking the filesystem.
    pub fn from_directories(
        data_dir: &std::path::Path,
        stores_dir: &std::path::Path,
        schemas_dir: &std::path::Path,
    ) -> Self {
        use crate::util::load_json;
        use walkdir::WalkDir;

        let schema_cache = SchemaCache::from_directory(schemas_dir);
        let file_manifest = validators::missing_files::build_file_manifest(data_dir, stores_dir);

        let mut json_entries = Vec::new();
        let mut logo_entries = Vec::new();
        let mut folder_entries = Vec::new();
        let mut sizes_entries = Vec::new();
        let mut valid_store_ids = HashSet::new();

        // Walk brand hierarchy
        if let Ok(brands) = std::fs::read_dir(data_dir) {
            for brand_entry in brands.filter_map(|e| e.ok()) {
                let brand_dir = brand_entry.path();
                if !brand_dir.is_dir() {
                    continue;
                }

                let brand_file = brand_dir.join("brand.json");
                if brand_file.exists() {
                    if let Some(data) = load_json(&brand_file) {
                        json_entries.push((
                            brand_file.to_string_lossy().to_string(),
                            "brand".to_string(),
                            data.clone(),
                        ));

                        // Logo task from brand.json
                        if let Some(logo_name) = data.get("logo").and_then(|v| v.as_str()) {
                            let logo_path = brand_dir.join(logo_name);
                            if let Ok(bytes) = std::fs::read(&logo_path) {
                                let filename = logo_path.file_name()
                                    .map(|f| f.to_string_lossy().to_string())
                                    .unwrap_or_default();
                                logo_entries.push((
                                    logo_path.to_string_lossy().to_string(),
                                    filename,
                                    bytes,
                                    Some(logo_name.to_string()),
                                ));
                            } else {
                                // Push empty bytes so the validator can report "file not found"
                                logo_entries.push((
                                    logo_path.to_string_lossy().to_string(),
                                    logo_name.to_string(),
                                    Vec::new(),
                                    Some(logo_name.to_string()),
                                ));
                            }
                        }

                        // Folder name task
                        let folder_name = brand_dir.file_name()
                            .map(|f| f.to_string_lossy().to_string())
                            .unwrap_or_default();
                        folder_entries.push((
                            brand_dir.to_string_lossy().to_string(),
                            folder_name,
                            data,
                            "id".to_string(),
                        ));
                    }
                }

                if let Ok(materials) = std::fs::read_dir(&brand_dir) {
                    for material_entry in materials.filter_map(|e| e.ok()) {
                        let material_dir = material_entry.path();
                        if !material_dir.is_dir() {
                            continue;
                        }

                        let material_file = material_dir.join("material.json");
                        if material_file.exists() {
                            if let Some(data) = load_json(&material_file) {
                                json_entries.push((
                                    material_file.to_string_lossy().to_string(),
                                    "material".to_string(),
                                    data.clone(),
                                ));

                                let folder_name = material_dir.file_name()
                                    .map(|f| f.to_string_lossy().to_string())
                                    .unwrap_or_default();
                                folder_entries.push((
                                    material_dir.to_string_lossy().to_string(),
                                    folder_name,
                                    data,
                                    "material".to_string(),
                                ));
                            }
                        }

                        if let Ok(filaments) = std::fs::read_dir(&material_dir) {
                            for filament_entry in filaments.filter_map(|e| e.ok()) {
                                let filament_dir = filament_entry.path();
                                if !filament_dir.is_dir() {
                                    continue;
                                }

                                let filament_file = filament_dir.join("filament.json");
                                if filament_file.exists() {
                                    if let Some(data) = load_json(&filament_file) {
                                        json_entries.push((
                                            filament_file.to_string_lossy().to_string(),
                                            "filament".to_string(),
                                            data.clone(),
                                        ));

                                        let folder_name = filament_dir.file_name()
                                            .map(|f| f.to_string_lossy().to_string())
                                            .unwrap_or_default();
                                        folder_entries.push((
                                            filament_dir.to_string_lossy().to_string(),
                                            folder_name,
                                            data,
                                            "id".to_string(),
                                        ));
                                    }
                                }

                                if let Ok(variants) = std::fs::read_dir(&filament_dir) {
                                    for variant_entry in variants.filter_map(|e| e.ok()) {
                                        let variant_dir = variant_entry.path();
                                        if !variant_dir.is_dir() {
                                            continue;
                                        }

                                        let variant_file = variant_dir.join("variant.json");
                                        if variant_file.exists() {
                                            if let Some(data) = load_json(&variant_file) {
                                                json_entries.push((
                                                    variant_file.to_string_lossy().to_string(),
                                                    "variant".to_string(),
                                                    data.clone(),
                                                ));

                                                let folder_name = variant_dir.file_name()
                                                    .map(|f| f.to_string_lossy().to_string())
                                                    .unwrap_or_default();
                                                folder_entries.push((
                                                    variant_dir.to_string_lossy().to_string(),
                                                    folder_name,
                                                    data,
                                                    "id".to_string(),
                                                ));
                                            }
                                        }

                                        let sizes_file = variant_dir.join("sizes.json");
                                        if sizes_file.exists() {
                                            if let Some(data) = load_json(&sizes_file) {
                                                json_entries.push((
                                                    sizes_file.to_string_lossy().to_string(),
                                                    "sizes".to_string(),
                                                    data.clone(),
                                                ));
                                                sizes_entries.push((
                                                    sizes_file.to_string_lossy().to_string(),
                                                    data,
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
        }

        // Walk stores
        if let Ok(stores) = std::fs::read_dir(stores_dir) {
            for store_entry in stores.filter_map(|e| e.ok()) {
                let store_dir = store_entry.path();
                if !store_dir.is_dir() {
                    continue;
                }

                let store_file = store_dir.join("store.json");
                if store_file.exists() {
                    if let Some(data) = load_json(&store_file) {
                        json_entries.push((
                            store_file.to_string_lossy().to_string(),
                            "store".to_string(),
                            data.clone(),
                        ));

                        if let Some(id) = data.get("id").and_then(|v| v.as_str()) {
                            valid_store_ids.insert(id.to_string());
                        }

                        // Store logo
                        if let Some(logo_name) = data.get("logo").and_then(|v| v.as_str()) {
                            let logo_path = store_dir.join(logo_name);
                            if let Ok(bytes) = std::fs::read(&logo_path) {
                                let filename = logo_path.file_name()
                                    .map(|f| f.to_string_lossy().to_string())
                                    .unwrap_or_default();
                                logo_entries.push((
                                    logo_path.to_string_lossy().to_string(),
                                    filename,
                                    bytes,
                                    Some(logo_name.to_string()),
                                ));
                            } else {
                                logo_entries.push((
                                    logo_path.to_string_lossy().to_string(),
                                    logo_name.to_string(),
                                    Vec::new(),
                                    Some(logo_name.to_string()),
                                ));
                            }
                        }

                        // Store folder name
                        let folder_name = store_dir.file_name()
                            .map(|f| f.to_string_lossy().to_string())
                            .unwrap_or_default();
                        folder_entries.push((
                            store_dir.to_string_lossy().to_string(),
                            folder_name,
                            data,
                            "id".to_string(),
                        ));
                    }
                }
            }
        }

        // Also collect sizes.json via WalkDir for gtin/store_id validators
        // (in case some were missed by the nested loop above)
        // Actually the nested loop above already collects all sizes.json entries,
        // so we don't need WalkDir here. But let's also catch any sizes.json
        // that might exist at unexpected locations.
        for entry in WalkDir::new(data_dir).into_iter().filter_map(|e| e.ok()) {
            if entry.file_name() == "sizes.json" {
                let path_str = entry.path().to_string_lossy().to_string();
                // Only add if not already collected
                if !sizes_entries.iter().any(|(p, _)| p == &path_str) {
                    if let Some(data) = load_json(entry.path()) {
                        sizes_entries.push((path_str, data));
                    }
                }
            }
        }

        DataSet {
            json_entries,
            logo_entries,
            folder_entries,
            sizes_entries,
            valid_store_ids,
            file_manifest,
            schema_cache,
        }
    }
}

/// Run all validations on a pre-loaded DataSet.
pub fn validate_dataset(dataset: &DataSet) -> ValidationResult {
    let mut result = ValidationResult::default();

    // 1. Missing files
    result.merge_from(&validators::validate_required_files(&dataset.file_manifest));

    // 2. JSON schema validation (parallel)
    let json_results: Vec<ValidationResult> = dataset.json_entries
        .par_iter()
        .map(|(path, schema_name, data)| {
            validators::validate_json(data, schema_name, &dataset.schema_cache, Some(path))
        })
        .collect();
    for r in json_results {
        result.merge_from(&r);
    }

    // 3. Logo validation (parallel)
    let logo_results: Vec<ValidationResult> = dataset.logo_entries
        .par_iter()
        .map(|(path, filename, bytes, logo_name)| {
            if bytes.is_empty() {
                // File was not found on disk
                let mut r = ValidationResult::default();
                r.add(crate::types::ValidationError::error(
                    "Logo",
                    "Logo file not found",
                    Some(path.clone()),
                ));
                r
            } else {
                validators::validate_logo(bytes, filename, logo_name.as_deref(), Some(path))
            }
        })
        .collect();
    for r in logo_results {
        result.merge_from(&r);
    }

    // 4. Folder name validation (parallel)
    let folder_results: Vec<ValidationResult> = dataset.folder_entries
        .par_iter()
        .map(|(path, folder_name, json_data, json_key)| {
            validators::validate_folder_name(folder_name, json_data, json_key, Some(path))
        })
        .collect();
    for r in folder_results {
        result.merge_from(&r);
    }

    // 5. Store ID validation
    let sizes_refs: Vec<(&str, &Value)> = dataset.sizes_entries
        .iter()
        .map(|(p, v)| (p.as_str(), v))
        .collect();
    result.merge_from(&validators::validate_store_ids(&dataset.valid_store_ids, &sizes_refs));

    // 6. GTIN/EAN validation
    result.merge_from(&validators::validate_gtin_ean(&sizes_refs));

    result
}
