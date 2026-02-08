use std::path::{Path, PathBuf};

use pyo3::prelude::*;
use rayon::prelude::*;

use crate::schema_cache::SchemaCache;
use crate::types::ValidationResult;
use crate::util::load_json;
use crate::validators::{
    validate_folder_name_impl, validate_gtin_ean_impl, validate_json_file_impl,
    validate_logo_file_impl, validate_required_files_impl, validate_store_ids_impl,
};

// ---- Task collection helpers ----

struct JsonTask {
    path: PathBuf,
    schema_name: String,
}

struct LogoTask {
    path: PathBuf,
    logo_name: Option<String>,
}

struct FolderTask {
    path: PathBuf,
    json_file: String,
    json_key: String,
}

fn collect_json_tasks(data_dir: &Path, stores_dir: &Path) -> Vec<JsonTask> {
    let mut tasks = Vec::new();

    if let Ok(brands) = std::fs::read_dir(data_dir) {
        for brand_entry in brands.filter_map(|e| e.ok()) {
            let brand_dir = brand_entry.path();
            if !brand_dir.is_dir() {
                continue;
            }

            let brand_file = brand_dir.join("brand.json");
            if brand_file.exists() {
                tasks.push(JsonTask {
                    path: brand_file,
                    schema_name: "brand".to_string(),
                });
            }

            if let Ok(materials) = std::fs::read_dir(&brand_dir) {
                for material_entry in materials.filter_map(|e| e.ok()) {
                    let material_dir = material_entry.path();
                    if !material_dir.is_dir() {
                        continue;
                    }

                    let material_file = material_dir.join("material.json");
                    if material_file.exists() {
                        tasks.push(JsonTask {
                            path: material_file,
                            schema_name: "material".to_string(),
                        });
                    }

                    if let Ok(filaments) = std::fs::read_dir(&material_dir) {
                        for filament_entry in filaments.filter_map(|e| e.ok()) {
                            let filament_dir = filament_entry.path();
                            if !filament_dir.is_dir() {
                                continue;
                            }

                            let filament_file = filament_dir.join("filament.json");
                            if filament_file.exists() {
                                tasks.push(JsonTask {
                                    path: filament_file,
                                    schema_name: "filament".to_string(),
                                });
                            }

                            if let Ok(variants) = std::fs::read_dir(&filament_dir) {
                                for variant_entry in variants.filter_map(|e| e.ok()) {
                                    let variant_dir = variant_entry.path();
                                    if !variant_dir.is_dir() {
                                        continue;
                                    }

                                    let variant_file = variant_dir.join("variant.json");
                                    if variant_file.exists() {
                                        tasks.push(JsonTask {
                                            path: variant_file,
                                            schema_name: "variant".to_string(),
                                        });
                                    }

                                    let sizes_file = variant_dir.join("sizes.json");
                                    if sizes_file.exists() {
                                        tasks.push(JsonTask {
                                            path: sizes_file,
                                            schema_name: "sizes".to_string(),
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Store validation tasks
    if let Ok(stores) = std::fs::read_dir(stores_dir) {
        for store_entry in stores.filter_map(|e| e.ok()) {
            let store_dir = store_entry.path();
            if !store_dir.is_dir() {
                continue;
            }

            let store_file = store_dir.join("store.json");
            if store_file.exists() {
                tasks.push(JsonTask {
                    path: store_file,
                    schema_name: "store".to_string(),
                });
            }
        }
    }

    tasks
}

fn collect_logo_tasks(data_dir: &Path, stores_dir: &Path) -> Vec<LogoTask> {
    let mut tasks = Vec::new();

    // Brand logos
    if let Ok(brands) = std::fs::read_dir(data_dir) {
        for brand_entry in brands.filter_map(|e| e.ok()) {
            let brand_dir = brand_entry.path();
            if !brand_dir.is_dir() {
                continue;
            }

            let brand_file = brand_dir.join("brand.json");
            if brand_file.exists() {
                if let Some(data) = load_json(&brand_file) {
                    if let Some(logo_name) = data.get("logo").and_then(|v| v.as_str()) {
                        let logo_path = brand_dir.join(logo_name);
                        tasks.push(LogoTask {
                            path: logo_path,
                            logo_name: Some(logo_name.to_string()),
                        });
                    }
                }
            }
        }
    }

    // Store logos
    if let Ok(stores) = std::fs::read_dir(stores_dir) {
        for store_entry in stores.filter_map(|e| e.ok()) {
            let store_dir = store_entry.path();
            if !store_dir.is_dir() {
                continue;
            }

            let store_file = store_dir.join("store.json");
            if store_file.exists() {
                if let Some(data) = load_json(&store_file) {
                    if let Some(logo_name) = data.get("logo").and_then(|v| v.as_str()) {
                        let logo_path = store_dir.join(logo_name);
                        tasks.push(LogoTask {
                            path: logo_path,
                            logo_name: Some(logo_name.to_string()),
                        });
                    }
                }
            }
        }
    }

    tasks
}

fn collect_folder_tasks(data_dir: &Path, stores_dir: &Path) -> Vec<FolderTask> {
    let mut tasks = Vec::new();

    if let Ok(brands) = std::fs::read_dir(data_dir) {
        for brand_entry in brands.filter_map(|e| e.ok()) {
            let brand_dir = brand_entry.path();
            if !brand_dir.is_dir() {
                continue;
            }

            tasks.push(FolderTask {
                path: brand_dir.clone(),
                json_file: "brand.json".to_string(),
                json_key: "id".to_string(),
            });

            if let Ok(materials) = std::fs::read_dir(&brand_dir) {
                for material_entry in materials.filter_map(|e| e.ok()) {
                    let material_dir = material_entry.path();
                    if !material_dir.is_dir() {
                        continue;
                    }

                    tasks.push(FolderTask {
                        path: material_dir.clone(),
                        json_file: "material.json".to_string(),
                        json_key: "material".to_string(),
                    });

                    if let Ok(filaments) = std::fs::read_dir(&material_dir) {
                        for filament_entry in filaments.filter_map(|e| e.ok()) {
                            let filament_dir = filament_entry.path();
                            if !filament_dir.is_dir() {
                                continue;
                            }

                            tasks.push(FolderTask {
                                path: filament_dir.clone(),
                                json_file: "filament.json".to_string(),
                                json_key: "id".to_string(),
                            });

                            if let Ok(variants) = std::fs::read_dir(&filament_dir) {
                                for variant_entry in variants.filter_map(|e| e.ok()) {
                                    let variant_dir = variant_entry.path();
                                    if !variant_dir.is_dir() {
                                        continue;
                                    }

                                    tasks.push(FolderTask {
                                        path: variant_dir,
                                        json_file: "variant.json".to_string(),
                                        json_key: "id".to_string(),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Store folders
    if let Ok(stores) = std::fs::read_dir(stores_dir) {
        for store_entry in stores.filter_map(|e| e.ok()) {
            let store_dir = store_entry.path();
            if !store_dir.is_dir() {
                continue;
            }

            tasks.push(FolderTask {
                path: store_dir,
                json_file: "store.json".to_string(),
                json_key: "id".to_string(),
            });
        }
    }

    tasks
}

// ---- Orchestrated batch validators (exposed to Python) ----

#[pyfunction]
#[pyo3(signature = (data_dir, stores_dir, schemas_dir=None))]
pub fn validate_all(
    py: Python<'_>,
    data_dir: &str,
    stores_dir: &str,
    schemas_dir: Option<&str>,
) -> ValidationResult {
    let data_dir = PathBuf::from(data_dir);
    let stores_dir = PathBuf::from(stores_dir);
    let schemas_dir = PathBuf::from(schemas_dir.unwrap_or("schemas"));

    py.allow_threads(|| {
        let mut result = ValidationResult::default();

        // 1. Missing files check
        eprintln!("Checking for missing required files...");
        result.merge_from(&validate_required_files_impl(&data_dir, &stores_dir));

        // 2. JSON validation (parallel)
        let schema_cache = SchemaCache::new(&schemas_dir);
        let json_tasks = collect_json_tasks(&data_dir, &stores_dir);
        eprintln!("Validating {} JSON files...", json_tasks.len());
        let json_results: Vec<ValidationResult> = json_tasks
            .par_iter()
            .map(|task| validate_json_file_impl(&task.path, &task.schema_name, &schema_cache))
            .collect();
        for r in json_results {
            result.merge_from(&r);
        }

        // 3. Logo validation (parallel)
        let logo_tasks = collect_logo_tasks(&data_dir, &stores_dir);
        eprintln!("Validating {} logo files...", logo_tasks.len());
        let logo_results: Vec<ValidationResult> = logo_tasks
            .par_iter()
            .map(|task| validate_logo_file_impl(&task.path, task.logo_name.as_deref()))
            .collect();
        for r in logo_results {
            result.merge_from(&r);
        }

        // 4. Folder name validation (parallel)
        let folder_tasks = collect_folder_tasks(&data_dir, &stores_dir);
        eprintln!("Validating {} folder names...", folder_tasks.len());
        let folder_results: Vec<ValidationResult> = folder_tasks
            .par_iter()
            .map(|task| validate_folder_name_impl(&task.path, &task.json_file, &task.json_key))
            .collect();
        for r in folder_results {
            result.merge_from(&r);
        }

        // 5. Store ID validation
        eprintln!("Validating store IDs...");
        result.merge_from(&validate_store_ids_impl(&data_dir, &stores_dir));

        // 6. GTIN/EAN validation
        eprintln!("Validating GTIN/EAN codes...");
        result.merge_from(&validate_gtin_ean_impl(&data_dir));

        result
    })
}

#[pyfunction]
#[pyo3(signature = (data_dir, stores_dir, schemas_dir=None))]
pub fn validate_json_files(
    py: Python<'_>,
    data_dir: &str,
    stores_dir: &str,
    schemas_dir: Option<&str>,
) -> ValidationResult {
    let data_dir = PathBuf::from(data_dir);
    let stores_dir = PathBuf::from(stores_dir);
    let schemas_dir = PathBuf::from(schemas_dir.unwrap_or("schemas"));

    py.allow_threads(|| {
        let schema_cache = SchemaCache::new(&schemas_dir);
        let tasks = collect_json_tasks(&data_dir, &stores_dir);
        eprintln!("Validating {} JSON files...", tasks.len());
        let results: Vec<ValidationResult> = tasks
            .par_iter()
            .map(|task| validate_json_file_impl(&task.path, &task.schema_name, &schema_cache))
            .collect();

        let mut result = ValidationResult::default();
        for r in results {
            result.merge_from(&r);
        }
        result
    })
}

#[pyfunction]
#[pyo3(signature = (data_dir, stores_dir))]
pub fn validate_logo_files(
    py: Python<'_>,
    data_dir: &str,
    stores_dir: &str,
) -> ValidationResult {
    let data_dir = PathBuf::from(data_dir);
    let stores_dir = PathBuf::from(stores_dir);

    py.allow_threads(|| {
        let tasks = collect_logo_tasks(&data_dir, &stores_dir);
        eprintln!("Validating {} logo files...", tasks.len());
        let results: Vec<ValidationResult> = tasks
            .par_iter()
            .map(|task| validate_logo_file_impl(&task.path, task.logo_name.as_deref()))
            .collect();

        let mut result = ValidationResult::default();
        for r in results {
            result.merge_from(&r);
        }
        result
    })
}

#[pyfunction]
#[pyo3(signature = (data_dir, stores_dir))]
pub fn validate_folder_names(
    py: Python<'_>,
    data_dir: &str,
    stores_dir: &str,
) -> ValidationResult {
    let data_dir = PathBuf::from(data_dir);
    let stores_dir = PathBuf::from(stores_dir);

    py.allow_threads(|| {
        let tasks = collect_folder_tasks(&data_dir, &stores_dir);
        eprintln!("Validating {} folder names...", tasks.len());
        let results: Vec<ValidationResult> = tasks
            .par_iter()
            .map(|task| validate_folder_name_impl(&task.path, &task.json_file, &task.json_key))
            .collect();

        let mut result = ValidationResult::default();
        for r in results {
            result.merge_from(&r);
        }
        result
    })
}
