use std::collections::HashMap;
use std::path::PathBuf;

use napi::bindgen_prelude::*;
use napi_derive::napi;

use ofd_validator_core as core;

// ---- Result types ----

#[napi(object)]
pub struct ValidationError {
    pub level: String,
    pub category: String,
    pub message: String,
    pub path: Option<String>,
}

#[napi(object)]
pub struct ValidationResult {
    pub errors: Vec<ValidationError>,
    pub is_valid: bool,
    pub error_count: u32,
    pub warning_count: u32,
}

impl From<core::ValidationResult> for ValidationResult {
    fn from(r: core::ValidationResult) -> Self {
        ValidationResult {
            is_valid: r.is_valid(),
            error_count: r.error_count() as u32,
            warning_count: r.warning_count() as u32,
            errors: r.errors.into_iter().map(|e| ValidationError {
                level: e.level.to_string(),
                category: e.category,
                message: e.message,
                path: e.path,
            }).collect(),
        }
    }
}

// ---- Path mode (filesystem-based, mirrors Python API) ----

#[napi]
pub fn validate_all(
    data_dir: String,
    stores_dir: String,
    schemas_dir: Option<String>,
) -> ValidationResult {
    let schemas = PathBuf::from(schemas_dir.as_deref().unwrap_or("schemas"));
    let dataset = core::DataSet::from_directories(
        &PathBuf::from(&data_dir),
        &PathBuf::from(&stores_dir),
        &schemas,
    );
    core::validate_dataset(&dataset).into()
}

#[napi]
pub fn validate_json_files(
    data_dir: String,
    stores_dir: String,
    schemas_dir: Option<String>,
) -> ValidationResult {
    use rayon::prelude::*;

    let schemas = PathBuf::from(schemas_dir.as_deref().unwrap_or("schemas"));
    let dataset = core::DataSet::from_directories(
        &PathBuf::from(&data_dir),
        &PathBuf::from(&stores_dir),
        &schemas,
    );

    let results: Vec<core::ValidationResult> = dataset.json_entries
        .par_iter()
        .map(|(path, schema_name, data)| {
            core::validators::validate_json(data, schema_name, &dataset.schema_cache, Some(path))
        })
        .collect();

    let mut result = core::ValidationResult::default();
    for r in results {
        result.merge_from(&r);
    }
    result.into()
}

#[napi]
pub fn validate_logo_files(
    data_dir: String,
    stores_dir: String,
) -> ValidationResult {
    use rayon::prelude::*;

    let schemas = PathBuf::from("schemas");
    let dataset = core::DataSet::from_directories(
        &PathBuf::from(&data_dir),
        &PathBuf::from(&stores_dir),
        &schemas,
    );

    let results: Vec<core::ValidationResult> = dataset.logo_entries
        .par_iter()
        .map(|(path, filename, bytes, logo_name)| {
            if bytes.is_empty() {
                let mut r = core::ValidationResult::default();
                r.add(core::ValidationError::error("Logo", "Logo file not found", Some(path.clone())));
                r
            } else {
                core::validators::validate_logo(bytes, filename, logo_name.as_deref(), Some(path))
            }
        })
        .collect();

    let mut result = core::ValidationResult::default();
    for r in results {
        result.merge_from(&r);
    }
    result.into()
}

#[napi]
pub fn validate_folder_names(
    data_dir: String,
    stores_dir: String,
) -> ValidationResult {
    use rayon::prelude::*;

    let schemas = PathBuf::from("schemas");
    let dataset = core::DataSet::from_directories(
        &PathBuf::from(&data_dir),
        &PathBuf::from(&stores_dir),
        &schemas,
    );

    let results: Vec<core::ValidationResult> = dataset.folder_entries
        .par_iter()
        .map(|(path, folder_name, json_data, json_key)| {
            core::validators::validate_folder_name(folder_name, json_data, json_key, Some(path))
        })
        .collect();

    let mut result = core::ValidationResult::default();
    for r in results {
        result.merge_from(&r);
    }
    result.into()
}

