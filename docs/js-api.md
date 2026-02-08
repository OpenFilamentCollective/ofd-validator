# JavaScript / Node.js API Reference

## Installation

```bash
npm install ofd-validator
```

Prebuilt native binaries are included for:
- Linux x64 (glibc)
- Linux arm64 (glibc)
- macOS x64
- macOS arm64 (Apple Silicon)
- Windows x64

If no prebuilt binary is available for your platform, you'll need Rust installed to compile from source during `npm install`.

## Path Mode

Path mode mirrors the Python API — pass directory paths and the library reads files from disk.

### `validateAll(dataDir, storesDir, schemasDir?)`

Run all validations (JSON schemas, logos, folder names, store IDs, GTIN/EAN, required files).

```javascript
const { validateAll } = require('ofd-validator');

const result = validateAll('./data', './stores', './schemas');
console.log(result.isValid);      // boolean
console.log(result.errorCount);   // number
console.log(result.warningCount); // number
for (const err of result.errors) {
  console.log(`${err.level} [${err.category}]: ${err.message} (${err.path})`);
}
```

### Individual Path-Mode Validators

```javascript
const {
  validateJsonFiles,
  validateLogoFiles,
  validateFolderNames,
  validateStoreIds,
  validateGtinEan,
  validateRequiredFiles,
  validateLogoFile,
  validateFolderName,
} = require('ofd-validator');

// Batch validators
const jsonResult = validateJsonFiles('./data', './stores', './schemas');
const logoResult = validateLogoFiles('./data', './stores');
const folderResult = validateFolderNames('./data', './stores');
const storeResult = validateStoreIds('./data', './stores');
const gtinResult = validateGtinEan('./data');
const missingResult = validateRequiredFiles('./data', './stores');

// Single-item validators
const singleLogo = validateLogoFile('./data/BrandX/logo.png', 'logo.png');
const singleFolder = validateFolderName('./data/BrandX', 'brand.json', 'id');
```

## Content Mode

Content mode accepts file contents directly as strings or Buffers. No filesystem access occurs. This is useful for:
- Validating data fetched from an API or database
- CI pipelines where files are in memory (e.g. from git)
- Server-side validation of uploaded data

The optional `filePath` parameter in content-mode functions is **only used for error message labeling** — it does not read from disk.

### `validateJsonContent(content, schemaName, schemas, filePath?)`

Validate a JSON string against a named schema.

```javascript
const { validateJsonContent } = require('ofd-validator');
const fs = require('fs');

const schemas = {
  brand: fs.readFileSync('schemas/brand_schema.json', 'utf-8'),
  material: fs.readFileSync('schemas/material_schema.json', 'utf-8'),
};

const brandJson = '{"id": "BrandX", "name": "Brand X", "logo": "logo.png"}';
const result = validateJsonContent(brandJson, 'brand', schemas, 'data/BrandX/brand.json');
```

### `validateLogoContent(content, filename, logoName?, filePath?)`

Validate a logo from raw bytes.

```javascript
const { validateLogoContent } = require('ofd-validator');

const logoBytes = fs.readFileSync('data/BrandX/logo.png');
const result = validateLogoContent(logoBytes, 'logo.png', 'logo.png', 'data/BrandX/logo.png');
```

### `validateFolderNameContent(folderName, jsonContent, jsonKey, filePath?)`

Validate that a folder name matches a JSON field value.

```javascript
const { validateFolderNameContent } = require('ofd-validator');

const result = validateFolderNameContent(
  'BrandX',                              // actual folder name
  '{"id": "BrandX", "name": "Brand X"}', // JSON content
  'id',                                   // key to check
  'data/BrandX'                           // label for errors
);
```

### `validateGtinEanContent(sizesEntries)`

Validate GTIN/EAN fields from parsed sizes data.

```javascript
const { validateGtinEanContent } = require('ofd-validator');

const result = validateGtinEanContent([
  { path: 'data/BrandX/PLA/Red/Standard/sizes.json', content: '[{"size": "1kg", "gtin": "1234567890123"}]' },
]);
```

### `validateStoreIdsContent(storeIds, sizesEntries)`

Validate that store IDs referenced in sizes data exist.

```javascript
const { validateStoreIdsContent } = require('ofd-validator');

const result = validateStoreIdsContent(
  ['amazon', 'printables-store'],  // known valid store IDs
  [
    { path: 'sizes.json', content: '[{"purchase_links": [{"store_id": "amazon", "url": "..."}]}]' },
  ]
);
```

### `validateAllContent(data)`

Batch-validate everything from in-memory data.

```javascript
const { validateAllContent } = require('ofd-validator');

const result = validateAllContent({
  jsonFiles: [
    { path: 'data/BrandX/brand.json', schemaName: 'brand', content: '{"id":"BrandX","name":"Brand X","logo":"logo.png"}' },
    { path: 'data/BrandX/PLA/material.json', schemaName: 'material', content: '{"material":"PLA"}' },
  ],
  logoFiles: [
    { path: 'data/BrandX/logo.png', filename: 'logo.png', content: fs.readFileSync('logo.png') },
  ],
  folders: [
    { path: 'data/BrandX', folderName: 'BrandX', jsonContent: '{"id":"BrandX"}', jsonKey: 'id' },
  ],
  storeIds: ['amazon', 'printables-store'],
  schemas: {
    brand: fs.readFileSync('schemas/brand_schema.json', 'utf-8'),
    material: fs.readFileSync('schemas/material_schema.json', 'utf-8'),
  },
});
```

## Types

### `ValidationResult`

```typescript
interface ValidationResult {
  errors: ValidationError[];
  isValid: boolean;       // true if no errors (warnings are OK)
  errorCount: number;     // count of ERROR-level issues
  warningCount: number;   // count of WARNING-level issues
}
```

### `ValidationError`

```typescript
interface ValidationError {
  level: "ERROR" | "WARNING";
  category: string;       // e.g. "JSON", "Logo", "Folder", "GTIN", "EAN", "StoreID", "Missing File"
  message: string;        // human-readable description
  path: string | null;    // file/folder path where the issue was found
}
```
