//! Logo file validator
//!
//! Validates logo files for:
//! - Existence
//! - Naming convention (logo.png, logo.jpg, logo.svg)
//! - Dimensions (100-400px, square) for raster images
//!
//! Uses optimized header parsing for PNG/JPEG (5-10x faster than full image loading)

use crate::types::{ValidationError, ValidationLevel, ValidationResult};
use crate::utils::{get_image_dimensions, helpers, load_json};
use regex::Regex;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const LOGO_MIN_SIZE: u32 = 100;
const LOGO_MAX_SIZE: u32 = 400;

lazy_static::lazy_static! {
    static ref LOGO_NAME_PATTERN: Regex = Regex::new(r"^logo\.(png|jpg|svg)$").unwrap();
}

/// Logo file validator
pub struct LogoValidator;

impl LogoValidator {
    /// Create a new logo validator
    pub fn new() -> Self {
        Self
    }

    /// Validate a single logo file
    ///
    /// # Arguments
    /// * `logo_path` - Path to the logo file
    /// * `logo_name` - Optional expected logo filename
    ///
    /// # Returns
    /// ValidationResult containing any errors found
    pub fn validate_logo_file<P: AsRef<Path>>(
        &self,
        logo_path: P,
        logo_name: Option<&str>,
    ) -> ValidationResult {
        let logo_path = logo_path.as_ref();
        let mut result = ValidationResult::new();

        // Check if logo name contains "/"  (should be just filename)
        if let Some(name) = logo_name {
            if name.contains('/') {
                result.add_error(ValidationError::with_path(
                    ValidationLevel::Error,
                    "Logo",
                    format!("Logo path '{}' contains '/' - only use filename", name),
                    logo_path.parent().unwrap_or(logo_path),
                ));
            }
        }

        // Check if file exists
        if !logo_path.exists() {
            result.add_error(ValidationError::with_path(
                ValidationLevel::Error,
                "Logo",
                "Logo file not found",
                logo_path,
            ));
            return result;
        }

        // Validate naming convention
        if let Some(filename) = logo_path.file_name().and_then(|n| n.to_str()) {
            if !LOGO_NAME_PATTERN.is_match(filename) {
                result.add_error(ValidationError::with_path(
                    ValidationLevel::Error,
                    "Logo",
                    format!(
                        "Logo name '{}' must be 'logo.png', 'logo.jpg' or 'logo.svg'",
                        filename
                    ),
                    logo_path,
                ));
            }
        }

        // Validate dimensions for raster images (skip SVG)
        let is_svg = logo_path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("svg"))
            .unwrap_or(false);

        if !is_svg {
            match get_image_dimensions(logo_path) {
                Ok((width, height)) => {
                    // Check if square
                    if width != height {
                        result.add_error(ValidationError::with_path(
                            ValidationLevel::Error,
                            "Logo",
                            format!("Logo must be square (width={}, height={})", width, height),
                            logo_path,
                        ));
                    }

                    // Check minimum size
                    if width < LOGO_MIN_SIZE || height < LOGO_MIN_SIZE {
                        result.add_error(ValidationError::with_path(
                            ValidationLevel::Error,
                            "Logo",
                            format!(
                                "Logo dimensions too small (minimum {}x{})",
                                LOGO_MIN_SIZE, LOGO_MIN_SIZE
                            ),
                            logo_path,
                        ));
                    }

                    // Check maximum size
                    if width > LOGO_MAX_SIZE || height > LOGO_MAX_SIZE {
                        result.add_error(ValidationError::with_path(
                            ValidationLevel::Error,
                            "Logo",
                            format!(
                                "Logo dimensions too large (maximum {}x{})",
                                LOGO_MAX_SIZE, LOGO_MAX_SIZE
                            ),
                            logo_path,
                        ));
                    }
                }
                Err(e) => {
                    result.add_error(ValidationError::with_path(
                        ValidationLevel::Error,
                        "Logo",
                        format!("Failed to read image: {}", e),
                        logo_path,
                    ));
                }
            }
        }

        result
    }

    /// Collect all logo validation tasks from data and stores directories
    ///
    /// # Arguments
    /// * `data_dir` - Path to the data directory
    /// * `stores_dir` - Path to the stores directory
    ///
    /// # Returns
    /// Vector of (logo_path, logo_name) pairs to validate
    pub fn collect_validation_tasks<P: AsRef<Path>>(
        data_dir: P,
        stores_dir: P,
    ) -> Vec<(PathBuf, Option<String>)> {
        let mut tasks = Vec::new();

        let data_dir = data_dir.as_ref();
        let stores_dir = stores_dir.as_ref();

        // Collect brand logos
        if data_dir.exists() {
            for brand_entry in WalkDir::new(data_dir).min_depth(1).max_depth(1) {
                let brand_entry = match brand_entry {
                    Ok(e) => e,
                    Err(_) => continue,
                };

                if !brand_entry.file_type().is_dir() {
                    continue;
                }

                let brand_dir = brand_entry.path();
                let brand_file = brand_dir.join("brand.json");

                if let Ok(Some(data)) = load_json(&brand_file) {
                    if let Some(logo_name) = helpers::get_json_string(&data, "logo") {
                        let logo_path = brand_dir.join(&logo_name);
                        tasks.push((logo_path, Some(logo_name)));
                    }
                }
            }
        }

        // Collect store logos
        if stores_dir.exists() {
            for store_entry in WalkDir::new(stores_dir).min_depth(1).max_depth(1) {
                let store_entry = match store_entry {
                    Ok(e) => e,
                    Err(_) => continue,
                };

                if !store_entry.file_type().is_dir() {
                    continue;
                }

                let store_dir = store_entry.path();
                let store_file = store_dir.join("store.json");

                if let Ok(Some(data)) = load_json(&store_file) {
                    if let Some(logo_name) = helpers::get_json_string(&data, "logo") {
                        let logo_path = store_dir.join(&logo_name);
                        tasks.push((logo_path, Some(logo_name)));
                    }
                }
            }
        }

        tasks
    }

    /// Validate all logo files in data and stores directories
    ///
    /// # Arguments
    /// * `data_dir` - Path to the data directory
    /// * `stores_dir` - Path to the stores directory
    ///
    /// # Returns
    /// Aggregated ValidationResult for all logos
    pub fn validate_all<P: AsRef<Path>>(
        &self,
        data_dir: P,
        stores_dir: P,
    ) -> ValidationResult {
        let tasks = Self::collect_validation_tasks(data_dir, stores_dir);
        let mut result = ValidationResult::new();

        for (logo_path, logo_name) in tasks {
            let file_result = self.validate_logo_file(&logo_path, logo_name.as_deref());
            result.merge(file_result);
        }

        result
    }
}

impl Default for LogoValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logo_name_pattern() {
        assert!(LOGO_NAME_PATTERN.is_match("logo.png"));
        assert!(LOGO_NAME_PATTERN.is_match("logo.jpg"));
        assert!(LOGO_NAME_PATTERN.is_match("logo.svg"));
        assert!(!LOGO_NAME_PATTERN.is_match("logo.gif"));
        assert!(!LOGO_NAME_PATTERN.is_match("image.png"));
        assert!(!LOGO_NAME_PATTERN.is_match("logo_test.png"));
    }

    #[test]
    fn test_logo_validator_creation() {
        let validator = LogoValidator::new();
        let _ = validator; // Ensure it compiles
    }
}
