# Migration Guide — ofd-validator v0.3.0

## Python API: No Breaking Changes

The Python API is **fully backwards-compatible**. The module name (`ofd_validator`), all functions, all types, and all behavior are identical. No code changes are needed in Python dependents.

```python
# This still works exactly as before
import ofd_validator

result = ofd_validator.validate_all("data", "stores", "schemas")
print(result.is_valid)
print(result.error_count)
```

All functions remain available:

| Function | Signature | Changed? |
|----------|-----------|----------|
| `validate_all` | `(data_dir, stores_dir, schemas_dir=None)` | No |
| `validate_json_files` | `(data_dir, stores_dir, schemas_dir=None)` | No |
| `validate_logo_files` | `(data_dir, stores_dir)` | No |
| `validate_folder_names` | `(data_dir, stores_dir)` | No |
| `validate_store_ids` | `(data_dir, stores_dir)` | No |
| `validate_gtin_ean` | `(data_dir)` | No |
| `validate_required_files` | `(data_dir, stores_dir)` | No |
| `validate_logo_file` | `(logo_path, logo_name=None)` | No |
| `validate_folder_name` | `(folder_path, json_file, json_key)` | No |

Types `ValidationResult`, `ValidationError`, and `ValidationLevel` are unchanged.

## Internal Restructuring

The project has been restructured from a single Rust crate into a Cargo workspace with three crates:

```
crates/
  ofd-validator-core/     # Pure validation logic (no FFI)
  ofd-validator-python/   # PyO3 bindings (what you install via pip)
  ofd-validator-js/       # napi-rs bindings (what you install via npm)
```

### Building from source

If you were building from source with `maturin`, the `pyproject.toml` now includes a `manifest-path` that points to the Python crate. `maturin develop` and `maturin build` continue to work from the repository root:

```bash
maturin develop --release   # still works
maturin build --release     # still works
```

## New: JavaScript / Node.js Support

Starting with this version, the validation library is also available as an npm package:

```bash
npm install ofd-validator
```

The JS binding offers the same path-mode API as Python, plus a **content mode** that accepts file contents directly (strings and Buffers) without reading from disk. See [docs/js-api.md](docs/js-api.md) for full documentation.
