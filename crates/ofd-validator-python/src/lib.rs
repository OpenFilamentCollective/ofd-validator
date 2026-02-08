use pyo3::prelude::*;

mod orchestrator;
mod types;
mod util;
mod validators;

use orchestrator::{
    validate_all, validate_folder_names, validate_json_files, validate_logo_files,
};
use types::{ValidationError, ValidationLevel, ValidationResult};
use validators::{
    validate_folder_name, validate_gtin_ean, validate_logo_file, validate_required_files,
    validate_store_ids,
};

#[pymodule]
fn ofd_validator(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<ValidationLevel>()?;
    m.add_class::<ValidationError>()?;
    m.add_class::<ValidationResult>()?;

    // Orchestrated batch validators (internally parallel with rayon)
    m.add_function(wrap_pyfunction!(validate_all, m)?)?;
    m.add_function(wrap_pyfunction!(validate_json_files, m)?)?;
    m.add_function(wrap_pyfunction!(validate_logo_files, m)?)?;
    m.add_function(wrap_pyfunction!(validate_folder_names, m)?)?;

    // Individual validators
    m.add_function(wrap_pyfunction!(validate_store_ids, m)?)?;
    m.add_function(wrap_pyfunction!(validate_gtin_ean, m)?)?;
    m.add_function(wrap_pyfunction!(validate_required_files, m)?)?;
    m.add_function(wrap_pyfunction!(validate_logo_file, m)?)?;
    m.add_function(wrap_pyfunction!(validate_folder_name, m)?)?;

    Ok(())
}