#[napi]
pub fn validate_store_ids(
    data_dir: String,
    stores_dir: String,
) -> ValidationResult {
    use std::collections::HashSet;
    use walkdir::WalkDir;

    let stores_path = PathBuf::from(&stores_dir);
    let data_path = PathBuf::from(&data_dir);

    let mut valid_store_ids = HashSet::new();
    if let Ok(entries) = std::fs::read_dir(&stores_path) {
        for entry in entries.filter_map(|e| e.ok()) {
            let store_dir = entry.path();
            if !store_dir.is_dir() { continue; }
            let store_file = store_dir.join("store.json");
            if let Some(data) = core::util::load_json(&store_file) {
                if let Some(id) = data.get("id").and_then(|v| v.as_str()) {
                    valid_store_ids.insert(id.to_string());
                }
            }
        }
    }

    let mut sizes_entries = Vec::new();
    for entry in WalkDir::new(&data_path).into_iter().filter_map(|e| e.ok()) {
        if entry.file_name() != "sizes.json" { continue; }
        if let Some(data) = core::util::load_json(entry.path()) {
            sizes_entries.push((entry.path().to_string_lossy().to_string(), data));
        }
    }

    let refs: Vec<(&str, &serde_json::Value)> = sizes_entries.iter().map(|(p, v)| (p.as_str(), v)).collect();
    core::validators::validate_store_ids(&valid_store_ids, &refs).into()
}

#[napi]
pub fn validate_gtin_ean(data_dir: String) -> ValidationResult {
    use walkdir::WalkDir;

    let data_path = PathBuf::from(&data_dir);
    let mut sizes_entries = Vec::new();
    for entry in WalkDir::new(&data_path).into_iter().filter_map(|e| e.ok()) {
        if entry.file_name() != "sizes.json" { continue; }
        if let Some(data) = core::util::load_json(entry.path()) {
            sizes_entries.push((entry.path().to_string_lossy().to_string(), data));
        }
    }

    let refs: Vec<(&str, &serde_json::Value)> = sizes_entries.iter().map(|(p, v)| (p.as_str(), v)).collect();
    core::validators::validate_gtin_ean(&refs).into()
}

#[napi]
pub fn validate_required_files(
    data_dir: String,
    stores_dir: String,
) -> ValidationResult {
    let manifest = core::validators::missing_files::build_file_manifest(
        &PathBuf::from(&data_dir),
        &PathBuf::from(&stores_dir),
    );
    core::validators::validate_required_files(&manifest).into()
}

#[napi]
pub fn validate_logo_file(
    logo_path: String,
    logo_name: Option<String>,
) -> ValidationResult {
    let path = PathBuf::from(&logo_path);

    if !path.exists() {
        let mut result = core::ValidationResult::default();
        result.add(core::ValidationError::error("Logo", "Logo file not found", Some(logo_path)));
        return result.into();
    }

    let filename = path.file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_default();

    match std::fs::read(&path) {
        Ok(bytes) => {
            core::validators::validate_logo(&bytes, &filename, logo_name.as_deref(), Some(&logo_path)).into()
        }
        Err(e) => {
            let mut result = core::ValidationResult::default();
            result.add(core::ValidationError::error("Logo", format!("Failed to read logo file: {}", e), Some(logo_path)));
            result.into()
        }
    }
}

#[napi]
pub fn validate_folder_name(
    folder_path: String,
    json_file: String,
    json_key: String,
) -> ValidationResult {
    let folder = PathBuf::from(&folder_path);
    let json_path = folder.join(&json_file);

    if !json_path.exists() {
        let mut result = core::ValidationResult::default();
        result.add(core::ValidationError::error("Folder", format!("Missing {}", json_file), Some(folder_path)));
        return result.into();
    }

    let data = match core::util::load_json(&json_path) {
        Some(v) => v,
        None => return core::ValidationResult::default().into(),
    };

    let actual_name = folder.file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_default();

    core::validators::validate_folder_name(&actual_name, &data, &json_key, Some(&folder_path)).into()
}

