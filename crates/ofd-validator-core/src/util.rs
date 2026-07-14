use serde_json::Value;

pub const ILLEGAL_CHARACTERS: &[char] = &[
    '#', '%', '&', '{', '}', '\\', '<', '>', '*', '?',
    '/', '$', '!', '\'', '"', ':', '@', '`', '|', '=',
];

pub const LOGO_MIN_SIZE: u32 = 100;
pub const LOGO_MAX_SIZE: u32 = 400;

pub fn parse_json(content: &str) -> Option<Value> {
    serde_json::from_str(content).ok()
}

pub fn cleanse_folder_name(name: &str) -> String {
    name.replace('/', " ").trim().to_string()
}

/// Extract the host (e.g. `shop.polymaker.com`) from an absolute URL, if parseable.
pub fn url_host(url: &str) -> Option<String> {
    url::Url::parse(url)
        .ok()
        .and_then(|u| u.host_str().map(|h| h.to_string()))
}

#[cfg(feature = "filesystem")]
pub fn load_json(path: &std::path::Path) -> Option<Value> {
    let content = std::fs::read_to_string(path).ok()?;
    parse_json(&content)
}
