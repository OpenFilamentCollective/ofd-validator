use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use crate::util::load_json;

/// Maps schema names (e.g. "brand", "material") to their filenames.
const SCHEMA_FILES: &[(&str, &str)] = &[
    ("store", "store_schema.json"),
    ("brand", "brand_schema.json"),
    ("material", "material_schema.json"),
    ("material_types", "material_types_schema.json"),
    ("filament", "filament_schema.json"),
    ("variant", "variant_schema.json"),
    ("sizes", "sizes_schema.json"),
];

#[derive(Clone)]
pub struct SchemaCache {
    /// Schemas indexed by name (e.g. "brand", "material")
    schemas_by_name: HashMap<String, Arc<Value>>,
    /// All schemas indexed by various URI keys for $ref resolution
    schemas_by_uri: HashMap<String, Arc<Value>>,
}

impl SchemaCache {
    pub fn new(schemas_dir: &Path) -> Self {
        let mut schemas_by_name = HashMap::new();
        let mut schemas_by_uri = HashMap::new();

        for (name, filename) in SCHEMA_FILES {
            let path = schemas_dir.join(filename);
            if let Some(schema) = load_json(&path) {
                let schema = Arc::new(schema);

                // Index by name
                schemas_by_name.insert(name.to_string(), Arc::clone(&schema));

                // Index by various URI forms for $ref resolution
                let relpath = format!("{}/{}", schemas_dir.display(), filename);
                schemas_by_uri.insert(relpath, Arc::clone(&schema));
                schemas_by_uri.insert(format!("./{}", filename), Arc::clone(&schema));
                schemas_by_uri.insert(filename.to_string(), Arc::clone(&schema));

                // Also register by $id if present
                if let Some(id) = schema.get("$id").and_then(|v| v.as_str()) {
                    schemas_by_uri.insert(id.to_string(), Arc::clone(&schema));
                }
            }
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
        // Try direct lookup
        if let Some(schema) = self.schemas_by_uri.get(uri) {
            return Some((**schema).clone());
        }

        // Try stripping leading "./"
        let stripped = uri.strip_prefix("./").unwrap_or(uri);
        if let Some(schema) = self.schemas_by_uri.get(stripped) {
            return Some((**schema).clone());
        }

        // Try matching by filename suffix
        for (key, schema) in &self.schemas_by_uri {
            if key.ends_with(stripped) {
                return Some((**schema).clone());
            }
        }

        None
    }

}
