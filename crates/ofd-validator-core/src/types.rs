use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Serialize)]
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

#[derive(Clone, Debug, Serialize)]
pub struct ValidationError {
    pub level: ValidationLevel,
    pub category: String,
    pub message: String,
    pub path: Option<String>,
}

impl ValidationError {
    pub fn error(category: impl Into<String>, message: impl Into<String>, path: Option<String>) -> Self {
        Self {
            level: ValidationLevel::Error,
            category: category.into(),
            message: message.into(),
            path,
        }
    }

    pub fn warning(category: impl Into<String>, message: impl Into<String>, path: Option<String>) -> Self {
        Self {
            level: ValidationLevel::Warning,
            category: category.into(),
            message: message.into(),
            path,
        }
    }
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.path {
            Some(p) => write!(f, "{} - {}: {} [{}]", self.level, self.category, self.message, p),
            None => write!(f, "{} - {}: {}", self.level, self.category, self.message),
        }
    }
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct ValidationResult {
    pub errors: Vec<ValidationError>,
}

impl ValidationResult {
    pub fn add(&mut self, error: ValidationError) {
        self.errors.push(error);
    }

    pub fn merge_from(&mut self, other: &ValidationResult) {
        self.errors.extend(other.errors.iter().cloned());
    }

    pub fn is_valid(&self) -> bool {
        !self.errors.iter().any(|e| e.level == ValidationLevel::Error)
    }

    pub fn error_count(&self) -> usize {
        self.errors.iter().filter(|e| e.level == ValidationLevel::Error).count()
    }

    pub fn warning_count(&self) -> usize {
        self.errors.iter().filter(|e| e.level == ValidationLevel::Warning).count()
    }
}
