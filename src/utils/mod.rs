//! Utility modules for validation
//!
//! This module contains shared utilities used across all validators:
//! - helpers: Common functions for JSON loading, path manipulation
//! - image_fast: Optimized image dimension reading
//! - schema_cache: Lazy-loading JSON schema cache
//! - parallel: Parallel task execution with Rayon

pub mod helpers;
pub mod image_fast;
pub mod parallel;
pub mod schema_cache;

// Re-export commonly used items
pub use helpers::{cleanse_folder_name, get_json_string, load_json};
pub use image_fast::get_image_dimensions;
pub use parallel::{run_tasks_parallel, ParallelConfig};
pub use schema_cache::SchemaCache;
