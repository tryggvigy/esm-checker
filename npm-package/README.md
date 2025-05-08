# is-esm-ready-yet

A tool to check if your Node.js project and its dependencies are ESM ready.

## Installation

```bash
npm install -g is-esm-ready-yet
```

Or run directly with npx:

```bash
npx is-esm-ready-yet
```

## Usage

Run the tool in your project directory:

```bash
is-esm-ready-yet
```

This will analyze your package.json and node_modules directory to check:
- Which dependencies are ESM ready
- Which dependencies are still using CommonJS
- Which dependencies claim to be ESM but have CommonJS dependencies
- Which dependencies have missing .js file extensions
- Any resolve or parse errors encountered

## Options

- `-p, --package-json-location`: Path to package.json file (default: ./package.json)
- `-o, --outfile`: Output JSON file to write results to
- `-c, --check`: Comma-separated list of specific dependencies to check

## Example

```bash
is-esm-ready-yet -p ./package.json -o report.json
```

## License

MIT