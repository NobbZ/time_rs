# Copilot Instructions

This document provides instructions for GitHub Copilot when working with this repository.

## Project Overview

This is a Rust workspace project called `time_rs` - a time tracking CLI application. The project uses Rust 2021 edition with a Cargo workspace structure. The binary is named `timers` (see `time_rs/Cargo.toml`).

## Build and Test

- Build the project: `cargo build`
- Run tests: `cargo nextest run` (preferred) or `cargo test`
- Run a specific test: `cargo nextest run <test_name>`
- Check code: `cargo check`
- Format code: `cargo fmt`
- Lint code: `cargo clippy`

## Code Style and Conventions

### Rust Conventions

- Follow standard Rust naming conventions (snake_case for functions/variables, PascalCase for types)
- Use Rust 2021 edition features
- Avoid `unwrap()` and `expect()` in non-test code - these are warned by clippy configuration
- Use `dbg!` macro only in test code
- Handle errors using `eyre` and `color-eyre` crates with proper context via `wrap_err()`

### Clippy Configuration

The project has strict clippy warnings enabled:
- `clippy::unwrap_used` - warns on unwrap usage (allowed in tests via `.clippy.toml`)
- `clippy::expect_used` - warns on expect usage (allowed in tests via `.clippy.toml`)
- `clippy::dbg_macro` - warns on dbg! macro usage (allowed in tests via `.clippy.toml`)

### Testing

- Use `rstest` for parameterized tests
- Use `rstest_reuse` for reusable test fixtures
- Prefer rstest fixtures and templates whenever possible and sensible for test setup and data sharing
- Use `assert_fs` for filesystem-related test assertions
- Test files should be placed alongside source files or in a `tests/` directory

## Licensing and REUSE Compliance

This project follows the [REUSE specification](https://reuse.software/) for license compliance:

- All source files must have SPDX license headers
- Rust files (`.rs`): `// SPDX-FileCopyrightText: <year> Norbert Melzer <timmelzer@gmail.com>` followed by `// SPDX-License-Identifier: MIT`
- Nix files (`.nix`): `# SPDX-FileCopyrightText: ...` followed by `# SPDX-License-Identifier: MIT`
- GitHub workflow files (`.yml` in `.github/workflows/`): Use `CC0-1.0` license
- Config files: Check existing similar files for the appropriate license
- Run `reuse lint` to check compliance

## Dependencies

When adding dependencies:
- Prefer well-maintained crates from crates.io
- Use `cargo audit` to check for security vulnerabilities
- Use `cargo deny` to verify license compatibility
- Update `Cargo.toml` in the appropriate workspace member

## Project Structure

```
time_rs/                    # Workspace root
├── Cargo.toml              # Workspace manifest
├── time_rs/                # Main crate
│   ├── Cargo.toml          # Crate manifest
│   └── src/
│       ├── main.rs         # CLI entry point
│       ├── lib.rs          # Library root
│       ├── cli/            # CLI parsing and commands
│       └── config/         # Configuration handling
└── .github/                # GitHub configuration
```

## Development Environment

This project uses Nix flakes for development environment management:
- Enter the dev environment: `nix develop`
- Format Nix files: `nix fmt`
- Pre-commit hooks are configured - run `pre-commit run --all-files` to check all hooks

## CI/CD and Workflows

GitHub Actions workflows are generated from CUE definitions in `internal/ci/`:
- Workflow definitions are in `internal/ci/*.cue` files
- Generate workflows: `make workflows`
- Validate workflows: `make check`
- **Important**: Do NOT edit workflow files in `.github/workflows/` directly - they are auto-generated
- To modify CI/CD, edit the CUE files and run `make workflows` to regenerate

## Pre-commit Hooks

The project uses pre-commit hooks that run automatically:
- `cargo fmt --check` - Ensures code is formatted
- `nix fmt -- --check` - Ensures Nix files are formatted
- `cargo audit` - Checks for security vulnerabilities
- `cargo deny check` - Verifies license compatibility
- `reuse` - Validates REUSE/SPDX compliance
- Standard checks: trailing whitespace, EOF, YAML/TOML validation, merge conflicts

Run manually: `pre-commit run --all-files`

## Common Tasks

### Starting Development
1. Clone the repository
2. Enter Nix shell: `nix develop` (or use direnv with `.envrc`)
3. Build the project: `cargo build`
4. Run tests: `cargo nextest run`

### Making Changes
1. Make your code changes
2. Format code: `cargo fmt`
3. Run linter: `cargo clippy`
4. Run tests: `cargo nextest run`
5. Check REUSE compliance: `reuse lint`
6. Run pre-commit hooks: `pre-commit run --all-files`

### Running the CLI
- Build and run: `cargo run -- <args>`
- Run installed binary: `timers <args>` (note: binary name is `timers`, not `time_rs`)

## Error Handling Patterns

The project uses `eyre` and `color-eyre` for error handling:
- Return `Result<T, eyre::Report>` for fallible operations
- Use `.wrap_err("context")` or `.wrap_err_with(|| "context")` to add context
- Chain contexts: `something().wrap_err("what failed").wrap_err("why it matters")`
- Initialize color-eyre in main: `color_eyre::install()?`
- Avoid panic: Use `Result` instead of `.unwrap()` or `.expect()` in production code

Example:
```rust
use eyre::{Result, WrapErr};

fn read_config() -> Result<Config> {
    let path = config_path()
        .wrap_err("Failed to determine config path")?;
    
    let contents = std::fs::read_to_string(&path)
        .wrap_err_with(|| format!("Failed to read config from {}", path.display()))?;
    
    serde_json::from_str(&contents)
        .wrap_err("Failed to parse config JSON")
}
```

## Anti-patterns to Avoid

### Do NOT:
- Use `.unwrap()` or `.expect()` in non-test code (clippy will warn)
- Use `dbg!()` macro in non-test code (clippy will warn)
- Edit files in `.github/workflows/` directly (they are generated from CUE)
- Modify generated files like `Cargo.nix` (maintained by other tools)
- Add dependencies without running `cargo audit` and `cargo deny check`
- Commit code without running `cargo fmt`
- Skip REUSE/SPDX license headers on new files
- Use bare `panic!()` - prefer returning errors

### DO:
- Add appropriate SPDX headers to all new files
- Use `wrap_err()` to provide context for errors
- Prefer `cargo nextest run` over `cargo test` for running tests
- Use rstest fixtures and templates for test setup
- Run `make workflows` after modifying CI/CD definitions
- Check `deny.toml` before adding dependencies with unusual licenses
- Use the provided Nix development environment for consistency
