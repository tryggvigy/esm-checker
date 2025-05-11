# esm-checker

https://esm-checker.fly.dev/

Tools and libraries to help Node.js developers check for ECMAScript Modules (ESM) readiness. This project provides both a Rust-based core implementation and an npm package for easy integration into Node.js projects.

## Project Structure

This is a monorepo containing several Rust crates and an npm package:

### Rust Crates

- `crates/walk_imports`: Core functionality for walking and analyzing JavaScript/TypeScript imports
- `crates/es_resolver`: JavaScript module resolution utilities
- `crates/reporter`: Main library implementation.
- `crates/web_server`: Web interface for generating ESM compatibility reports
- `crates/fetch_and_report`: Utilities for fetching npm packages and generating reports
- `crates/esm_checker_bindings`: Node-API bindings for the Rust implementation

### `@esm-checker/checker` NPM Package

[Documentation](crates/esm_checker_bindings/README.md).

Inspects your project's dependencies and generates a report on their ESM compatibility.

### `crates/web_server`

Contains the code for the web server that powers https://esm-checker.fly.dev/

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

MIT
