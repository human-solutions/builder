# WARP.md

This file provides guidance to WARP (warp.dev) when working with code in this repository.

## Quickstart

Build and test the project:
```bash
# Build all crates
cargo build --workspace

# Build release binary 
cargo build --release -p builder

# Run all tests (requires external dependencies - see below)
cargo test --workspace

# Alternative: use nextest for better test output
cargo nextest run

# Check code without building
cargo check --workspace
```

Run the builder tool:
```bash
# Show version
./target/release/builder -V

# Run with configuration file
./target/release/builder path/to/builder.json

# Example using the examples crate
cd crates/examples
cargo run  # This builds and runs the example
```

## Architecture Overview

Builder is a Rust workspace containing a command-line tool for building web assets, WASM, and mobile libraries. The architecture is:

1. **Configuration Phase**: Rust build scripts use the `BuilderCmd` struct (from `builder-command` crate) with fluent builder pattern to configure build commands, then generate a `builder.json` file
2. **Execution Phase**: The `builder` CLI binary reads the JSON configuration and executes each build command in sequence

Key files:
- `crates/command/src/lib.rs` - Contains `BuilderCmd` fluent API and `Cmd` enum with all command types
- `crates/builder/src/main.rs` - CLI entry point that dispatches to command modules
- Individual command implementations in feature crates: `sass`, `wasm`, `uniffi`, `fontforge`, etc.

## Command Types

The `Cmd` enum in `crates/command/src/lib.rs` supports these build operations:

- **Sass** - SCSS compilation with dart-sass or built-in grass compiler
- **Wasm** - Rust to WebAssembly compilation with wasm-bindgen and optimization  
- **Uniffi** - Swift/Kotlin bindings generation from UniFFI .udl files
- **SwiftPackage** - Swift package creation
- **FontForge** - Font processing (SFD to WOFF2/OTF)
- **Assemble** - Asset scanning and Rust code generation
- **Localized** - Internationalized content handling
- **Copy** - Simple file copying with filtering

## JSON Configuration Format

Build scripts create configuration using the fluent API:

```rust
use builder_command::{BuilderCmd, SassCmd, WasmProcessingCmd, Output, Profile, DataProvider};

BuilderCmd::new()
    .add_sass(SassCmd::new("styles/main.scss")
        .add_output(Output::new("dist")
            .asset_code_gen("src/assets.rs", DataProvider::Embed)))
    .add_wasm(WasmProcessingCmd::new("my-wasm-package", Profile::Release)
        .add_output(Output::new("dist/wasm")))
    .run();  // Generates builder.json and executes
```

This generates a JSON configuration that the CLI tool processes.

## Workspace Structure

Key crates and their roles:

- **`builder`** - CLI binary entry point (`crates/builder/src/main.rs`)
- **`command`** - Command definitions and fluent API (`crates/command/src/lib.rs`)
- **`common`** - Shared utilities, logging, and file system operations
- **`assets`** - Asset management library for generated code
- Feature-specific crates:
  - **`sass`** - SCSS compilation logic
  - **`wasm`** - WebAssembly build pipeline with debug symbol handling
  - **`uniffi`** - UniFFI bindings with caching
  - **`fontforge`** - Font processing integration
  - **`assemble`** - Asset directory scanning and code generation
  - **`localized`** - Multi-language asset handling
  - **`copy`** - File copying operations
  - **`swift_package`** - Swift package generation
- **`examples`** - Working example showing multi-provider asset generation

## External Dependencies

For full testing, install these external tools:

```bash
# WASM target for Rust
rustup target add wasm32-unknown-unknown

# FontForge for font processing
# macOS:
brew install fontforge
# Linux:
sudo apt-get install fontforge

# Dart Sass for advanced SCSS features (optional - has fallback)
curl -L https://github.com/sass/dart-sass/releases/download/1.77.8/dart-sass-1.77.8-linux-x64.tar.gz | tar xz -C /usr/local/bin --strip-components=1 dart-sass
```

No database or external services are required - all dependencies are build tools.

## Testing

Different test categories:

```bash
# Unit tests only (no external deps needed)
cargo test --lib --workspace

# All tests including integration tests (requires external tools)
cargo test --workspace

# Using nextest for better output
cargo nextest run --workspace

# Test a specific command implementation
cargo test -p builder-sass

# Run examples to test end-to-end functionality
cd crates/examples && cargo run
```

## Asset Code Generation

Builder can generate Rust code for type-safe asset access:

**Two data providers:**
- `DataProvider::FileSystem` - Loads assets from disk at runtime
- `DataProvider::Embed` - Embeds assets in binary using rust-embed

**Usage in build scripts:**
```rust
.add_output(Output::new("dist")
    .asset_code_gen("src/assets.rs", DataProvider::Embed))
```

**Runtime configuration (FileSystem provider only):**
```rust
use builder_assets::set_asset_base_path;
set_asset_base_path("/path/to/assets");
```

See `crates/examples/` for a complete working example with both providers.

## WASM Debug Symbols

Four debug symbol modes for WASM builds:

```rust
WasmProcessingCmd::new("package", Profile::Release)
    .debug_symbols(DebugSymbolsMode::Strip)        // Remove (default)
    .debug_symbols(DebugSymbolsMode::Keep)         // Keep in main file  
    .debug_symbols(DebugSymbolsMode::WriteAdjacent) // Separate .debug.wasm
    .debug_symbols(DebugSymbolsMode::WriteTo("path")) // Custom path
```

## Release Process

This project uses `cargo-dist` for releases:

1. Update version in root `Cargo.toml` (workspace.package.version)
2. Create and push annotated tag:
   ```bash
   git tag v0.1.28 -m "Version 0.1.28: description"
   git push --tags
   ```
3. GitHub Actions automatically builds and publishes binaries
4. Install via: `cargo binstall builder`

## Key Implementation Notes

- Uses `camino-fs` for UTF-8 path handling throughout
- Error handling with `anyhow` 
- JSON serialization via `serde` for configuration files
- Workspace uses Rust 2024 edition
- All command modules implement caching based on content hashes
- Asset code generation supports content negotiation and compression

## Sources of Truth

- **README.md** - User-facing documentation and feature overview
- **CLAUDE.md** - Architecture details and development workflow  
- **Cargo.toml** - Workspace configuration and dependencies
- **.github/workflows/rust.yml** - CI setup and external tool requirements
- **crates/examples/** - Working end-to-end example