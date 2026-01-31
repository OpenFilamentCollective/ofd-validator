//! PyO3 Python bindings for the OFD validator
//!
//! This module provides Python bindings using PyO3, allowing the Rust validator
//! to be imported and used from Python code.
//!
//! Conditional compilation: Only included when the "python" feature is enabled.

#[cfg(feature = "python")]
use pyo3::prelude::*;

#[cfg(feature = "python")]
use crate::ValidationOrchestrator;

/// Python wrapper for ValidationOrchestrator
#[cfg(feature = "python")]
#[pyclass(name = "PyValidationOrchestrator")]
pub struct PyValidationOrchestrator {
    inner: ValidationOrchestrator,
}

#[cfg(feature = "python")]
#[pymethods]
impl PyValidationOrchestrator {
    /// Create a new validation orchestrator
    ///
    /// # Arguments
    /// * `data_dir` - Path to the data directory
    /// * `stores_dir` - Path to the stores directory
    #[new]
    fn new(data_dir: String, stores_dir: String) -> PyResult<Self> {
        Ok(Self {
            inner: ValidationOrchestrator::new(data_dir, stores_dir),
        })
    }

    /// Run all validations and return results as JSON string
    fn validate_all(&self) -> PyResult<String> {
        let result = self.inner.validate_all();
        let json = serde_json::to_string(&result.to_json_value())
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        Ok(json)
    }

    /// Validate JSON files and return results as JSON string
    fn validate_json_files(&self) -> PyResult<String> {
        let result = self.inner.validate_json_files();
        let json = serde_json::to_string(&result.to_json_value())
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        Ok(json)
    }

    /// Validate logo files and return results as JSON string
    fn validate_logo_files(&self) -> PyResult<String> {
        let result = self.inner.validate_logo_files();
        let json = serde_json::to_string(&result.to_json_value())
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        Ok(json)
    }

    /// Validate folder names and return results as JSON string
    fn validate_folder_names(&self) -> PyResult<String> {
        let result = self.inner.validate_folder_names();
        let json = serde_json::to_string(&result.to_json_value())
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        Ok(json)
    }

    /// Validate store IDs and return results as JSON string
    fn validate_store_ids(&self) -> PyResult<String> {
        let result = self.inner.validate_store_ids();
        let json = serde_json::to_string(&result.to_json_value())
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        Ok(json)
    }

    /// Validate GTIN/EAN and return results as JSON string
    fn validate_gtin(&self) -> PyResult<String> {
        let result = self.inner.validate_gtin();
        let json = serde_json::to_string(&result.to_json_value())
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        Ok(json)
    }

    /// Check for missing files and return results as JSON string
    fn validate_missing_files(&self) -> PyResult<String> {
        let result = self.inner.validate_missing_files();
        let json = serde_json::to_string(&result.to_json_value())
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        Ok(json)
    }
}

/// Python module definition
#[cfg(feature = "python")]
#[pymodule]
fn ofd_validator(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyValidationOrchestrator>()?;
    Ok(())
}

// Dummy module for when python feature is not enabled
#[cfg(not(feature = "python"))]
pub fn python_not_enabled() {
    // This function exists only to make the module compile without the python feature
}
