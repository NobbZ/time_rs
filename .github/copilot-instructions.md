<!--
SPDX-FileCopyrightText: 2025 Norbert Melzer <timmelzer@gmail.com>

SPDX-License-Identifier: CC0-1.0
-->

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
