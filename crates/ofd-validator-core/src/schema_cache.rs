use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

/// Maps schema names (e.g. "brand", "material") to their filenames.
pub const SCHEMA_FILES: &[(&str, &str)] = &[
    ("store", "store_schema.json"),
    ("brand", "brand_schema.json"),
    ("material", "material_schema.json"),
    ("material_types", "material_types_schema.json"),
    ("filament", "filament_schema.json"),
    ("variant", "variant_schema.json"),
    ("sizes", "sizes_schema.json"),
];

/// Schema name for a `*_schema.json` filename, e.g. "sizes_schema.json" -> "sizes".
fn name_from_filename(filename: &str) -> Option<&str> {
    filename.strip_suffix("_schema.json").filter(|n| is_schema_name(n))
}

/// Schema names are plain identifiers (e.g. "material_types"), never paths or URIs.
fn is_schema_name(name: &str) -> bool {
    !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Filename for a schema name, e.g. "sizes" -> "sizes_schema.json".
fn filename_for(name: &str) -> String {
    SCHEMA_FILES
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, f)| f.to_string())
        .unwrap_or_else(|| format!("{}_schema.json", name))
}

/// Register a schema under every key a `$ref` might use to reach it.
/// A `$id` never replaces an already-registered key, so callers must register
/// known schemas before extra ones.
fn register_uris(
    schemas_by_uri: &mut HashMap<String, Arc<Value>>,
    filename: &str,
    schema: &Arc<Value>,
) {
    schemas_by_uri.insert(format!("./{}", filename), Arc::clone(schema));
    schemas_by_uri.insert(filename.to_string(), Arc::clone(schema));
    if let Some(id) = schema.get("$id").and_then(|v| v.as_str()) {
        schemas_by_uri
            .entry(id.to_string())
            .or_insert_with(|| Arc::clone(schema));
    }
}

#[derive(Clone)]
pub struct SchemaCache {
    /// Schemas indexed by name (e.g. "brand", "material")
    schemas_by_name: HashMap<String, Arc<Value>>,
    /// All schemas indexed by various URI keys for $ref resolution
    schemas_by_uri: HashMap<String, Arc<Value>>,
}

impl SchemaCache {
    /// Create a schema cache from a directory on the filesystem.
    #[cfg(feature = "filesystem")]
    pub fn from_directory(schemas_dir: &std::path::Path) -> Self {
        let mut schemas_by_name = HashMap::new();
        let mut schemas_by_uri = HashMap::new();

        // Known schemas first, then any other `*_schema.json` in the directory so that a
        // new cross-file `$ref` (e.g. "./sustainability_schema.json") resolves without a release.
        let mut files: Vec<(String, String)> = SCHEMA_FILES
            .iter()
            .map(|(n, f)| (n.to_string(), f.to_string()))
            .collect();
        match std::fs::read_dir(schemas_dir) {
            Ok(entries) => {
                let mut extra: Vec<(String, String)> = entries
                    .filter_map(|e| e.ok())
                    .filter_map(|e| e.file_name().into_string().ok())
                    .filter(|f| !files.iter().any(|(_, known)| known == f))
                    .filter_map(|f| name_from_filename(&f).map(|n| (n.to_string(), f.clone())))
                    .collect();
                extra.sort();
                files.extend(extra);
            }
            Err(e) => eprintln!(
                "Warning: failed to read schema directory {}: {}",
                schemas_dir.display(),
                e
            ),
        }

        for (name, filename) in &files {
            let path = schemas_dir.join(filename);
            let content = match std::fs::read_to_string(&path) {
                Ok(c) => c,
                // A missing known schema is reported when something validates against it.
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
                Err(e) => {
                    eprintln!("Warning: failed to read schema {}: {}", path.display(), e);
                    continue;
                }
            };
            match serde_json::from_str::<Value>(&content) {
                Ok(schema) => {
                    let schema = Arc::new(schema);

                    schemas_by_name.insert(name.clone(), Arc::clone(&schema));

                    let relpath = format!("{}/{}", schemas_dir.display(), filename);
                    schemas_by_uri.insert(relpath, Arc::clone(&schema));
                    register_uris(&mut schemas_by_uri, filename, &schema);
                }
                Err(e) => eprintln!("Warning: invalid JSON in schema {}: {}", path.display(), e),
            }
        }

        Self {
            schemas_by_name,
            schemas_by_uri,
        }
    }

    /// Create a schema cache from pre-loaded schema values.
    /// Keys in the map should be schema names (e.g. "brand", "material"); a name outside
    /// `SCHEMA_FILES` is registered as `<name>_schema.json` for `$ref` resolution.
    /// Keys that are not plain names (paths, URIs) are ignored.
    pub fn from_map(schemas: HashMap<String, Value>) -> Self {
        let mut schemas_by_name = HashMap::new();
        let mut schemas_by_uri = HashMap::new();

        // Known schemas first, then the rest in a stable order, so `$id`s resolve deterministically.
        let mut names: Vec<&String> = schemas.keys().filter(|n| is_schema_name(n)).collect();
        names.sort_by_key(|n| {
            let known = SCHEMA_FILES.iter().position(|(k, _)| k == n);
            (known.unwrap_or(usize::MAX), n.as_str())
        });

        for name in names {
            let filename = filename_for(name);
            let schema = Arc::new(schemas[name].clone());

            schemas_by_name.insert(name.clone(), Arc::clone(&schema));
            register_uris(&mut schemas_by_uri, &filename, &schema);
        }

        Self {
            schemas_by_name,
            schemas_by_uri,
        }
    }

