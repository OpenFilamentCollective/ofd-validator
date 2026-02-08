use pyo3::prelude::*;
use pyo3::types::PyDict;

use ofd_validator_core as core;

#[pyclass(eq, eq_int)]
#[derive(Clone, Debug, PartialEq)]
pub enum ValidationLevel {
    Error,
    Warning,
}

impl From<core::ValidationLevel> for ValidationLevel {
    fn from(level: core::ValidationLevel) -> Self {
        match level {
            core::ValidationLevel::Error => ValidationLevel::Error,
            core::ValidationLevel::Warning => ValidationLevel::Warning,
        }
    }
}

impl std::fmt::Display for ValidationLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationLevel::Error => write!(f, "ERROR"),
            ValidationLevel::Warning => write!(f, "WARNING"),
        }
    }
}

#[pymethods]
impl ValidationLevel {
    #[getter]
    fn value(&self) -> &str {
        match self {
            ValidationLevel::Error => "ERROR",
            ValidationLevel::Warning => "WARNING",
        }
    }

    fn __str__(&self) -> String {
        self.to_string()
    }

    fn __repr__(&self) -> String {
        format!("ValidationLevel.{}", match self {
            ValidationLevel::Error => "Error",
            ValidationLevel::Warning => "Warning",
        })
    }
}

#[pyclass]
#[derive(Clone, Debug)]
pub struct ValidationError {
    #[pyo3(get)]
    pub level: ValidationLevel,
    #[pyo3(get)]
    pub category: String,
    #[pyo3(get)]
    pub message: String,
    #[pyo3(get)]
    pub path: Option<String>,
}

impl From<core::ValidationError> for ValidationError {
    fn from(e: core::ValidationError) -> Self {
        Self {
            level: e.level.into(),
            category: e.category,
            message: e.message,
            path: e.path,
        }
    }
}

#[pymethods]
impl ValidationError {
    #[new]
    #[pyo3(signature = (level, category, message, path=None))]
    fn new(level: ValidationLevel, category: String, message: String, path: Option<String>) -> Self {
        Self { level, category, message, path }
    }

    fn __str__(&self) -> String {
        let path_str = match &self.path {
            Some(p) => format!(" [{}]", p),
            None => String::new(),
        };
        format!("{} - {}: {}{}", self.level, self.category, self.message, path_str)
    }

    fn __repr__(&self) -> String {
        self.__str__()
    }

    fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let dict = PyDict::new(py);
        dict.set_item("level", self.level.value())?;
        dict.set_item("category", &self.category)?;
        dict.set_item("message", &self.message)?;
        dict.set_item("path", &self.path)?;
        Ok(dict)
    }
}

#[pyclass]
#[derive(Clone, Debug, Default)]
pub struct ValidationResult {
    pub errors: Vec<ValidationError>,
}

impl From<core::ValidationResult> for ValidationResult {
    fn from(r: core::ValidationResult) -> Self {
        Self {
            errors: r.errors.into_iter().map(|e| e.into()).collect(),
        }
    }
}

impl ValidationResult {
    pub fn count_errors(&self) -> usize {
        self.errors.iter().filter(|e| e.level == ValidationLevel::Error).count()
    }

    pub fn count_warnings(&self) -> usize {
        self.errors.iter().filter(|e| e.level == ValidationLevel::Warning).count()
    }

    pub fn is_valid_check(&self) -> bool {
        !self.errors.iter().any(|e| e.level == ValidationLevel::Error)
    }
}

#[pymethods]
impl ValidationResult {
    #[new]
    fn new() -> Self {
        Self::default()
    }

    fn add_error(&mut self, error: ValidationError) {
        self.errors.push(error);
    }

    fn merge(&mut self, other: &ValidationResult) {
        self.errors.extend(other.errors.iter().cloned());
    }

    #[getter]
    fn errors(&self) -> Vec<ValidationError> {
        self.errors.clone()
    }

    #[getter]
    fn is_valid(&self) -> bool {
        self.is_valid_check()
    }

    #[getter]
    fn error_count(&self) -> usize {
        self.count_errors()
    }

    #[getter]
    fn warning_count(&self) -> usize {
        self.count_warnings()
    }

    fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let dict = PyDict::new(py);

        let error_dicts: Vec<Bound<'py, PyDict>> = self.errors
            .iter()
            .map(|e| e.to_dict(py))
            .collect::<PyResult<_>>()?;
        dict.set_item("errors", error_dicts)?;
        dict.set_item("error_count", self.count_errors())?;
        dict.set_item("warning_count", self.count_warnings())?;
        dict.set_item("is_valid", self.is_valid_check())?;

        Ok(dict)
    }

    fn __str__(&self) -> String {
        format!(
            "ValidationResult(errors={}, warnings={}, valid={})",
            self.count_errors(),
            self.count_warnings(),
            self.is_valid_check()
        )
    }

    fn __repr__(&self) -> String {
        self.__str__()
    }
}
