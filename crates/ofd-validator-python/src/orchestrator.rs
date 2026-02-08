use std::path::PathBuf;

use pyo3::prelude::*;
use rayon::ThreadPoolBuilder;

use ofd_validator_core as core;

use crate::types::ValidationResult;
use crate::util::log_step;

/// Run the closure on a custom Rayon thread pool when `max_workers` is set,
/// otherwise use the global pool (default Rayon behaviour).
fn with_thread_pool<F, R>(max_workers: Option<usize>, f: F) -> R
where
    F: FnOnce() -> R + Send,
    R: Send,
{
    match max_workers {
        Some(n) if n > 0 => {
            let pool = ThreadPoolBuilder::new()
                .num_threads(n)
                .build()
                .expect("failed to build Rayon thread pool");
            pool.install(f)
        }
        _ => f(),
    }
}

#[pyfunction]
#[pyo3(signature = (data_dir, stores_dir, schemas_dir=None, max_workers=None))]
pub fn validate_all(
    py: Python<'_>,
    data_dir: &str,
    stores_dir: &str,
    schemas_dir: Option<&str>,
    max_workers: Option<usize>,
) -> ValidationResult {
    let data_dir = PathBuf::from(data_dir);
    let stores_dir = PathBuf::from(stores_dir);
    let schemas_dir = PathBuf::from(schemas_dir.unwrap_or("schemas"));

    py.allow_threads(|| {
        with_thread_pool(max_workers, || {
            log_step("Loading dataset", None);
            let dataset = core::DataSet::from_directories(&data_dir, &stores_dir, &schemas_dir);

            log_step("Checking required files", None);
            log_step("Validating JSON schemas", Some(dataset.json_entries.len()));
            log_step("Validating logos", Some(dataset.logo_entries.len()));
            log_step("Validating folder names", Some(dataset.folder_entries.len()));
            log_step("Validating store IDs", None);
            log_step("Validating GTIN/EAN codes", None);

            core::validate_dataset(&dataset).into()
        })
    })
}

#[pyfunction]
#[pyo3(signature = (data_dir, stores_dir, schemas_dir=None, max_workers=None))]
pub fn validate_json_files(
    py: Python<'_>,
    data_dir: &str,
    stores_dir: &str,
    schemas_dir: Option<&str>,
    max_workers: Option<usize>,
) -> ValidationResult {
    let data_dir = PathBuf::from(data_dir);
    let stores_dir = PathBuf::from(stores_dir);
    let schemas_dir = PathBuf::from(schemas_dir.unwrap_or("schemas"));

    py.allow_threads(|| {
        with_thread_pool(max_workers, || {
            let dataset = core::DataSet::from_directories(&data_dir, &stores_dir, &schemas_dir);
            log_step("Validating JSON schemas", Some(dataset.json_entries.len()));

            use rayon::prelude::*;
            let results: Vec<core::ValidationResult> = dataset.json_entries
                .par_iter()
                .map(|(path, schema_name, data)| {
                    core::validators::validate_json(data, schema_name, &dataset.schema_cache, Some(path))
                })
                .collect();

            let mut result = core::ValidationResult::default();
            for r in results {
                result.merge_from(&r);
            }
            result.into()
        })
    })
}

#[pyfunction]
#[pyo3(signature = (data_dir, stores_dir, max_workers=None))]
pub fn validate_logo_files(
    py: Python<'_>,
    data_dir: &str,
    stores_dir: &str,
    max_workers: Option<usize>,
) -> ValidationResult {
    let data_dir = PathBuf::from(data_dir);
    let stores_dir = PathBuf::from(stores_dir);
    let schemas_dir = PathBuf::from("schemas");

    py.allow_threads(|| {
        with_thread_pool(max_workers, || {
            let dataset = core::DataSet::from_directories(&data_dir, &stores_dir, &schemas_dir);
            log_step("Validating logos", Some(dataset.logo_entries.len()));

            use rayon::prelude::*;
            let results: Vec<core::ValidationResult> = dataset.logo_entries
                .par_iter()
                .map(|(path, filename, bytes, logo_name)| {
                    if bytes.is_empty() {
                        let mut r = core::ValidationResult::default();
                        r.add(core::ValidationError::error(
                            "Logo",
                            "Logo file not found",
                            Some(path.clone()),
                        ));
                        r
                    } else {
                        core::validators::validate_logo(bytes, filename, logo_name.as_deref(), Some(path))
                    }
                })
                .collect();

            let mut result = core::ValidationResult::default();
            for r in results {
                result.merge_from(&r);
            }
            result.into()
        })
    })
}

#[pyfunction]
#[pyo3(signature = (data_dir, stores_dir, max_workers=None))]
pub fn validate_folder_names(
    py: Python<'_>,
    data_dir: &str,
    stores_dir: &str,
    max_workers: Option<usize>,
) -> ValidationResult {
    let data_dir = PathBuf::from(data_dir);
    let stores_dir = PathBuf::from(stores_dir);
    let schemas_dir = PathBuf::from("schemas");

    py.allow_threads(|| {
        with_thread_pool(max_workers, || {
            let dataset = core::DataSet::from_directories(&data_dir, &stores_dir, &schemas_dir);
            log_step("Validating folder names", Some(dataset.folder_entries.len()));

            use rayon::prelude::*;
            let results: Vec<core::ValidationResult> = dataset.folder_entries
                .par_iter()
                .map(|(path, folder_name, json_data, json_key)| {
                    core::validators::validate_folder_name(folder_name, json_data, json_key, Some(path))
                })
                .collect();

            let mut result = core::ValidationResult::default();
            for r in results {
                result.merge_from(&r);
            }
            result.into()
        })
    })
}
