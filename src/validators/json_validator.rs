use std::path::Path;

use jsonschema::Retrieve;
use serde_json::Value;

use crate::schema_cache::SchemaCache;
use crate::types::{ValidationError, ValidationResult};
use crate::util::load_json;

/// Custom retriever for resolving $ref URIs against our schema cache.
struct SchemaRetriever {
    cache: SchemaCache,
}

impl Retrieve for SchemaRetriever {
    fn retrieve(
        &self,
        uri: &jsonschema::Uri<&str>,
    ) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        let uri_str = uri.as_str();

        // Strip fragment (e.g. "#/definitions/...") — jsonschema handles fragments internally
        let base_uri = uri_str.split('#').next().unwrap_or(uri_str);

        if base_uri.is_empty() {
            return Err(format!("Empty URI after stripping fragment: {}", uri_str).into());
        }

        // The jsonschema crate resolves relative $ref like "./foo.json" into
        // "json-schema:///foo.json". Strip that prefix to match our cache keys.
        let lookup_key = base_uri
            .strip_prefix("json-schema:///")
            .or_else(|| base_uri.strip_prefix("json-schema://"))
            .unwrap_or(base_uri);

        self.cache
            .resolve_ref(lookup_key)
            .ok_or_else(|| format!("Schema not found: {}", uri_str).into())
    }
}

pub fn validate_json_file_impl(
    json_path: &Path,
    schema_name: &str,
    schema_cache: &SchemaCache,
) -> ValidationResult {
    let mut result = ValidationResult::default();
    let path_str = json_path.to_string_lossy().to_string();

    // Load the JSON data
    let data = match load_json(json_path) {
        Some(v) => v,
        None => {
            result.add(ValidationError::error(
                "JSON",
                "Failed to load JSON file",
                Some(path_str),
            ));
            return result;
        }
    };

    // Get the schema
    let schema = match schema_cache.get(schema_name) {
        Some(s) => s,
        None => {
            result.add(ValidationError::error(
                "JSON",
                format!("Schema '{}' not found", schema_name),
                Some(path_str),
            ));
            return result;
        }
    };

    // Build validator with custom retriever for $ref resolution
    let retriever = SchemaRetriever {
        cache: schema_cache.clone(),
    };

    let validator = match jsonschema::options()
        .with_retriever(retriever)
        .build(schema)
    {
        Ok(v) => v,
        Err(e) => {
            result.add(ValidationError::error(
                "JSON",
                format!("Schema compilation error: {}", e),
                Some(path_str),
            ));
            return result;
        }
    };

    // Validate — collect all errors using iter_errors
    for error in validator.iter_errors(&data) {
        let json_path_str = format!("{}", error.instance_path);
        result.add(ValidationError::error(
            "JSON",
            format!(
                "Schema validation failed: {} at {}",
                error, json_path_str
            ),
            Some(path_str.clone()),
        ));
    }

    result
}
