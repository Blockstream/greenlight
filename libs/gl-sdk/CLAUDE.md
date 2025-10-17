# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`gl-sdk` is a Rust library that provides UniFFI-based language bindings for the Greenlight Lightning Network client. It wraps the core `gl-client` library and generates bindings for Python, Kotlin, Swift, and Ruby using Mozilla's UniFFI framework.

**Key Architecture:**
- Core Rust library (`src/lib.rs`) wraps `gl-client` types with UniFFI-compatible interfaces
- UniFFI generates foreign language bindings from the Rust implementation
- Main types: `Credentials`, `Node`, and `Signer` (all currently under development)
- Dependencies: Built on top of `gl-client` (v0.3.1) which handles the actual Greenlight protocol

## Building and Testing

**Build the library:**
```bash
# From workspace root or any directory
task sdk:build

# Or directly with cargo from workspace root
cargo build -p gl-sdk
```

**Build for release:**
```bash
task sdk:build-release
```

**Run tests:**
```bash
# From workspace root
cargo test -p gl-sdk

# Run specific test
cargo test -p gl-sdk <test_name>
```

Note: Currently there are no tests in the `tests/` directory.

## Generating Language Bindings

UniFFI bindings must be generated after building the library. The build must complete first to produce the shared library that uniffi-bindgen processes.

**Generate all bindings:**
```bash
task sdk:bindings-all
```

**Generate specific language:**
```bash
task sdk:bindings-python   # Python
task sdk:bindings-kotlin   # Kotlin
task sdk:bindings-swift    # Swift
task sdk:bindings-ruby     # Ruby
```

**Clean generated bindings:**
```bash
task sdk:clean
```

All bindings are generated into `bindings/` directory.

## Important Notes

- **Workspace structure:** This is a workspace member. The workspace root is at `/home/cdecker/dev/greenlight/202509-sdk/public/`
- **Task commands:** All `task sdk:*` commands work from any directory in the workspace
- **UniFFI workflow:** Build library first, then generate bindings. UniFFI reads the compiled library to generate foreign code.
- **Working directory:** When using cargo directly, commands should be run from the workspace root with `-p gl-sdk`
- **Current state:** Most functionality is unimplemented (marked with `unimplemented!()` or `todo!()`). The library is in early development.

## Related Libraries

- `gl-client`: Core Greenlight client library that this SDK wraps
- `uniffi-bindgen`: Custom workspace binary for generating language bindings
- `gl-testing`: Testing utilities (available as dev dependency via uv)

## Development with Python

Python bindings and testing use `uv` for dependency management:
- Use `uv run python3` instead of `python` or `python3`
- Python version: 3.10+ (specified in `.python-version`)
- Dev dependency: `gl-testing` for integration tests