    pub fn get(&self, schema_name: &str) -> Option<&Value> {
        self.schemas_by_name.get(schema_name).map(|v| v.as_ref())
    }

    pub fn resolve_ref(&self, uri: &str) -> Option<Value> {
        if let Some(schema) = self.schemas_by_uri.get(uri) {
            return Some((**schema).clone());
        }

        let stripped = uri.strip_prefix("./").unwrap_or(uri);
        if let Some(schema) = self.schemas_by_uri.get(stripped) {
            return Some((**schema).clone());
        }

        for (key, schema) in &self.schemas_by_uri {
            if key.ends_with(stripped) {
                return Some((**schema).clone());
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validators::validate_json;
    use serde_json::json;

    fn schemas() -> HashMap<String, Value> {
        let mut m = HashMap::new();
        m.insert(
            "filament".to_string(),
            json!({
                "type": "object",
                "properties": {
                    "name": {"type": "string"},
                    "sustainability": {"$ref": "./sustainability_schema.json"}
                },
                "additionalProperties": false
            }),
        );
        m.insert(
            "sustainability".to_string(),
            json!({
                "$id": "./sustainability_schema.json",
                "type": "object",
                "properties": {"note": {"type": "string"}},
                "additionalProperties": false
            }),
        );
        m
    }

    #[test]
    fn name_from_filename_strips_suffix() {
        assert_eq!(name_from_filename("sizes_schema.json"), Some("sizes"));
        assert_eq!(name_from_filename("_schema.json"), None);
        assert_eq!(name_from_filename("trait_rules.json"), None);
    }

    #[test]
    fn from_map_resolves_cross_file_ref() {
        let cache = SchemaCache::from_map(schemas());
        let ok = validate_json(&json!({"name": "x", "sustainability": {"note": "y"}}), "filament", &cache, None);
        assert!(ok.errors.is_empty(), "{:?}", ok.errors);

        let bad = validate_json(&json!({"name": "x", "sustainability": {"bogus": 1}}), "filament", &cache, None);
        assert!(!bad.errors.is_empty());
    }

    #[test]
    fn from_map_ignores_non_name_keys() {
        let mut m = schemas();
        m.insert("./sustainability_schema.json".to_string(), json!({"type": "string"}));
        let cache = SchemaCache::from_map(m);
        assert!(cache.get("./sustainability_schema.json").is_none());
        let ok = validate_json(&json!({"sustainability": {"note": "y"}}), "filament", &cache, None);
        assert!(ok.errors.is_empty(), "{:?}", ok.errors);
    }

    #[test]
    fn extra_schema_id_does_not_shadow_known_schema() {
        let mut m = schemas();
        m.insert("material_types".to_string(), json!({"$id": "./material_types_schema.json", "type": "string"}));
        // A stray copy claiming the same `$id` must not replace the real schema.
        m.insert("aaa_copy".to_string(), json!({"$id": "./material_types_schema.json", "type": "number"}));
        let cache = SchemaCache::from_map(m);
        assert_eq!(
            cache.resolve_ref("./material_types_schema.json"),
            Some(json!({"$id": "./material_types_schema.json", "type": "string"}))
        );
    }

    /// Temp directory removed on drop, so a failed assertion doesn't leave files behind.
    #[cfg(feature = "filesystem")]
    struct TempDir(std::path::PathBuf);

    #[cfg(feature = "filesystem")]
    impl TempDir {
        fn new(label: &str) -> Self {
            let dir = std::env::temp_dir().join(format!("ofd-schema-cache-{}-{}", label, std::process::id()));
            std::fs::remove_dir_all(&dir).ok();
            std::fs::create_dir_all(&dir).unwrap();
            Self(dir)
        }
    }

    #[cfg(feature = "filesystem")]
    impl Drop for TempDir {
        fn drop(&mut self) {
            std::fs::remove_dir_all(&self.0).ok();
        }
    }

    #[cfg(feature = "filesystem")]
    #[test]
    fn from_directory_loads_unlisted_schema_files() {
        let dir = TempDir::new("unlisted");
        // sustainability_schema.json is not in SCHEMA_FILES, so this exercises the directory scan.
        for (name, schema) in schemas() {
            std::fs::write(dir.0.join(format!("{}_schema.json", name)), schema.to_string()).unwrap();
        }
        std::fs::write(dir.0.join("extra_schema.json"), json!({"type": "string"}).to_string()).unwrap();
        // A malformed extra schema is skipped (with a warning) rather than breaking the cache.
        std::fs::write(dir.0.join("broken_schema.json"), "{").unwrap();

        let cache = SchemaCache::from_directory(&dir.0);
        assert!(cache.get("extra").is_some());
        assert!(cache.get("sustainability").is_some());
        assert!(cache.get("broken").is_none());
        let bad = validate_json(&json!({"sustainability": {"bogus": 1}}), "filament", &cache, None);
        assert!(!bad.errors.is_empty());
        let ok = validate_json(&json!({"sustainability": {"note": "y"}}), "filament", &cache, None);
        assert!(ok.errors.is_empty(), "{:?}", ok.errors);
    }
}
