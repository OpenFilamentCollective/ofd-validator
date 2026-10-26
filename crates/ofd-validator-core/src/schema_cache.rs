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
    ("sustainability", "sustainability_schema.json"),
];

/// Schema name for a `*_schema.json` filename, e.g. "sizes_schema.json" -> "sizes".
fn name_from_filename(filename: &str) -> Option<&str> {
    filename.strip_suffix("_schema.json").filter(|n| !n.is_empty())
}

/// Register a schema under every key a `$ref` might use to reach it.
fn register_uris(
    schemas_by_uri: &mut HashMap<String, Arc<Value>>,
    filename: &str,
    schema: &Arc<Value>,
) {
    schemas_by_uri.insert(format!("./{}", filename), Arc::clone(schema));
    schemas_by_uri.insert(filename.to_string(), Arc::clone(schema));
    if let Some(id) = schema.get("$id").and_then(|v| v.as_str()) {
        schemas_by_uri.insert(id.to_string(), Arc::clone(schema));
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
        use crate::util::load_json;

        let mut schemas_by_name = HashMap::new();
        let mut schemas_by_uri = HashMap::new();

        // Known schemas first, then any other `*_schema.json` in the directory so that a
        // new cross-file `$ref` (e.g. "./sustainability_schema.json") resolves without a release.
        let mut files: Vec<(String, String)> = SCHEMA_FILES
            .iter()
            .map(|(n, f)| (n.to_string(), f.to_string()))
            .collect();
        if let Ok(entries) = std::fs::read_dir(schemas_dir) {
            let mut extra: Vec<(String, String)> = entries
                .filter_map(|e| e.ok())
                .filter_map(|e| e.file_name().into_string().ok())
                .filter(|f| !files.iter().any(|(_, known)| known == f))
                .filter_map(|f| name_from_filename(&f).map(|n| (n.to_string(), f.clone())))
                .collect();
            extra.sort();
            files.extend(extra);
        }

        for (name, filename) in &files {
            let path = schemas_dir.join(filename);
            if let Some(schema) = load_json(&path) {
                let schema = Arc::new(schema);

                schemas_by_name.insert(name.clone(), Arc::clone(&schema));

                let relpath = format!("{}/{}", schemas_dir.display(), filename);
                schemas_by_uri.insert(relpath, Arc::clone(&schema));
                register_uris(&mut schemas_by_uri, filename, &schema);
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
    pub fn from_map(schemas: HashMap<String, Value>) -> Self {
        let mut schemas_by_name = HashMap::new();
        let mut schemas_by_uri = HashMap::new();

        for (name, schema_value) in &schemas {
            let filename = SCHEMA_FILES
                .iter()
                .find(|(n, _)| n == name)
                .map(|(_, f)| f.to_string())
                .unwrap_or_else(|| format!("{}_schema.json", name));
            let schema = Arc::new(schema_value.clone());

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

    #[cfg(feature = "filesystem")]
    #[test]
    fn from_directory_loads_unlisted_schema_files() {
        let dir = std::env::temp_dir().join(format!("ofd-schema-cache-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        for (name, schema) in schemas() {
            std::fs::write(dir.join(format!("{}_schema.json", name)), schema.to_string()).unwrap();
        }
        // An unlisted schema file is picked up too.
        std::fs::write(dir.join("extra_schema.json"), json!({"type": "string"}).to_string()).unwrap();

        let cache = SchemaCache::from_directory(&dir);
        assert!(cache.get("extra").is_some());
        let bad = validate_json(&json!({"sustainability": {"bogus": 1}}), "filament", &cache, None);
        assert!(!bad.errors.is_empty());
        let ok = validate_json(&json!({"sustainability": {"note": "y"}}), "filament", &cache, None);
        assert!(ok.errors.is_empty(), "{:?}", ok.errors);

        std::fs::remove_dir_all(&dir).ok();
    }
}
