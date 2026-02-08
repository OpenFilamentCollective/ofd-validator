use std::path::Path;
use regex::Regex;
use std::sync::LazyLock;

use crate::types::{ValidationError, ValidationResult};
use crate::util::{LOGO_MAX_SIZE, LOGO_MIN_SIZE};

static LOGO_NAME_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^logo\.(png|jpg|svg)$").unwrap()
});

pub fn validate_logo_file_impl(
    logo_path: &Path,
    logo_name: Option<&str>,
) -> ValidationResult {
    let mut result = ValidationResult::default();

    // Check if logo name contains "/"
    if let Some(name) = logo_name {
        if name.contains('/') {
            result.add(ValidationError::error(
                "Logo",
                format!("Logo path '{}' contains '/' - only use filename", name),
                logo_path.parent().map(|p| p.to_string_lossy().to_string()),
            ));
        }
    }

    if !logo_path.exists() {
        result.add(ValidationError::error(
            "Logo",
            "Logo file not found",
            Some(logo_path.to_string_lossy().to_string()),
        ));
        return result;
    }

    // Validate naming convention
    let filename = match logo_path.file_name() {
        Some(name) => name.to_string_lossy().to_string(),
        None => return result,
    };

    if !LOGO_NAME_RE.is_match(&filename) {
        result.add(ValidationError::error(
            "Logo",
            format!(
                "Logo name '{}' must be 'logo.png', 'logo.jpg' or 'logo.svg'",
                filename
            ),
            Some(logo_path.to_string_lossy().to_string()),
        ));
    }

    // Validate dimensions for raster images (skip SVG)
    if !filename.ends_with(".svg") {
        match image::image_dimensions(logo_path) {
            Ok((width, height)) => {
                if width != height {
                    result.add(ValidationError::error(
                        "Logo",
                        format!(
                            "Logo must be square (width={}, height={})",
                            width, height
                        ),
                        Some(logo_path.to_string_lossy().to_string()),
                    ));
                }

                if width < LOGO_MIN_SIZE || height < LOGO_MIN_SIZE {
                    result.add(ValidationError::error(
                        "Logo",
                        format!(
                            "Logo dimensions too small (minimum {}x{})",
                            LOGO_MIN_SIZE, LOGO_MIN_SIZE
                        ),
                        Some(logo_path.to_string_lossy().to_string()),
                    ));
                }

                if width > LOGO_MAX_SIZE || height > LOGO_MAX_SIZE {
                    result.add(ValidationError::error(
                        "Logo",
                        format!(
                            "Logo dimensions too large (maximum {}x{})",
                            LOGO_MAX_SIZE, LOGO_MAX_SIZE
                        ),
                        Some(logo_path.to_string_lossy().to_string()),
                    ));
                }
            }
            Err(e) => {
                result.add(ValidationError::error(
                    "Logo",
                    format!("Failed to read image: {}", e),
                    Some(logo_path.to_string_lossy().to_string()),
                ));
            }
        }
    }

    result
}