// ---- String/Content mode (in-memory, no filesystem access) ----

#[napi(object)]
pub struct JsonFileInput {
    pub path: String,
    pub schema_name: String,
    pub content: String,
}

#[napi(object)]
pub struct LogoFileInput {
    pub path: String,
    pub filename: String,
    pub content: Buffer,
}

#[napi(object)]
pub struct FolderInput {
    pub path: String,
    pub folder_name: String,
    pub json_content: String,
    pub json_key: String,
}

#[napi(object)]
pub struct SizesInput {
    pub path: String,
    pub content: String,
}

#[napi(object)]
pub struct ValidateAllContentInput {
    pub json_files: Vec<JsonFileInput>,
    pub logo_files: Vec<LogoFileInput>,
    pub folders: Vec<FolderInput>,
    pub store_ids: Vec<String>,
    pub schemas: HashMap<String, String>,
}

#[napi]
pub fn validate_json_content(
    content: String,
    schema_name: String,
    schemas: HashMap<String, String>,
    file_path: Option<String>,
) -> Result<ValidationResult> {
    let data: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| Error::from_reason(format!("Invalid JSON: {}", e)))?;

    let schema_values: HashMap<String, serde_json::Value> = schemas.into_iter()
        .map(|(k, v)| {
            let val = serde_json::from_str(&v)
                .map_err(|e| Error::from_reason(format!("Invalid schema JSON for '{}': {}", k, e)));
            val.map(|v| (k, v))
        })
        .collect::<Result<_>>()?;

    let cache = core::SchemaCache::from_map(schema_values);
    Ok(core::validators::validate_json(&data, &schema_name, &cache, file_path.as_deref()).into())
}

#[napi]
pub fn validate_logo_content(
    content: Buffer,
    filename: String,
    logo_name: Option<String>,
    file_path: Option<String>,
) -> ValidationResult {
    core::validators::validate_logo(
        content.as_ref(),
        &filename,
        logo_name.as_deref(),
        file_path.as_deref(),
    ).into()
}

#[napi]
pub fn validate_folder_name_content(
    folder_name: String,
    json_content: String,
    json_key: String,
    file_path: Option<String>,
) -> Result<ValidationResult> {
    let data: serde_json::Value = serde_json::from_str(&json_content)
        .map_err(|e| Error::from_reason(format!("Invalid JSON: {}", e)))?;

    Ok(core::validators::validate_folder_name(
        &folder_name,
        &data,
        &json_key,
        file_path.as_deref(),
    ).into())
}

#[napi]
pub fn validate_gtin_ean_content(
    sizes_entries: Vec<SizesInput>,
) -> Result<ValidationResult> {
    let parsed: Vec<(String, serde_json::Value)> = sizes_entries.into_iter()
        .map(|entry| {
            let val = serde_json::from_str(&entry.content)
                .map_err(|e| Error::from_reason(format!("Invalid JSON for '{}': {}", entry.path, e)));
            val.map(|v| (entry.path, v))
        })
        .collect::<Result<_>>()?;

    let refs: Vec<(&str, &serde_json::Value)> = parsed.iter().map(|(p, v)| (p.as_str(), v)).collect();
    Ok(core::validators::validate_gtin_ean(&refs).into())
}

#[napi]
pub fn validate_store_ids_content(
    store_ids: Vec<String>,
    sizes_entries: Vec<SizesInput>,
) -> Result<ValidationResult> {
    let valid_ids: std::collections::HashSet<String> = store_ids.into_iter().collect();

    let parsed: Vec<(String, serde_json::Value)> = sizes_entries.into_iter()
        .map(|entry| {
            let val = serde_json::from_str(&entry.content)
                .map_err(|e| Error::from_reason(format!("Invalid JSON for '{}': {}", entry.path, e)));
            val.map(|v| (entry.path, v))
        })
        .collect::<Result<_>>()?;

    let refs: Vec<(&str, &serde_json::Value)> = parsed.iter().map(|(p, v)| (p.as_str(), v)).collect();
    Ok(core::validators::validate_store_ids(&valid_ids, &refs).into())
}

