# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Architecture

This is a Rust workspace containing a command-line tool for building web assets, WASM, and mobile libraries. The project is structured as follows:

- **Main binary**: `crates/builder/` - CLI entry point that reads a configuration file and dispatches to command modules
- **Command library**: `crates/command/` - Contains all command implementations and the main `BuilderCmd` struct
- **Feature crates**: Individual crates for each build command type:
  - `assemble/` - Asset assembly and inclusion
  - `sass/` - SASS/SCSS compilation
  - `localized/` - Localized asset handling
  - `fontforge/` - FontForge integration
  - `uniffi/` - Uniffi bindings generation
  - `wasm/` - WASM compilation and optimization
  - `copy/` - File copying operations
  - `swift_package/` - Swift package generation
- **Common utilities**: `crates/common/` - Shared utilities including file system operations and logging
- **Runtime library**: `crates/assets/` - Runtime support for generated asset code with content negotiation
- **Examples**: `crates/examples/` - Working example demonstrating multi-provider asset generation (excluded from workspace, requires builder binary)

The tool works by:
1. Reading a YAML configuration file (builder.yaml format)
2. Parsing it into a `BuilderCmd` structure containing multiple command types using serde
3. Executing each command in sequence through their respective modules

## Development Commands

### Building and Testing
```bash
# Clean build workflow (build builder binary first, then everything else)
# This is required because build.rs files in the workspace use the builder binary
cargo build -p builder && cargo build

# Or for tests
cargo build -p builder && cargo nextest run

# Build the project
cargo build

# Run tests (requires external dependencies)
cargo nextest run

# Check code without building
cargo check

# Build specific crate
cargo build -p builder

# Run the examples crate (excluded from workspace, requires builder binary)
cd crates/examples && cargo run
```

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
- **Assemble** - Asset scanning and Rust code generation
- **Localized** - Internationalized content handling
- **Copy** - Simple file copying with filtering

## Working with Command Modules

When adding new commands or modifying existing ones:
1. Each command has its own module in `crates/command/src/`
2. Commands must implement `Display` and `FromStr` for serialization
3. Add the command variant to the `Cmd` enum in `lib.rs`
4. Update the match statements in both the enum implementation and main dispatcher
5. Create a corresponding crate in `crates/` for the actual implementation

The builder uses YAML serialization via serde for configuration files, providing human-readable and standard format handling with automatic field serialization.

## Asset Code Generation

Builder can generate Rust code for type-safe asset access with two data providers:

- **DataProvider::FileSystem** - Loads assets from disk at runtime
- **DataProvider::Embed** - Embeds assets in binary using rust-embed

Usage in build.rs:
```rust
use builder_command::{BuilderCmd, CopyCmd, DataProvider, Output};

BuilderCmd::new()
    .add_copy(CopyCmd::new("assets")
        .recursive(true)
        .file_extensions(["css", "js", "png"])
        .add_output(Output::new("dist")
            .asset_code_gen("src/assets.rs", DataProvider::Embed)))
    .exec("path/to/builder");  // Execute using builder binary
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
