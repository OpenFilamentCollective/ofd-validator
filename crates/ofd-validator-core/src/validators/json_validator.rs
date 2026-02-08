use jsonschema::Retrieve;
use serde_json::Value;

use crate::schema_cache::SchemaCache;
use crate::types::{ValidationError, ValidationResult};

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

        let base_uri = uri_str.split('#').next().unwrap_or(uri_str);

        if base_uri.is_empty() {
            return Err(format!("Empty URI after stripping fragment: {}", uri_str).into());
        }

        let lookup_key = base_uri
            .strip_prefix("json-schema:///")
            .or_else(|| base_uri.strip_prefix("json-schema://"))
            .unwrap_or(base_uri);

        self.cache
            .resolve_ref(lookup_key)
            .ok_or_else(|| format!("Schema not found: {}", uri_str).into())
    }
}

/// Validate parsed JSON data against a named schema.
pub fn validate_json(
    data: &Value,
    schema_name: &str,
    schema_cache: &SchemaCache,
    path_label: Option<&str>,
) -> ValidationResult {
    let mut result = ValidationResult::default();
    let path_str = path_label.map(|s| s.to_string());

    let schema = match schema_cache.get(schema_name) {
        Some(s) => s,
        None => {
            result.add(ValidationError::error(
                "JSON",
                format!("Schema '{}' not found", schema_name),
                path_str,
            ));
            return result;
        }
    };

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
                path_str,
            ));
            return result;
        }
    };

    for error in validator.iter_errors(data) {
        let json_path_str = format!("{}", error.instance_path);
        result.add(ValidationError::error(
            "JSON",
            format!(
                "Schema validation failed: {} at {}",
                error, json_path_str
            ),
            path_str.clone(),
        ));
    }

    result
}
