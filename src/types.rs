//! Core data types for OFD validation
//!
//! This module contains the fundamental data structures used throughout
//! the validation system, mirroring the Python implementation in types.py

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Severity level of a validation error
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ValidationLevel {
    Error,
    Warning,
}

impl std::fmt::Display for ValidationLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationLevel::Error => write!(f, "ERROR"),
            ValidationLevel::Warning => write!(f, "WARNING"),
        }
    }
}

/// Represents a single validation error or warning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub level: ValidationLevel,
    pub category: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<PathBuf>,
}

impl ValidationError {
    /// Create a new validation error
    pub fn new(
        level: ValidationLevel,
        category: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            level,
            category: category.into(),
            message: message.into(),
            path: None,
        }
    }

    /// Create a new error with a file path
    pub fn with_path(
        level: ValidationLevel,
        category: impl Into<String>,
        message: impl Into<String>,
        path: impl Into<PathBuf>,
    ) -> Self {
        Self {
            level,
            category: category.into(),
            message: message.into(),
            path: Some(path.into()),
        }
    }
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ref path) = self.path {
            write!(
                f,
                "{} - {}: {} [{}]",
                self.level,
                self.category,
                self.message,
                path.display()
            )
        } else {
            write!(f, "{} - {}: {}", self.level, self.category, self.message)
        }
    }
}

/// Aggregates validation errors and provides summary statistics
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub errors: Vec<ValidationError>,
}

impl ValidationResult {
    /// Create a new empty validation result
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
        }
    }

    /// Add a validation error to the result
    pub fn add_error(&mut self, error: ValidationError) {
        self.errors.push(error);
    }

    /// Merge another ValidationResult into this one
    pub fn merge(&mut self, other: ValidationResult) {
        self.errors.extend(other.errors);
    }

    /// Check if there are no ERROR-level issues
    pub fn is_valid(&self) -> bool {
        !self.errors.iter().any(|e| e.level == ValidationLevel::Error)
    }

    /// Count of ERROR-level issues
    pub fn error_count(&self) -> usize {
        self.errors
            .iter()
            .filter(|e| e.level == ValidationLevel::Error)
            .count()
    }

    /// Count of WARNING-level issues
    pub fn warning_count(&self) -> usize {
        self.errors
            .iter()
            .filter(|e| e.level == ValidationLevel::Warning)
            .count()
    }

    /// Convert to a JSON-serializable dictionary format
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "errors": self.errors.iter().map(|e| {
                serde_json::json!({
                    "level": e.level,
                    "category": &e.category,
                    "message": &e.message,
                    "path": e.path.as_ref().map(|p| p.display().to_string()),
                })
            }).collect::<Vec<_>>(),
            "error_count": self.error_count(),
            "warning_count": self.warning_count(),
            "is_valid": self.is_valid(),
        })
    }
}

/// Type of validation task to execute
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskType {
    Json,
    Logo,
    Folder,
}

/// Represents a validation task to be executed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationTask {
    pub task_type: TaskType,
    pub name: String,
    pub path: PathBuf,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra_data: Option<HashMap<String, serde_json::Value>>,
}

impl ValidationTask {
    /// Create a new validation task
    pub fn new(task_type: TaskType, name: impl Into<String>, path: impl Into<PathBuf>) -> Self {
        Self {
            task_type,
            name: name.into(),
            path: path.into(),
            extra_data: None,
        }
    }

    /// Create a task with extra data
    pub fn with_extra_data(
        task_type: TaskType,
        name: impl Into<String>,
        path: impl Into<PathBuf>,
        extra_data: HashMap<String, serde_json::Value>,
    ) -> Self {
        Self {
            task_type,
            name: name.into(),
            path: path.into(),
            extra_data: Some(extra_data),
        }
    }

    /// Get a string value from extra_data
    pub fn get_extra_string(&self, key: &str) -> Option<&str> {
        self.extra_data
            .as_ref()?
            .get(key)?
            .as_str()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_level() {
        assert_eq!(ValidationLevel::Error.to_string(), "ERROR");
        assert_eq!(ValidationLevel::Warning.to_string(), "WARNING");
    }

    #[test]
    fn test_validation_error() {
        let err = ValidationError::new(ValidationLevel::Error, "Test", "Test message");
        assert_eq!(err.level, ValidationLevel::Error);
        assert_eq!(err.category, "Test");
        assert_eq!(err.message, "Test message");
        assert!(err.path.is_none());
    }

    #[test]
    fn test_validation_error_with_path() {
        let err = ValidationError::with_path(
            ValidationLevel::Warning,
            "Test",
            "Test message",
            PathBuf::from("/test/path"),
        );
        assert_eq!(err.level, ValidationLevel::Warning);
        assert_eq!(err.path, Some(PathBuf::from("/test/path")));
    }

    #[test]
    fn test_validation_result() {
        let mut result = ValidationResult::new();
        assert!(result.is_valid());
        assert_eq!(result.error_count(), 0);
        assert_eq!(result.warning_count(), 0);

        result.add_error(ValidationError::new(
            ValidationLevel::Warning,
            "Test",
            "Warning message",
        ));
        assert!(result.is_valid()); // Still valid (only warning)
        assert_eq!(result.error_count(), 0);
        assert_eq!(result.warning_count(), 1);

        result.add_error(ValidationError::new(
            ValidationLevel::Error,
            "Test",
            "Error message",
        ));
        assert!(!result.is_valid()); // Now invalid
        assert_eq!(result.error_count(), 1);
        assert_eq!(result.warning_count(), 1);
    }

    #[test]
    fn test_validation_result_merge() {
        let mut result1 = ValidationResult::new();
        result1.add_error(ValidationError::new(
            ValidationLevel::Error,
            "Test",
            "Error 1",
        ));

        let mut result2 = ValidationResult::new();
        result2.add_error(ValidationError::new(
            ValidationLevel::Warning,
            "Test",
            "Warning 1",
        ));

        result1.merge(result2);
        assert_eq!(result1.error_count(), 1);
        assert_eq!(result1.warning_count(), 1);
    }

    #[test]
    fn test_validation_task() {
        let task = ValidationTask::new(TaskType::Json, "Test task", "/test/path");
        assert_eq!(task.task_type, TaskType::Json);
        assert_eq!(task.name, "Test task");
        assert_eq!(task.path, PathBuf::from("/test/path"));
        assert!(task.extra_data.is_none());
    }
}
