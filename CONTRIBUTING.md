# Contributing to ESM Checker

Thank you for your interest in contributing to ESM Checker! This document provides guidelines and instructions for contributing to the project.

## Development Setup

### Prerequisites

- Rust (latest stable version)
- Node.js (LTS version)
- yarn
- Git

### Getting Started

1. Fork the repository
2. Clone your fork:
   ```bash
   git clone https://github.com/YOUR_USERNAME/is-esm-ready-yet.git
   cd is-esm-ready-yet
   ```
3. Install Rust dependencies:
   ```bash
   cargo build
   ```
4. Install Node.js dependencies (if working on the npm package):
   ```bash
   cd crates/esm_checker_bindings
   yarn install
   ```

## Project Structure

The project is organized as a monorepo with several Rust crates:

- `crates/walk_imports`: Core functionality for walking and analyzing JavaScript/TypeScript imports
- `crates/es_resolver`: JavaScript module resolution utilities
- `crates/reporter`: Main library implementation.
- `crates/web_server`: Web interface for generating ESM compatibility reports
- `crates/fetch_and_report`: Utilities for fetching npm packages and generating reports. Used by the web server
- `crates/esm_checker_bindings`: Node-API bindings for the Rust implementation

## Development Workflow

1. Create a new branch for your feature or bugfix:
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. Make your changes, following the coding standards below

3. Run tests:
   ```bash
   cargo test
   ```

4. Update bindings code-gen:
   ```bash
   cd crates/esm_checker_bindings
   yarn napi build
   ```

5. If working on the Node.js bindings, also run:
   ```bash
   cd crates/esm_checker_bindings
   node --test
   ```

6. Commit your changes with a clear, descriptive commit message

7. Push to your fork and create a Pull Request


### Rust Code

- Follow the [Rust Style Guide](https://doc.rust-lang.org/1.0.0/style/style/naming/README.html)
- Run `cargo fmt` before committing
- Run `cargo clippy` to check for common issues
- Write tests for new functionality
- Document public APIs with rustdoc comments

## Debugging

Check out the [DEBUGGING.md](DEBUGGING.md) file for debugging tips.

## Pull Request Process

1. Update the README.md with details of changes if needed
2. Update the documentation if you're changing functionality
3. The PR will be merged once you have the sign-off of at least one maintainer
4. Make sure all tests pass before submitting

## Reporting Bugs

- Use the GitHub issue tracker
- Include steps to reproduce the bug
- Include expected and actual behavior
- Include relevant logs and error messages
- Include your environment details (OS, Node.js version, etc.)

## Feature Requests

- Use the GitHub issue tracker
- Clearly describe the feature
- Explain why this feature would be useful
- Include any relevant examples

## Questions and Discussion

Feel free to open an issue for any questions or discussions about the project.

## License

By contributing to ESM Checker, you agree that your contributions will be licensed under the project's MIT License.
