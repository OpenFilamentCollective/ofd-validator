# OFD Validator

High-performance validation library for the [Open Filament Database](https://github.com/OpenFilamentCollective/open-filament-database), written in Rust with Python and Node.js bindings.

## Features

- **JSON Schema Validation** &mdash; validates filament data against JSON schemas with compiled schema caching
- **Logo Validation** &mdash; checks image dimensions (PNG, JPEG) and validates SVG root elements
- **Folder Name Validation** &mdash; ensures folder names match their JSON content IDs
- **Store ID Validation** &mdash; cross-references store IDs in purchase links
- **GTIN/EAN Validation** &mdash; validates product barcodes
- **Missing File Detection** &mdash; checks for required files at each hierarchy level
- **Parallel Processing** &mdash; multi-threaded validation using [Rayon](https://github.com/rayon-rs/rayon)

## Installation

### Python

```bash
pip install ofd-validator
```

Requires Python 3.10+. No additional Python dependencies needed (self-contained compiled extension).

### Node.js

Published to [GitHub Packages](https://github.com/OpenFilamentCollective/ofd-validator/packages). Add a `.npmrc` in your project root:

```
@openfilamentcollective:registry=https://npm.pkg.github.com
```

Then install:

```bash
npm install @openfilamentcollective/ofd-validator
```

Prebuilt binaries included for Linux (x64, arm64), macOS (x64, arm64), and Windows (x64).

## Python Usage

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

## Node.js Usage

### Path mode (filesystem-based)

```javascript
const { validateAll, validateJsonFiles } = require('@openfilamentcollective/ofd-validator');

const result = validateAll('./data', './stores', './schemas');
console.log(result.isValid);      // boolean
console.log(result.errorCount);   // number
console.log(result.warningCount); // number
for (const err of result.errors) {
  console.log(`${err.level} [${err.category}]: ${err.message} (${err.path})`);
}
```

### Content mode (in-memory, no filesystem access)

Pass file contents directly as strings or Buffers. Useful for CI pipelines, API-fetched data, or server-side validation.

```javascript
const { validateJsonContent, validateAllContent } = require('@openfilamentcollective/ofd-validator');
const fs = require('fs');

// Single JSON validation
const schemas = {
  brand: fs.readFileSync('schemas/brand_schema.json', 'utf-8'),
};
const result = validateJsonContent(
  '{"id": "BrandX", "name": "Brand X", "logo": "logo.png"}',
  'brand',
  schemas,
  'data/BrandX/brand.json'  // optional label for error messages
);

// Batch validation from in-memory data
const fullResult = validateAllContent({
  jsonFiles: [
    { path: 'brand.json', schemaName: 'brand', content: '{"id":"BrandX"}' },
  ],
  logoFiles: [
    { path: 'logo.png', filename: 'logo.png', content: fs.readFileSync('logo.png') },
  ],
  folders: [
    { path: 'data/BrandX', folderName: 'BrandX', jsonContent: '{"id":"BrandX"}', jsonKey: 'id' },
  ],
  storeIds: ['amazon'],
  schemas: schemas,
});
```

For the full JS API reference, see [docs/js-api.md](docs/js-api.md).

## Development

### Prerequisites

- Rust (stable)
- Python 3.10+ and [maturin](https://github.com/PyO3/maturin) (for Python bindings)
- Node.js 18+ (for JS bindings)

### Building

```bash
# Python: build and install into current env
maturin develop --release

# JS: build native addon
cd crates/ofd-validator-js && npm install && npm run build

# Check all crates
cargo check --workspace
```

### Testing

```bash
cargo test --workspace
```

### Project structure

```
ofd-validator/
├── Cargo.toml                            # Workspace definition
├── pyproject.toml                        # Python package config (maturin)
├── crates/
│   ├── ofd-validator-core/               # Pure Rust validation library (no FFI)
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── types.rs                  # ValidationLevel, ValidationError, ValidationResult
│   │       ├── schema_cache.rs           # Compiled JSON schema cache
│   │       ├── util.rs                   # Constants, helpers
│   │       ├── orchestrator.rs           # DataSet + parallel batch validation
│   │       └── validators/               # Individual validator implementations
│   ├── ofd-validator-python/             # PyO3 bindings
│   │   └── src/
│   │       ├── lib.rs                    # #[pymodule] definition
│   │       ├── types.rs                  # PyO3 wrapper types
│   │       ├── orchestrator.rs           # Python batch validators
│   │       └── validators.rs             # Python individual validators
│   └── ofd-validator-js/                 # napi-rs bindings
│       ├── src/lib.rs                    # #[napi] exports (path mode + content mode)
│       ├── package.json                  # npm package config
│       └── build.rs                      # napi-build
├── docs/
│   └── js-api.md                         # Full JS API reference
└── MIGRATION.md                          # Migration guide for dependents
```

## License

MIT

## Links

- [PyPI](https://pypi.org/project/ofd-validator/)
- [npm (GitHub Packages)](https://github.com/OpenFilamentCollective/ofd-validator/packages)
- [Repository](https://github.com/OpenFilamentCollective/ofd-validator)
- [Open Filament Database](https://github.com/OpenFilamentCollective/open-filament-database)
