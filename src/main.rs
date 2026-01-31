//! OFD Validator CLI
//!
//! Standalone command-line interface for the OFD validator.
//! Provides the same functionality as the Python CLI but with better performance.
//!
//! This binary is only compiled when the "binary" feature is enabled.

#[cfg(feature = "binary")]
use clap::{Parser, Subcommand};
#[cfg(feature = "binary")]
use ofd_validator::ValidationOrchestrator;
#[cfg(feature = "binary")]
use std::path::PathBuf;
#[cfg(feature = "binary")]
use std::process;

#[cfg(feature = "binary")]
#[derive(Parser)]
#[command(name = "ofd-validator")]
#[command(about = "High-performance validator for the Open Filament Database", long_about = None)]
struct Cli {
    /// Path to the data directory
    #[arg(long, default_value = "data")]
    data_dir: PathBuf,

    /// Path to the stores directory
    #[arg(long, default_value = "stores")]
    stores_dir: PathBuf,

    /// Output results as JSON
    #[arg(long)]
    json: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[cfg(feature = "binary")]
#[derive(Subcommand)]
enum Commands {
    /// Validate all files (default)
    All,
    /// Validate JSON files against schemas
    JsonFiles,
    /// Validate logo files
    Logos,
    /// Validate folder names
    FolderNames,
    /// Validate store ID references
    StoreIds,
    /// Validate GTIN/EAN fields
    Gtin,
    /// Check for missing required files
    MissingFiles,
}

#[cfg(feature = "binary")]
fn main() {
    let cli = Cli::parse();

    let orchestrator = ValidationOrchestrator::new(&cli.data_dir, &cli.stores_dir);

    let result = match cli.command {
        Some(Commands::JsonFiles) => orchestrator.validate_json_files(),
        Some(Commands::Logos) => orchestrator.validate_logo_files(),
        Some(Commands::FolderNames) => orchestrator.validate_folder_names(),
        Some(Commands::StoreIds) => orchestrator.validate_store_ids(),
        Some(Commands::Gtin) => orchestrator.validate_gtin(),
        Some(Commands::MissingFiles) => orchestrator.validate_missing_files(),
        Some(Commands::All) | None => orchestrator.validate_all(),
    };

    if cli.json {
        // Output as JSON
        let json = serde_json::to_string_pretty(&result.to_json_value()).unwrap();
        println!("{}", json);
    } else {
        // Human-readable output
        if result.errors.is_empty() {
            println!("All validations passed!");
        } else {
            // Group errors by category
            use std::collections::HashMap;
            let mut errors_by_category: HashMap<String, Vec<_>> = HashMap::new();

            for error in &result.errors {
                errors_by_category
                    .entry(error.category.clone())
                    .or_insert_with(Vec::new)
                    .push(error);
            }

            // Print errors grouped by category
            let mut categories: Vec<_> = errors_by_category.keys().collect();
            categories.sort();

            for category in categories {
                let errors = &errors_by_category[category];
                println!("\n{} ({}):", category, errors.len());
                println!("{}", "-".repeat(80));

                for error in errors {
                    println!("  {}", error);
                }
            }

            println!(
                "\nValidation failed: {} errors, {} warnings",
                result.error_count(),
                result.warning_count()
            );
        }
    }

    // Exit with appropriate code
    if result.is_valid() {
        process::exit(0);
    } else {
        process::exit(1);
    }
}

// When binary feature is not enabled, provide a dummy main
#[cfg(not(feature = "binary"))]
fn main() {
    eprintln!("This binary requires the 'binary' feature to be enabled.");
    eprintln!("Build with: cargo build --features binary");
    std::process::exit(1);
}
