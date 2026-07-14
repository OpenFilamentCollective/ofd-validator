use serde_json::Value;
use std::collections::HashSet;
use std::sync::LazyLock;

use crate::types::{ValidationError, ValidationResult};

/// Known tracking query-parameter keys (compared case-insensitively). Prefix families
/// (`utm_`, `mc_`, `pk_`) are handled separately in `is_tracking_key`.
///
/// Note: Shopify product selectors `variant` and `id` are intentionally NOT included —
/// they can point at a specific colour/product, so we treat them as meaningful.
static TRACKING_PARAMS: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    [
        "tag", "linkcode", "psc", "th", "ref", "ref_", "fbclid", "gclid", "gclsrc", "dclid",
        "msclkid", "yclid", "twclid", "ttclid", "igshid", "si", "mc_cid", "mc_eid", "_pos", "_psq",
        "_ss", "_v", "_sid", "spm", "scm", "aff", "affid", "srsltid", "gad_source", "epik",
        "pk_campaign", "pk_kwd",
        // Amazon / marketplace search-result context (identity lives in the /dp/<ASIN> path)
        "dib", "dib_tag", "keywords", "qid", "sr", "sprefix", "crid", "ascsubtag", "smid",
        "content-id", "qsid", "rnid", "refinements", "_encoding", "pldnsite",
    ]
    .into_iter()
    .collect()
});

fn is_tracking_key(key: &str) -> bool {
    let k = key.trim().to_ascii_lowercase();
    k.starts_with("utm_")
        || k.starts_with("mc_")
        || k.starts_with("pk_")
        || k.starts_with("pd_rd_")
        || k.starts_with("pf_rd_")
        || TRACKING_PARAMS.contains(k.as_str())
}

/// Hosts where the product identity is entirely in the path, so every query param is
/// search/tracking context and can be dropped wholesale (e.g. Amazon `/dp/<ASIN>`).
fn has_disposable_query(base: &str) -> bool {
    let host = match crate::util::url_host(base) {
        Some(h) => h.to_ascii_lowercase(),
        None => return false,
    };
    if host == "a.co" || host == "amzn.to" || host == "amzn.eu" {
        return true;
    }
    ["amazon", "ebay"]
        .iter()
        .any(|b| host.starts_with(&format!("{b}.")) || host.contains(&format!(".{b}.")))
}

/// Whether a URL fragment looks like tracking data (e.g. `#utm_source=x`) rather than a
/// content anchor (e.g. `#productinfo`, which we keep).
fn is_tracking_fragment(fragment: &str) -> bool {
    if !fragment.contains('=') {
        return false;
    }
    let first_key = fragment.split(['&', '=']).next().unwrap_or("");
    is_tracking_key(first_key)
}

/// Remove known tracking parameters (plus an empty `?` and tracking-only fragments) from a
/// URL, preserving the original ordering/encoding of everything kept. Idempotent.
pub fn strip_tracking(url: &str) -> String {
    let (before_frag, fragment) = match url.split_once('#') {
        Some((b, f)) => (b, Some(f)),
        None => (url, None),
    };
    let (base, query) = match before_frag.split_once('?') {
        Some((b, q)) => (b, Some(q)),
        None => (before_frag, None),
    };

    let mut out = String::from(base);

    if let Some(q) = query {
        if !has_disposable_query(base) {
            let kept: Vec<&str> = q
                .split('&')
                .filter(|pair| {
                    if pair.is_empty() {
                        return false;
                    }
                    let key = pair.split('=').next().unwrap_or("");
                    !is_tracking_key(key)
                })
                .collect();
            if !kept.is_empty() {
                out.push('?');
                out.push_str(&kept.join("&"));
            }
        }
    }

    if let Some(frag) = fragment {
        if !frag.is_empty() && !is_tracking_fragment(frag) {
            out.push('#');
            out.push_str(frag);
        }
    }

    out
}

/// True if stripping tracking data would change the URL. Mirrors the webui `hasTracking`
/// helper; retained as a public API for consumers even though the rule uses `strip_tracking`
/// directly.
#[allow(dead_code)]
pub fn has_tracking(url: &str) -> bool {
    strip_tracking(url) != url
}

