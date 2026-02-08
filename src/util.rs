use serde_json::Value;
use std::io::IsTerminal;
use std::path::Path;

pub const ILLEGAL_CHARACTERS: &[char] = &[
    '#', '%', '&', '{', '}', '\\', '<', '>', '*', '?',
    '/', '$', '!', '\'', '"', ':', '@', '`', '|', '=',
];

/// Print a colored progress step to stderr.
/// Format: "  -> Stage description... (count items)"
pub fn log_step(label: &str, count: Option<usize>) {
    let is_tty = std::io::stderr().is_terminal();
    if is_tty {
        match count {
            Some(n) => eprintln!("  \x1b[36m->\x1b[0m {} \x1b[2m({} items)\x1b[0m", label, n),
            None => eprintln!("  \x1b[36m->\x1b[0m {}", label),
        }
    } else {
        match count {
            Some(n) => eprintln!("  -> {} ({} items)", label, n),
            None => eprintln!("  -> {}", label),
        }
    }
}

pub const LOGO_MIN_SIZE: u32 = 100;
pub const LOGO_MAX_SIZE: u32 = 400;

pub fn load_json(path: &Path) -> Option<Value> {
    let content = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

pub fn cleanse_folder_name(name: &str) -> String {
    name.replace('/', " ").trim().to_string()
}
