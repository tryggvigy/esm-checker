# esm-checker

https://esm-checker.fly.dev/

Tools and libraries to help Node.js developers check for ECMAScript Modules (ESM) readiness. This project provides both a Rust-based core implementation and an npm package for easy integration into Node.js projects.

## Project Structure

This is a monorepo containing several Rust crates and an npm package:

### Rust Crates

- `crates/walk_imports`: Core functionality for walking and analyzing JavaScript/TypeScript imports
- `crates/is_esm_ready_yet`: Main library implementation
- `crates/es_resolver`: JavaScript module resolution utilities
- `crates/web_server`: Web interface for generating ESM compatibility reports
- `crates/fetch_and_report`: Utilities for fetching npm packages and generating reports
- `crates/napi_esm`: Node-API bindings for the Rust implementation

### NPM Package

The `npm-package` directory contains the Node.js wrapper that makes this tool available as an npm package.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

MIT
