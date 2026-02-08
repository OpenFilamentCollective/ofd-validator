use serde_json::Value;
use std::path::Path;

pub const ILLEGAL_CHARACTERS: &[char] = &[
    '#', '%', '&', '{', '}', '\\', '<', '>', '*', '?',
    '/', '$', '!', '\'', '"', ':', '@', '`', '|', '=',
];

pub const LOGO_MIN_SIZE: u32 = 100;
pub const LOGO_MAX_SIZE: u32 = 400;

pub fn load_json(path: &Path) -> Option<Value> {
    let content = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

pub fn cleanse_folder_name(name: &str) -> String {
    name.replace('/', " ").trim().to_string()
}
