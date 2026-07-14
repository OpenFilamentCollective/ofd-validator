use serde_json::Value;
use std::collections::{BTreeSet, HashMap};

use crate::types::{ValidationError, ValidationResult};

/// Minimum number of distinct variants sharing one URL before we warn.
const MIN_VARIANTS: usize = 3;

/// From a sizes.json path label `.../<brand>/<MAT>/<filament>/<variant>/sizes.json`,
/// return `(filament_key, variant_folder)` = (`.../<brand>/<MAT>/<filament>`, `<variant>`).
fn filament_key(path: &str) -> Option<(String, String)> {
    let norm = path.replace('\\', "/");
    let parts: Vec<&str> = norm.split('/').collect();
    if parts.len() < 3 {
        return None;
    }
    let variant = parts[parts.len() - 2].to_string();
    let key = parts[..parts.len() - 2].join("/");
    Some((key, variant))
}

/// Warn when the same purchase URL is reused across many variants of one filament (a generic
/// link copied to every colour). Non-blocking; one warning per (filament, url).
pub fn validate_duplicate_links(sizes_entries: &[(&str, &Value)]) -> ValidationResult {
    let mut result = ValidationResult::default();

    // filament_key -> url -> set of variant folder names
    let mut map: HashMap<String, HashMap<String, BTreeSet<String>>> = HashMap::new();

    for (path_str, sizes_data) in sizes_entries {
        let (fkey, variant) = match filament_key(path_str) {
            Some(v) => v,
            None => continue,
        };
        let sizes_arr = match sizes_data.as_array() {
            Some(a) => a,
            None => continue,
        };
        for size in sizes_arr {
            let purchase_links = match size.get("purchase_links").and_then(|v| v.as_array()) {
                Some(l) => l,
                None => continue,
            };
            for link in purchase_links {
                if let Some(url) = link.get("url").and_then(|v| v.as_str()) {
                    map.entry(fkey.clone())
                        .or_default()
                        .entry(url.to_string())
                        .or_default()
                        .insert(variant.clone());
                }
            }
        }
    }

    // Deterministic output ordering.
    let mut filaments: Vec<&String> = map.keys().collect();
    filaments.sort();
    for fkey in filaments {
        let urls = &map[fkey];
        let mut url_keys: Vec<&String> = urls.keys().collect();
        url_keys.sort();
        for url in url_keys {
            let variants = &urls[url];
            if variants.len() >= MIN_VARIANTS {
                result.add(ValidationError::warning(
                    "DuplicateLink",
                    format!(
                        "The same purchase url is reused across {} variants of this filament: {}",
                        variants.len(),
                        url
                    ),
                    Some(format!("{}/*/sizes.json", fkey)),
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

    fn sized(url: &str) -> Value {
        json!([{
            "filament_weight": 1000, "diameter": 1.75,
            "purchase_links": [{ "store_id": "s", "url": url }]
        }])
    }

    #[test]
    fn flags_generic_link_reused_across_variants() {
        let base = "data/b/PLA/snapspeed_pla";
        let a = sized("https://a.com/generic");
        let b = sized("https://a.com/generic");
        let c = sized("https://a.com/generic");
        let entries = vec![
            (format!("{base}/red/sizes.json"), &a),
            (format!("{base}/blue/sizes.json"), &b),
            (format!("{base}/green/sizes.json"), &c),
        ];
        let refs: Vec<(&str, &Value)> = entries.iter().map(|(p, v)| (p.as_str(), *v)).collect();
        let res = validate_duplicate_links(&refs);
        assert_eq!(res.warning_count(), 1);
        assert!(res.is_valid());
    }

    #[test]
    fn distinct_links_pass() {
        let base = "data/b/PLA/f";
        let a = sized("https://a.com/1");
        let b = sized("https://a.com/2");
        let c = sized("https://a.com/3");
        let entries = vec![
            (format!("{base}/red/sizes.json"), &a),
            (format!("{base}/blue/sizes.json"), &b),
            (format!("{base}/green/sizes.json"), &c),
        ];
        let refs: Vec<(&str, &Value)> = entries.iter().map(|(p, v)| (p.as_str(), *v)).collect();
        assert_eq!(validate_duplicate_links(&refs).warning_count(), 0);
    }

    #[test]
    fn same_url_across_two_variants_below_threshold() {
        let base = "data/b/PLA/f";
        let a = sized("https://a.com/x");
        let b = sized("https://a.com/x");
        let entries = vec![
            (format!("{base}/red/sizes.json"), &a),
            (format!("{base}/blue/sizes.json"), &b),
        ];
        let refs: Vec<(&str, &Value)> = entries.iter().map(|(p, v)| (p.as_str(), *v)).collect();
        assert_eq!(validate_duplicate_links(&refs).warning_count(), 0);
    }
}
