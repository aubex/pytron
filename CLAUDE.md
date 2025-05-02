# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build/Test Commands
- Build: `cargo build` (debug) or `cargo build --release` (optimized)
- Run: `cargo run -- <commands>` (e.g., `cargo run -- zip` or `cargo run -- run robot.zip main.py`)
- Test: `cargo test` (run all tests)
- Test single: `cargo test <test_name>` (e.g., `cargo test tests::test_zip_directory`)
- Test with output: `cargo test <test_name> -- --nocapture` (shows test output)
- Lint: `cargo clippy`
- Format: `cargo fmt`

## Code Style Guidelines
- **Imports**: Group standard library, then external crates, then internal modules
- **Formatting**: Follow rustfmt conventions (4-space indentation)
- **Types**: Use strong typing; prefer Result<T, E> for error handling
- **Naming**: snake_case for variables/functions, CamelCase for types/enums/traits
- **Error Handling**: Use Result with `?` operator; provide context in error messages
- **Comments**: Document public API with /// comments; explain complex logic
- **Python**: For embedded Python, follow PEP 8 with 4-space indentation

## Project Structure
- Rust CLI tool that packages Python scripts into zip files and runs them
- Uses uv for Python dependency management and script execution