/// Warn about tracking parameters in `purchase_links[].url` inside sizes.json entries.
/// Each entry is (path_label, parsed sizes.json Value). Emits non-blocking warnings that
/// suggest the cleaned URL in the message.
pub fn validate_url_tracking(sizes_entries: &[(&str, &Value)]) -> ValidationResult {
    let mut result = ValidationResult::default();

    for (path_str, sizes_data) in sizes_entries {
        let sizes_arr = match sizes_data.as_array() {
            Some(a) => a,
            None => continue,
        };

        for (size_idx, size) in sizes_arr.iter().enumerate() {
            let purchase_links = match size.get("purchase_links").and_then(|v| v.as_array()) {
                Some(links) => links,
                None => continue,
            };

            for (link_idx, link) in purchase_links.iter().enumerate() {
                if let Some(url) = link.get("url").and_then(|v| v.as_str()) {
                    let cleaned = strip_tracking(url);
                    if cleaned != url {
                        result.add(ValidationError::warning(
                            "URLTracking",
                            format!(
                                "Tracking parameters in url at $[{}].purchase_links[{}]; suggested: {}",
                                size_idx, link_idx, cleaned
                            ),
                            Some(path_str.to_string()),
                        ));
                    }
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

    #[test]
    fn strips_amazon_affiliate() {
        assert_eq!(
            strip_tracking("https://www.amazon.eg/dp/B0GVSM88NP/?tag=3dfil-20&linkCode=ogi&psc=1&th=1"),
            "https://www.amazon.eg/dp/B0GVSM88NP/"
        );
    }

    #[test]
    fn strips_amazon_search_query_wholesale() {
        let dirty = "https://www.amazon.com/AmeLeis-Printer-Filament-Bundle-1-75mm/dp/B0GYNWLN2F?dib=eyJ2IjoiMSJ9.abc&dib_tag=se&keywords=AmeLeis&qid=1784042998&sr=8-1";
        assert_eq!(
            strip_tracking(dirty),
            "https://www.amazon.com/AmeLeis-Printer-Filament-Bundle-1-75mm/dp/B0GYNWLN2F"
        );
        assert!(has_tracking(dirty));
    }

    #[test]
    fn keeps_shopify_variant_selector() {
        let u = "https://shop.polymaker.com/products/panchroma-galaxy?variant=44933539987513";
        assert_eq!(strip_tracking(u), u);
    }

    #[test]
    fn keeps_variant_and_content_fragment() {
        let u = "https://shop.polymaker.com/products/panchroma-pla?variant=45411122413625#productinfo";
        assert_eq!(strip_tracking(u), u);
    }

    #[test]
    fn removes_trailing_empty_query() {
        assert_eq!(
            strip_tracking("https://us.polymaker.com/products/panchroma-glow?"),
            "https://us.polymaker.com/products/panchroma-glow"
        );
    }

    #[test]
    fn removes_trackers_but_keeps_meaningful() {
        assert_eq!(
            strip_tracking("https://x.com/p?utm_source=a&variant=5&gclid=z"),
            "https://x.com/p?variant=5"
        );
    }

    #[test]
    fn strips_tracking_fragment_keeps_anchor() {
        assert_eq!(strip_tracking("https://x.com/p#utm_source=a"), "https://x.com/p");
        assert_eq!(strip_tracking("https://x.com/p#section-2"), "https://x.com/p#section-2");
    }

    #[test]
    fn is_idempotent() {
        let dirty = "https://www.amazon.eg/dp/B0GVSM88NP/?tag=3dfil-20&th=1";
        let once = strip_tracking(dirty);
        assert_eq!(strip_tracking(&once), once);
        assert!(has_tracking(dirty));
        assert!(!has_tracking(&once));
    }

    #[test]
    fn rule_emits_warning_not_error() {
        let sizes = json!([{
            "filament_weight": 1000, "diameter": 1.75,
            "purchase_links": [{ "store_id": "amazon", "url": "https://a.com/x?tag=aff-20" }]
        }]);
        let entries = vec![("data/b/PLA/f/red/sizes.json", &sizes)];
        let res = validate_url_tracking(&entries);
        assert_eq!(res.warning_count(), 1);
        assert_eq!(res.error_count(), 0);
        assert!(res.is_valid()); // warnings never invalidate
    }
}
