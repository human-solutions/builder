# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Architecture

This is a Rust workspace containing a command-line tool for building web assets, WASM, and mobile libraries. The project is structured as follows:

- **Builder crate**: `crates/builder/` - Both library (`lib.rs`) and binary (`main.rs`)
  - **Library**: Exports `builder::execute(BuilderCmd)` for direct in-process execution
  - **Binary**: CLI wrapper that reads YAML config files and calls the library
- **Command library**: `crates/command/` - Contains command type definitions and `BuilderCmd` struct
- **Feature crates**: Individual crates for each build command type:
  - `sass/` - SASS/SCSS compilation
  - `localized/` - Localized asset handling
  - `fontforge/` - FontForge integration
  - `uniffi/` - Uniffi bindings generation
  - `wasm/` - WASM compilation and optimization
  - `copy/` - File copying operations
  - `swift_package/` - Swift package generation
- **Common utilities**: `crates/common/` - Shared utilities including file system operations and logging
- **Runtime library**: `crates/assets/` - Runtime support for generated asset code with content negotiation
- **Examples**: `crates/examples/` - Working example demonstrating multi-provider asset generation

The tool works by:
1. Reading a YAML configuration file (builder.yaml format)
2. Parsing it into a `BuilderCmd` structure containing multiple command types using serde
3. Executing each command in sequence via `builder::execute()` (library) or the CLI binary

## Development Commands

### Building and Testing
```bash
# Build the project (examples included in workspace)
cargo build

# Run all tests
cargo test

# Or use nextest for better output
cargo nextest run

# Run specific test suites
cargo test -p common          # Common utilities
cargo test -p localized       # Localization
cargo test -p builder         # Integration tests (CLI + library)

# Check code without building
cargo check

# Build examples (real-world usage)
cd crates/examples && cargo build
```

**Note**: The `builder` crate provides both a library and binary. Examples use `builder::execute()` directly (no subprocess spawning), eliminating cargo locking issues.

### External Dependencies Required for Testing
- **FontForge**:
  - Linux: `sudo apt-get install fontforge`
  - macOS: `brew install fontforge`
- **Sass**: Download dart-sass from GitHub releases (optional - has grass fallback)
- **WASM target**: `rustup target add wasm32-unknown-unknown`
- **nextest** (optional): `cargo install cargo-nextest` for better test output

### Running the Tool
The builder binary expects a YAML configuration file as its first argument:
```bash
./target/debug/builder path/to/builder.yaml
```

### Release Process
1. Update version in root `Cargo.toml` (workspace.package.version)
2. Create and push annotated git tag: `git tag v0.1.X -m "Version 0.1.X: description"`
3. Push tag: `git push --tags`
4. CI automatically builds and releases via cargo-dist
5. Users can install via: `cargo binstall builder`

Current version: 0.1.28

## Key Design Patterns

- **Command Pattern**: Each build operation is implemented as a separate command struct with its own module
- **Configuration-driven**: The tool is entirely driven by YAML configuration files
- **Workspace architecture**: Modular design with separate crates for different responsibilities
- **Error handling**: Uses `anyhow` for error handling throughout
- **File system abstraction**: Uses `camino-fs` for UTF-8 path handling

## Command Types

The `Cmd` enum supports these build operations:

- **Sass** - SCSS compilation with dart-sass or built-in grass compiler
- **Wasm** - Rust to WebAssembly compilation with wasm-bindgen and optimization
- **Uniffi** - Swift/Kotlin bindings generation from UniFFI .udl files
- **SwiftPackage** - Swift package creation
- **FontForge** - Font processing (SFD to WOFF2/OTF)
- **Localized** - Internationalized content handling
- **Copy** - File copying with filtering and asset code generation support

## Working with Command Modules

When adding new commands or modifying existing ones:
1. Each command type definition lives in `crates/command/src/`
2. Commands must implement serialization via serde
3. Add the command variant to the `Cmd` enum in `crates/command/src/lib.rs`
4. Create the implementation crate in `crates/` with a public `run()` function
5. Add the crate dependency to `crates/builder/Cargo.toml`
6. Update the match statement in `crates/builder/src/lib.rs` `run_commands()` function

The builder uses YAML serialization via serde for configuration files, providing human-readable and standard format handling with automatic field serialization.

## Testing

- **Unit tests**: In `crates/common/src/site_fs/tests/` and `crates/localized/src/tests/`
- **Integration tests**: In `crates/builder/tests/cli_integration.rs` - tests both CLI and library execution
- **Examples**: `crates/examples/` provides real-world usage in build.rs

The architecture eliminates nested cargo calls by using direct library execution (`builder::execute()`) instead of spawning the binary.

## Asset Code Generation

Builder can generate Rust code for type-safe asset access with two data providers:

- **DataProvider::FileSystem** - Loads assets from disk at runtime
- **DataProvider::Embed** - Embeds assets in binary using rust-embed

Usage in build.rs:
```rust
use builder::builder_command::{BuilderCmd, CopyCmd, DataProvider, Output};

let cmd = BuilderCmd::new()
    .add_copy(CopyCmd::new("assets")
        .recursive(true)
        .file_extensions(["css", "js", "png"])
        .add_output(Output::new("dist")
            .asset_code_gen("src/assets.rs", DataProvider::Embed)));

builder::execute(cmd);  // Direct in-process execution
```

Runtime configuration (FileSystem provider only):
```rust
use builder_assets::set_asset_base_path;
set_asset_base_path("/path/to/assets");
```

The generated code uses the `builder-assets` crate for runtime support. See `crates/examples/build.rs` for a complete working example with both providers.

## WASM Debug Symbols

Four debug symbol modes for WASM builds:

```rust
WasmProcessingCmd::new("package", Profile::Release)
    .debug_symbols(DebugSymbolsMode::Strip)        // Remove (default)
    .debug_symbols(DebugSymbolsMode::Keep)         // Keep in main file
    .debug_symbols(DebugSymbolsMode::WriteAdjacent) // Separate .debug.wasm
    .debug_symbols(DebugSymbolsMode::WriteTo("path")) // Custom path
```

## Key Implementation Notes

- Uses `camino-fs` for UTF-8 path handling throughout
- Error handling with `anyhow`
- YAML serialization via `serde` for configuration files
- Workspace uses Rust 2024 edition
- All command modules implement caching based on content hashes
- Asset code generation supports content negotiation and compression
