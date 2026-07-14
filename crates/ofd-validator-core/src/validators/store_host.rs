use serde_json::Value;
use std::collections::HashMap;

use crate::types::{ValidationError, ValidationResult};
use crate::util::url_host;

/// Build a map of `store_id -> canonical host` from store.json entries.
/// `json_entries` are (path_label, schema_name, Value).
pub fn build_store_hosts(json_entries: &[(String, String, Value)]) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for (_, schema_name, data) in json_entries {
        if schema_name != "store" {
            continue;
        }
        let id = data.get("id").and_then(|v| v.as_str());
        let host = data
            .get("storefront_url")
            .and_then(|v| v.as_str())
            .and_then(url_host);
        if let (Some(id), Some(host)) = (id, host) {
            map.insert(id.to_string(), host);
        }
    }
    map
}

/// The registrable-ish base domain: the last two dot-separated labels.
/// `shop.polymaker.com` and `us.polymaker.com` both -> `polymaker.com`.
fn base_domain(host: &str) -> String {
    let labels: Vec<&str> = host.split('.').collect();
    let n = labels.len();
    if n >= 2 {
        labels[n - 2..].join(".")
    } else {
        host.to_string()
    }
}

/// Warn when a purchase-link URL's host differs from the store's storefront host but shares
/// the same base domain (an inconsistent subdomain, e.g. `shop.` vs `us.`). Cross-domain
/// links are left alone (they may be intentional resellers). Non-blocking warnings.
pub fn validate_store_host(
    store_hosts: &HashMap<String, String>,
    sizes_entries: &[(&str, &Value)],
) -> ValidationResult {
    let mut result = ValidationResult::default();

    for (path_str, sizes_data) in sizes_entries {
        let sizes_arr = match sizes_data.as_array() {
            Some(a) => a,
            None => continue,
        };
        for (size_idx, size) in sizes_arr.iter().enumerate() {
            let purchase_links = match size.get("purchase_links").and_then(|v| v.as_array()) {
                Some(l) => l,
                None => continue,
            };
            for (link_idx, link) in purchase_links.iter().enumerate() {
                let store_id = match link.get("store_id").and_then(|v| v.as_str()) {
                    Some(s) => s,
                    None => continue,
                };
                let canonical = match store_hosts.get(store_id) {
                    Some(h) => h,
                    None => continue,
                };
                let link_host = match link.get("url").and_then(|v| v.as_str()).and_then(url_host) {
                    Some(h) => h,
                    None => continue,
                };
                if &link_host != canonical && base_domain(&link_host) == base_domain(canonical) {
                    result.add(ValidationError::warning(
                        "StoreHost",
                        format!(
                            "url host '{}' at $[{}].purchase_links[{}] differs from store '{}' storefront host '{}'",
                            link_host, size_idx, link_idx, store_id, canonical
                        ),
                        Some(path_str.to_string()),
                    ));
                }
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn hosts() -> HashMap<String, String> {
        let mut m = HashMap::new();
        m.insert("polymaker".to_string(), "us.polymaker.com".to_string());
        m
    }

    #[test]
    fn flags_subdomain_drift() {
        let sizes = json!([{
            "filament_weight": 1000, "diameter": 1.75,
            "purchase_links": [{ "store_id": "polymaker", "url": "https://shop.polymaker.com/products/x" }]
        }]);
        let entries = vec![("data/b/PLA/f/red/sizes.json", &sizes)];
        let res = validate_store_host(&hosts(), &entries);
        assert_eq!(res.warning_count(), 1);
        assert!(res.is_valid());
    }

    #[test]
    fn same_host_passes() {
        let sizes = json!([{
            "filament_weight": 1000, "diameter": 1.75,
            "purchase_links": [{ "store_id": "polymaker", "url": "https://us.polymaker.com/products/x" }]
        }]);
        let entries = vec![("p", &sizes)];
        assert_eq!(validate_store_host(&hosts(), &entries).warning_count(), 0);
    }

    #[test]
    fn cross_domain_is_left_alone() {
        let sizes = json!([{
            "filament_weight": 1000, "diameter": 1.75,
            "purchase_links": [{ "store_id": "polymaker", "url": "https://www.amazon.com/dp/x" }]
        }]);
        let entries = vec![("p", &sizes)];
        assert_eq!(validate_store_host(&hosts(), &entries).warning_count(), 0);
    }

    #[test]
    fn builds_hosts_from_store_entries() {
        let store = json!({ "id": "polymaker", "storefront_url": "https://us.polymaker.com/" });
        let entries = vec![("stores/polymaker/store.json".to_string(), "store".to_string(), store)];
        let map = build_store_hosts(&entries);
        assert_eq!(map.get("polymaker").map(|s| s.as_str()), Some("us.polymaker.com"));
    }
}
