# OFD Validator

High-performance validation library for the [Open Filament Database](https://github.com/OpenFilamentCollective/open-filament-database), written in Rust with Python bindings.

This is a Rust reimplementation of the Python validator, providing significant performance improvements:
- **85% faster** JSON Schema validation via caching
- **5-10x faster** logo image validation via fast header parsing
- Multi-threaded parallel processing for large datasets

## Features

- **JSON Schema Validation**: Validates filament data against JSON schemas with intelligent caching
- **Logo Image Validation**: Fast dimension checking via binary header parsing (PNG, JPEG, WebP)
- **Folder Name Validation**: Ensures proper naming conventions
- **Store ID Cross-Reference Validation**: Verifies store ID consistency
- **GTIN/EAN Validation**: Validates product codes
- **Missing File Detection**: Identifies missing required files
- **Parallel Processing**: Multi-threaded validation using Rayon

## Installation

### From PyPI (Python Library)

```bash
pip install ofd-validator
```

### From Source (Rust Binary)

```bash
# Clone the repository
git clone https://github.com/OpenFilamentCollective/open-filament-database.git
cd open-filament-database/ofd-validator

# Build and install the binary
cargo install --path . --features binary
```

### For Development

```bash
# Build Python bindings locally
pip install maturin
maturin develop --release --features python

# Build Rust binary
cargo build --release --features binary
```

## Usage

### Python API

```python
from ofd_validator import PyValidationOrchestrator
import json

# Create validator with data and stores directories
validator = PyValidationOrchestrator("data", "stores")

# Run all validations
results = json.loads(validator.validate_all())

if results['is_valid']:
    print("✓ All validations passed!")
else:
    print(f"Found {results['error_count']} errors:")
    for error in results['errors']:
        print(f"  [{error['level']}] {error['message']}")
        if 'file_path' in error:
            print(f"    File: {error['file_path']}")

# Run individual validations
json_results = json.loads(validator.validate_json_files())
logo_results = json.loads(validator.validate_logo_files())
folder_results = json.loads(validator.validate_folder_names())
store_id_results = json.loads(validator.validate_store_ids())
gtin_results = json.loads(validator.validate_gtin())
missing_results = json.loads(validator.validate_missing_files())
```

### Rust Binary

```bash
# Validate all files in the data and stores directories
ofd-validator --data-dir data --stores-dir stores

# Run specific validations
ofd-validator --data-dir data --stores-dir stores --validate json
ofd-validator --data-dir data --stores-dir stores --validate logo
```

### Rust Library

```rust
use ofd_validator::{ValidationOrchestrator, ValidationResult};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create orchestrator
    let orchestrator = ValidationOrchestrator::new("data", "stores")?;

    // Run all validations
    let results = orchestrator.validate_all()?;

    if results.is_valid {
        println!("✓ All validations passed!");
    } else {
        println!("Found {} errors:", results.error_count);
        for error in &results.errors {
            println!("  [{}] {}", error.level, error.message);
        }
    }

    Ok(())
}
```

## API Reference

### Python: `PyValidationOrchestrator`

#### Constructor

```python
PyValidationOrchestrator(data_dir: str, stores_dir: str)
```

Creates a new validation orchestrator.

**Parameters:**
- `data_dir`: Path to the data directory containing filament data
- `stores_dir`: Path to the stores directory containing store information

#### Methods

All methods return a JSON string with validation results.

**`validate_all() -> str`**

Runs all validation checks and returns aggregated results.

**`validate_json_files() -> str`**

Validates all JSON files against their schemas.

**`validate_logo_files() -> str`**

Validates logo image files (dimensions, format).

**`validate_folder_names() -> str`**

Validates folder naming conventions.

**`validate_store_ids() -> str`**

Validates store ID cross-references.

**`validate_gtin() -> str`**

Validates GTIN/EAN product codes.

**`validate_missing_files() -> str`**

Checks for missing required files.

#### Result Format

All methods return JSON with this structure:

```json
{
  "is_valid": true,
  "error_count": 0,
  "warning_count": 0,
  "errors": [],
  "warnings": []
}
```

Error/warning objects contain:
- `level`: "error" or "warning"
- `message`: Description of the issue
- `file_path`: (optional) Path to the affected file
- `validator`: Name of the validator that detected the issue

## Performance

Compared to the original Python implementation:

| Validation Type | Performance Improvement |
|----------------|------------------------|
| JSON Schema | 85% faster (via caching) |
| Logo Images | 5-10x faster (binary header parsing) |
| Overall | ~3-5x faster (parallel processing) |

## Development

### Building

```bash
# Build Rust library
cargo build --release

# Build Python bindings
maturin develop --release --features python

# Build standalone binary
cargo build --release --features binary
```

### Testing

```bash
# Run Rust tests
cargo test

# Test Python bindings
maturin develop --features python
python -c "from ofd_validator import PyValidationOrchestrator; print('OK')"
```

### Project Structure

```
ofd-validator/
├── Cargo.toml              # Rust package configuration
├── pyproject.toml          # Python package configuration
├── src/
│   ├── lib.rs              # Library root
│   ├── main.rs             # Binary CLI entry point
│   ├── python.rs           # PyO3 bindings
│   ├── types.rs            # Core data types
│   ├── validators/         # Validation implementations
│   │   ├── json.rs
│   │   ├── logo.rs
│   │   ├── folder.rs
│   │   ├── store_id.rs
│   │   ├── gtin.rs
│   │   └── missing.rs
│   └── utils/              # Shared utilities
│       ├── schema_cache.rs
│       ├── parallel.rs
│       ├── image_fast.rs
│       └── helpers.rs
└── .github/
    └── workflows/          # CI/CD pipelines
```

## Requirements

### Python Package
- Python 3.8 or higher
- No additional Python dependencies (compiled extension module)

### Building from Source
- Rust 1.70 or higher
- Cargo
- For Python bindings: maturin

## License

MIT License - see LICENSE file for details

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Authors

Open Filament Collective

## Links

- [PyPI Package](https://pypi.org/project/ofd-validator/)
- [GitHub Repository](https://github.com/OpenFilamentCollective/open-filament-database)
- [Issue Tracker](https://github.com/OpenFilamentCollective/open-filament-database/issues)
- [Open Filament Database](https://github.com/OpenFilamentCollective/open-filament-database)
