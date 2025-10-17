# gl-sdk

Rust SDK for Greenlight with UniFFI bindings support.

## Building

Build the library using Task:

```bash
task sdk:build
```

Or directly with cargo from the workspace root:

```bash
cd /home/cdecker/dev/greenlight/202509-sdk/public
cargo build -p gl-sdk
```

## Generating Language Bindings

### Using Task (Recommended)

The easiest way to generate bindings is using the Task commands:

```bash
# Generate Python bindings
task sdk:bindings-python

# Generate Kotlin bindings
task sdk:bindings-kotlin

# Generate Swift bindings
task sdk:bindings-swift

# Generate Ruby bindings
task sdk:bindings-ruby

# Generate all language bindings
task sdk:bindings-all
```

These commands work from any directory in the workspace.

### Using uniffi-bindgen Directly

The project uses [UniFFI](https://mozilla.github.io/uniffi-rs/) to generate bindings for multiple languages from the UDL definition in `src/sdk.udl`.

The `uniffi-bindgen` tool is included in the workspace at `libs/uniffi-bindgen`.

**Note:** When using uniffi-bindgen directly, all commands must be run from the workspace root.

#### Generate Python Bindings

```bash
cd /home/cdecker/dev/greenlight/202509-sdk/public
cargo run --bin uniffi-bindgen -- generate \
  --library $CARGO_TARGET_DIR/debug/libglsdk.so \
  --language python \
  --out-dir ./libs/gl-sdk/bindings
```

#### Generate Bindings for Other Languages

Replace `--language python` with:
- `kotlin` for Kotlin
- `swift` for Swift
- `ruby` for Ruby

Example for Kotlin:

```bash
cargo run --bin uniffi-bindgen -- generate \
  --library $CARGO_TARGET_DIR/debug/libglsdk.so \
  --language kotlin \
  --out-dir ./libs/gl-sdk/bindings
```

## Files

- `src/sdk.udl` - UniFFI interface definition
- `build.rs` - Build script that generates Rust scaffolding
- `bindings/` - Generated language bindings (created by uniffi-bindgen)
