# @esm-checker/checker

A tool to check if your dependencies are ESM ready. This package provides both a library and CLI interface to analyze your project's dependencies and determine their ESM compatibility.

## Installation

```bash
# Install as a dependency
npm install @esm-checker/checker

# Or use npx to run without installation
npx @esm-checker/checker --package-json-location ./package.json
```

## Usage

### As a Library

```javascript
import { generateReport } from '@esm-checker/checker';

// Generate a report for all dependencies
const report = generateReport('./package.json');

// Or check specific dependencies
const report = generateReport('./package.json', ['react', 'lodash']);

// The report contains:
// - total: Total number of dependencies checked
// - esm: Array of ESM-compatible dependencies
// - cjs: Array of CommonJS dependencies
// - fauxEsm: Object containing:
//   - withCommonjsDependencies: Array of faux ESM packages with CommonJS transitive dependencies
//   - withMissingJsFileExtensions: Array of faux ESM packages with missing file extensions in relative imports
// - resolveErrors: Array of dependencies that couldn't be resolved
// - parseErrors: Array of dependencies that couldn't be parsed
```

### As a CLI

```bash
# Check all dependencies in package.json
npx @esm-checker/checker --package-json-location ./package.json

# Check specific dependencies
npx @esm-checker/checker --package-json-location ./package.json --check react,lodash

# Save report to a file
npx @esm-checker/checker --package-json-location ./package.json --outfile ./esm-report.json
```

#### CLI Options

- `-p, --package-json-location <PACKAGE_JSON_LOCATION>`: Path to package.json file to check (required)
- `-o, --outfile <OUTFILE>`: Output .json file to write results to (absolute path)
- `-c, --check <CHECK>`: Comma-separated list of dependencies to check (checks all if omitted)
- `-h, --help`: Print help
- `-V, --version`: Print version

## Requirements

- Node.js >= 10
- Supported platforms:
  - Windows (x64)
  - macOS (x64, arm64)
  - Linux (x64-gnu, x64-musl)

## License

MIT
