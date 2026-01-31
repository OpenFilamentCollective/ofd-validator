//! Parallel execution utilities for validation tasks
//!
//! This module provides high-performance parallel processing using Rayon,
//! allowing validation tasks to run across multiple CPU cores efficiently.

use rayon::prelude::*;
use std::sync::{Arc, Mutex};

use crate::types::{ValidationResult, ValidationTask};

/// Configuration for parallel execution
#[derive(Debug, Clone)]
pub struct ParallelConfig {
    /// Maximum number of worker threads (None = CPU count - 2)
    pub max_workers: Option<usize>,
}

impl Default for ParallelConfig {
    fn default() -> Self {
        // Default to CPU count - 2, minimum of 1
        let cpu_count = num_cpus::get();
        let workers = std::cmp::max(1, cpu_count.saturating_sub(2));

        Self {
            max_workers: Some(workers),
        }
    }
}

impl ParallelConfig {
    /// Create a new configuration with specified worker count
    pub fn new(max_workers: Option<usize>) -> Self {
        Self { max_workers }
    }

    /// Get the number of workers to use
    pub fn worker_count(&self) -> usize {
        self.max_workers.unwrap_or_else(|| {
            let cpu_count = num_cpus::get();
            std::cmp::max(1, cpu_count.saturating_sub(2))
        })
    }

    /// Build a Rayon thread pool with the configured worker count
    pub fn build_thread_pool(&self) -> rayon::ThreadPool {
        rayon::ThreadPoolBuilder::new()
            .num_threads(self.worker_count())
            .build()
            .expect("Failed to build thread pool")
    }
}

/// Execute validation tasks in parallel
///
/// This function runs validation tasks concurrently using Rayon,
/// automatically managing thread synchronization and result aggregation.
///
/// # Arguments
/// * `tasks` - Vector of validation tasks to execute
/// * `executor` - Function that executes a single task and returns a ValidationResult
/// * `config` - Parallel execution configuration
///
/// # Returns
/// Aggregated ValidationResult containing all errors from all tasks
pub fn run_tasks_parallel<F>(
    tasks: Vec<ValidationTask>,
    executor: F,
    config: &ParallelConfig,
) -> ValidationResult
where
    F: Fn(&ValidationTask) -> ValidationResult + Send + Sync,
{
    if tasks.is_empty() {
        return ValidationResult::new();
    }

    // Build thread pool with configured worker count
    let pool = config.build_thread_pool();

    // Wrap executor in Arc for sharing across threads
    let executor = Arc::new(executor);

    // Shared result accumulator
    let accumulated_result = Arc::new(Mutex::new(ValidationResult::new()));

    // Execute tasks in parallel using the thread pool
    pool.install(|| {
        tasks.par_iter().for_each(|task| {
            let result = executor(task);

            // Merge result into accumulated result (thread-safe)
            let mut acc = accumulated_result.lock().unwrap();
            acc.merge(result);
        });
    });

    // Extract final result
    Arc::try_unwrap(accumulated_result)
        .unwrap()
        .into_inner()
        .unwrap()
}

/// Execute validation tasks in parallel and collect results as a vector
///
/// Unlike `run_tasks_parallel`, this function collects individual results
/// without merging them, useful for debugging or separate error handling.
pub fn run_tasks_parallel_collect<F>(
    tasks: Vec<ValidationTask>,
    executor: F,
    config: &ParallelConfig,
) -> Vec<ValidationResult>
where
    F: Fn(&ValidationTask) -> ValidationResult + Send + Sync,
{
    if tasks.is_empty() {
        return Vec::new();
    }

    let pool = config.build_thread_pool();
    let executor = Arc::new(executor);

    pool.install(|| tasks.par_iter().map(|task| executor(task)).collect())
}

/// Get the default number of workers (CPU count - 2, min 1)
pub fn default_worker_count() -> usize {
    let cpu_count = num_cpus::get();
    std::cmp::max(1, cpu_count.saturating_sub(2))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{TaskType, ValidationError, ValidationLevel};
    use std::path::PathBuf;

    #[test]
    fn test_parallel_config_default() {
        let config = ParallelConfig::default();
        let worker_count = config.worker_count();

        // Should be at least 1
        assert!(worker_count >= 1);

        // Should be CPU count - 2 or 1, whichever is larger
        let cpu_count = num_cpus::get();
        let expected = std::cmp::max(1, cpu_count.saturating_sub(2));
        assert_eq!(worker_count, expected);
    }

    #[test]
    fn test_parallel_config_custom() {
        let config = ParallelConfig::new(Some(4));
        assert_eq!(config.worker_count(), 4);
    }

    #[test]
    fn test_run_tasks_parallel_empty() {
        let tasks: Vec<ValidationTask> = Vec::new();
        let config = ParallelConfig::default();

        let result = run_tasks_parallel(tasks, |_| ValidationResult::new(), &config);

        assert!(result.is_valid());
        assert_eq!(result.error_count(), 0);
    }

    #[test]
    fn test_run_tasks_parallel() {
        // Create test tasks
        let tasks: Vec<ValidationTask> = (0..10)
            .map(|i| {
                ValidationTask::new(
                    TaskType::Json,
                    format!("Task {}", i),
                    PathBuf::from(format!("/test/{}.json", i)),
                )
            })
            .collect();

        let config = ParallelConfig::default();

        // Execute tasks with a simple executor that adds an error for each task
        let result = run_tasks_parallel(
            tasks,
            |task| {
                let mut res = ValidationResult::new();
                res.add_error(ValidationError::new(
                    ValidationLevel::Error,
                    "Test",
                    format!("Error from {}", task.name),
                ));
                res
            },
            &config,
        );

        // Should have 10 errors (one per task)
        assert_eq!(result.error_count(), 10);
        assert!(!result.is_valid());
    }

    #[test]
    fn test_default_worker_count() {
        let count = default_worker_count();
        assert!(count >= 1);

        let cpu_count = num_cpus::get();
        let expected = std::cmp::max(1, cpu_count.saturating_sub(2));
        assert_eq!(count, expected);
    }
}
