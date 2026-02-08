use image::GenericImageView;
use regex::Regex;
use std::sync::LazyLock;

use crate::types::{ValidationError, ValidationResult};
use crate::util::{LOGO_MAX_SIZE, LOGO_MIN_SIZE};

static LOGO_NAME_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^logo\.(png|jpg|svg)$").unwrap()
});

fn after_doctype_end(s: &str) -> &str {
    s.find('>').map(|i| s[i + 1..].trim_start()).unwrap_or(s)
}

/// Validate a logo file from its raw bytes.
///
/// `filename` is the file's basename (e.g. "logo.png").
/// `logo_name` is the value from brand.json/store.json "logo" field, if available.
/// `path_label` is used for error reporting only.
pub fn validate_logo(
    content: &[u8],
    filename: &str,
    logo_name: Option<&str>,
    path_label: Option<&str>,
) -> ValidationResult {
    let mut result = ValidationResult::default();
    let path_str = path_label.map(|s| s.to_string());
    let parent_path = path_label.map(|p| {
        // Strip the filename to get the parent directory for logo_name errors
        p.rsplit_once('/').map(|(parent, _)| parent.to_string()).unwrap_or_else(|| p.to_string())
    });

    // Check if logo name contains "/"
    if let Some(name) = logo_name {
        if name.contains('/') {
            result.add(ValidationError::error(
                "Logo",
                format!("Logo path '{}' contains '/' - only use filename", name),
                parent_path,
            ));
        }
    }

    // Validate naming convention
    if !LOGO_NAME_RE.is_match(filename) {
        result.add(ValidationError::error(
            "Logo",
            format!(
                "Logo name '{}' must be 'logo.png', 'logo.jpg' or 'logo.svg'",
                filename
            ),
            path_str.clone(),
        ));
    }

    if filename.ends_with(".svg") {
        let content_str = String::from_utf8_lossy(content);
        let trimmed = content_str.trim_start();
        let after_decl = if trimmed.starts_with("<?xml") {
            trimmed.find("?>")
                .map(|i| trimmed[i + 2..].trim_start())
                .unwrap_or(trimmed)
        } else {
            trimmed
        };
        let after_doctype = if after_decl.to_lowercase().starts_with("<!doctype svg") {
            after_doctype_end(after_decl)
        } else {
            after_decl
        };
        // Skip XML comments (<!-- ... -->)
        let mut after_comments = after_doctype;
        while after_comments.starts_with("<!--") {
            after_comments = after_comments.find("-->")
                .map(|i| after_comments[i + 3..].trim_start())
                .unwrap_or(after_comments);
        }
        if !after_comments.to_lowercase().starts_with("<svg") {
            result.add(ValidationError::error(
                "Logo",
                "File has .svg extension but is not a valid SVG (root element is not <svg>)",
                path_str,
            ));
        }
    } else {
        // Validate dimensions for raster images
        match image::load_from_memory(content) {
            Ok(img) => {
                let (width, height) = img.dimensions();

                if width != height {
                    result.add(ValidationError::error(
                        "Logo",
                        format!(
                            "Logo must be square (width={}, height={})",
                            width, height
                        ),
                        path_str.clone(),
                    ));
                }

                if width < LOGO_MIN_SIZE || height < LOGO_MIN_SIZE {
                    result.add(ValidationError::error(
                        "Logo",
                        format!(
                            "Logo dimensions too small (minimum {}x{})",
                            LOGO_MIN_SIZE, LOGO_MIN_SIZE
                        ),
                        path_str.clone(),
                    ));
                }

                if width > LOGO_MAX_SIZE || height > LOGO_MAX_SIZE {
                    result.add(ValidationError::error(
                        "Logo",
                        format!(
                            "Logo dimensions too large (maximum {}x{})",
                            LOGO_MAX_SIZE, LOGO_MAX_SIZE
                        ),
                        path_str,
                    ));
                }
            }
            Err(e) => {
                result.add(ValidationError::error(
                    "Logo",
                    format!("Failed to read image: {}", e),
                    path_str,
                ));
            }
        }
    }

    result
}