#[napi]
pub fn validate_all_content(
    data: ValidateAllContentInput,
) -> Result<ValidationResult> {
    use rayon::prelude::*;

    // Build schema cache from provided schemas
    let schema_values: HashMap<String, serde_json::Value> = data.schemas.into_iter()
        .map(|(k, v)| {
            let val = serde_json::from_str(&v)
                .map_err(|e| Error::from_reason(format!("Invalid schema JSON for '{}': {}", k, e)));
            val.map(|v| (k, v))
        })
        .collect::<Result<_>>()?;
    let schema_cache = core::SchemaCache::from_map(schema_values);

    // Parse all JSON file inputs
    let json_parsed: Vec<(String, String, serde_json::Value)> = data.json_files.into_iter()
        .map(|f| {
            let val = serde_json::from_str(&f.content)
                .map_err(|e| Error::from_reason(format!("Invalid JSON for '{}': {}", f.path, e)));
            val.map(|v| (f.path, f.schema_name, v))
        })
        .collect::<Result<_>>()?;

    // Parse folder JSON inputs
    let folder_parsed: Vec<(String, String, serde_json::Value, String)> = data.folders.into_iter()
        .map(|f| {
            let val = serde_json::from_str(&f.json_content)
                .map_err(|e| Error::from_reason(format!("Invalid JSON for folder '{}': {}", f.path, e)));
            val.map(|v| (f.path, f.folder_name, v, f.json_key))
        })
        .collect::<Result<_>>()?;

    let mut result = core::ValidationResult::default();

    // JSON validation (parallel)
    let json_results: Vec<core::ValidationResult> = json_parsed
        .par_iter()
        .map(|(path, schema_name, data)| {
            core::validators::validate_json(data, schema_name, &schema_cache, Some(path))
        })
        .collect();
    for r in json_results {
        result.merge_from(&r);
    }

    // Logo validation (parallel) — convert Buffer to Vec<u8> for Send safety
    let logo_data: Vec<(String, String, Vec<u8>)> = data.logo_files.into_iter()
        .map(|logo| (logo.path, logo.filename, logo.content.to_vec()))
        .collect();
    let logo_results: Vec<core::ValidationResult> = logo_data
        .par_iter()
        .map(|(path, filename, bytes)| {
            core::validators::validate_logo(
                bytes,
                filename,
                None,
                Some(path.as_str()),
            )
        })
        .collect();
    for r in logo_results {
        result.merge_from(&r);
    }

    // Folder name validation (parallel)
    let folder_results: Vec<core::ValidationResult> = folder_parsed
        .par_iter()
        .map(|(path, folder_name, json_data, json_key)| {
            core::validators::validate_folder_name(folder_name, json_data, json_key, Some(path))
        })
        .collect();
    for r in folder_results {
        result.merge_from(&r);
    }

    // Collect sizes entries from json_parsed for gtin/store_id validation
    let sizes_entries: Vec<(&str, &serde_json::Value)> = json_parsed.iter()
        .filter(|(_, schema_name, _)| schema_name == "sizes")
        .map(|(path, _, data)| (path.as_str(), data))
        .collect();

    // Store ID validation
    let valid_ids: std::collections::HashSet<String> = data.store_ids.into_iter().collect();
    result.merge_from(&core::validators::validate_store_ids(&valid_ids, &sizes_entries));

    // GTIN/EAN validation
    result.merge_from(&core::validators::validate_gtin_ean(&sizes_entries));

    Ok(result.into())
}
