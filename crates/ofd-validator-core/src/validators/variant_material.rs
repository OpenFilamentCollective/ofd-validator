use serde_json::Value;
use std::collections::HashSet;

use crate::types::{ValidationError, ValidationResult};

/// Canonical material-type tokens (mirrors `schemas/material_types_schema.json`).
/// Used as a fallback; `validate_dataset` prefers the live schema enum.
pub const MATERIAL_TYPES_FALLBACK: &[&str] = &[
    "PLA", "PETG", "TPU", "ABS", "ASA", "PC", "PCTG", "PP", "PA6", "PA11", "PA12", "PA66", "CPE",
    "TPE", "HIPS", "PHA", "PET", "PEI", "PBT", "PVB", "PVA", "PEKK", "PEEK", "BVOH", "TPC", "PPS",
    "PPSU", "PVC", "PEBA", "PVDF", "PPA", "PCL", "PES", "PMMA", "POM", "PPE", "PS", "PSU", "TPI",
    "SBS", "OBC", "EVA",
];

/// Split a display name / id into upper-cased alphanumeric tokens.
fn tokens(s: &str) -> Vec<String> {
    s.split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|t| !t.is_empty())
        .map(|t| t.to_ascii_uppercase())
        .collect()
}

/// Return the first material token embedded in `name`, if any (whole-token match).
fn find_material_token(name: &str, material_types: &HashSet<String>) -> Option<String> {
    tokens(name).into_iter().find(|t| material_types.contains(t))
}

/// Extract the variant folder name from a `.../<variant>/variant.json` path label.
fn variant_folder(path: &str) -> Option<String> {
    let norm = path.replace('\\', "/");
    let mut parts = norm.rsplit('/');
    let _file = parts.next()?; // variant.json
    parts.next().map(|s| s.to_string())
}

/// Warn when a variant's colour name (or its folder id) embeds a material-type token,
/// which is redundant with the parent MATERIAL directory. Non-blocking warnings;
/// at most one per variant. `variant_entries` are (path_label, parsed variant.json Value).
pub fn validate_variant_material(
    variant_entries: &[(&str, &Value)],
    material_types: &HashSet<String>,
) -> ValidationResult {
    let mut result = ValidationResult::default();

    for (path_str, data) in variant_entries {
        if let Some(name) = data.get("name").and_then(|v| v.as_str()) {
            if let Some(tok) = find_material_token(name, material_types) {
                result.add(ValidationError::warning(
                    "VariantMaterial",
                    format!(
                        "Colour name '{}' contains material type '{}', which is redundant with the material folder",
                        name, tok
                    ),
                    Some(path_str.to_string()),
                ));
                continue;
            }
        }

        if let Some(folder) = variant_folder(path_str) {
            if let Some(tok) = find_material_token(&folder, material_types) {
                result.add(ValidationError::warning(
                    "VariantMaterial",
                    format!(
                        "Variant folder id '{}' contains material type '{}', which is redundant with the material folder",
                        folder, tok
                    ),
                    Some(path_str.to_string()),
                ));
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn material_set() -> HashSet<String> {
        MATERIAL_TYPES_FALLBACK.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn flags_material_in_colour_name() {
        let v = json!({ "name": "Galaxy PETG", "color_hex": "#112233" });
        let entries = vec![("data/b/PETG/f/galaxy_petg/variant.json", &v)];
        let res = validate_variant_material(&entries, &material_set());
        assert_eq!(res.warning_count(), 1);
        assert_eq!(res.error_count(), 0);
        assert!(res.is_valid());
    }

    #[test]
    fn flags_material_in_folder_id_when_name_clean() {
        let v = json!({ "name": "Sakura Pink", "color_hex": "#ffaabb" });
        let entries = vec![("data/sunlu/PETG/petg/sakura_pink_petg/variant.json", &v)];
        let res = validate_variant_material(&entries, &material_set());
        assert_eq!(res.warning_count(), 1);
    }

    #[test]
    fn clean_name_and_folder_pass() {
        let v = json!({ "name": "Sakura Pink", "color_hex": "#ffaabb" });
        let entries = vec![("data/sunlu/PETG/petg/sakura_pink/variant.json", &v)];
        let res = validate_variant_material(&entries, &material_set());
        assert_eq!(res.warning_count(), 0);
    }

    #[test]
    fn whole_token_match_avoids_substrings() {
        // "PETAL" must not match "PET"; "Space" must not match "PA"/"AS".
        let v = json!({ "name": "Petal Space Grey", "color_hex": "#777777" });
        let entries = vec![("data/b/PLA/f/petal/variant.json", &v)];
        let res = validate_variant_material(&entries, &material_set());
        assert_eq!(res.warning_count(), 0);
    }
}
