# OFD Validator

High-performance validation library for the [Open Filament Database](https://github.com/OpenFilamentCollective/open-filament-database), written in Rust with Python bindings via [PyO3](https://pyo3.rs/).

## Features

- **JSON Schema Validation** &mdash; validates filament data against JSON schemas with compiled schema caching
- **Logo Validation** &mdash; checks image dimensions (PNG, JPEG) and validates SVG root elements
- **Folder Name Validation** &mdash; ensures folder names match their JSON content IDs
- **Store ID Validation** &mdash; cross-references store IDs in purchase links
- **GTIN/EAN Validation** &mdash; validates product barcodes
- **Missing File Detection** &mdash; checks for required files at each hierarchy level
- **Parallel Processing** &mdash; multi-threaded validation using [Rayon](https://github.com/rayon-rs/rayon)

## Installation

```bash
pip install ofd-validator
```

Requires Python 3.10+. No additional Python dependencies needed (self-contained compiled extension).

## Usage

### Functions

The library exposes standalone functions rather than a class. All functions accept directory paths as strings and return a `ValidationResult` object.

#### Batch validators (internally parallel)

```python
from ofd_validator import validate_all, validate_json_files, validate_logo_files, validate_folder_names

# Run all validations at once
result = validate_all("data", "stores")

# Or run specific batches
result = validate_json_files("data", "stores")
result = validate_logo_files("data", "stores")
result = validate_folder_names("data", "stores")

# JSON schemas can be in a custom directory
result = validate_all("data", "stores", schemas_dir="schemas")
result = validate_json_files("data", "stores", schemas_dir="schemas")
```

#### Individual validators

```python
from ofd_validator import validate_store_ids, validate_gtin_ean, validate_required_files
from ofd_validator import validate_logo_file, validate_folder_name

result = validate_store_ids("data", "stores")
result = validate_gtin_ean("data")
result = validate_required_files("data", "stores")

# Single-item validators
result = validate_logo_file("data/BrandX/logo.png", logo_name="logo.png")
result = validate_folder_name("data/BrandX", "brand.json", "id")
```

### Result objects

```python
result = validate_all("data", "stores")

result.is_valid      # True if no errors (warnings are OK)
result.error_count   # Number of errors
result.warning_count # Number of warnings
result.errors        # List of ValidationError objects

# Merge results from multiple runs
combined = ValidationResult()
combined.merge(result_a)
combined.merge(result_b)

# Serialize to dict (for JSON output)
d = result.to_dict()
# {"is_valid": True, "error_count": 0, "warning_count": 0, "errors": [...]}
```

### Error objects

```python
for error in result.errors:
    error.level     # ValidationLevel.Error or ValidationLevel.Warning
    error.level.value  # "ERROR" or "WARNING"
    error.category  # e.g. "JSON Schema", "Logo", "Folder Name"
    error.message   # Human-readable description
    error.path      # Optional file path (str or None)
```

## Development

### Prerequisites

- Rust (stable)
- Python 3.10+
- [maturin](https://github.com/PyO3/maturin) (`pip install maturin`)

### Building

```bash
# Build and install into current Python env (debug)
maturin develop

# Build optimized release wheel
maturin develop --release

# Build wheel for distribution
maturin build --release
```

### Testing

```bash
cargo test
```

### Project structure

```
ofd-validator/
├── Cargo.toml                        # Rust package config
├── pyproject.toml                    # Python package config (maturin)
└── src/
    ├── lib.rs                        # PyO3 module definition
    ├── types.rs                      # ValidationLevel, ValidationError, ValidationResult
    ├── schema_cache.rs               # Compiled JSON schema cache
    ├── util.rs                       # Shared helpers (logging, constants, JSON loading)
    ├── orchestrator.rs               # Batch validators with parallel task collection
    └── validators/
        ├── mod.rs                    # Individual pyfunction wrappers
        ├── json_validator.rs         # JSON schema validation
        ├── logo_validator.rs         # Logo file validation (PNG, JPEG, SVG)
        ├── folder_name.rs            # Folder name vs JSON content check
        ├── store_id.rs               # Store ID cross-reference check
        ├── gtin.rs                   # GTIN/EAN barcode validation
        └── missing_files.rs          # Required file existence check
```

## License

MIT

## Links

- [PyPI](https://pypi.org/project/ofd-validator/)
- [Repository](https://github.com/OpenFilamentCollective/ofd-validator)
- [Open Filament Database](https://github.com/OpenFilamentCollective/open-filament-database)
