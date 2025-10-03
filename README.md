# Builder

A command-line tool for building web assets, WASM, and mobile libraries. Builder simplifies the build process by reading a configuration file and executing multiple build commands in sequence.

## Overview

Builder is both a library and a CLI tool. The `builder` crate provides:
- **Library API** (`builder::execute()`): Direct in-process execution for use in build.rs scripts
- **CLI Binary**: Standalone tool that reads YAML configuration files

This dual approach eliminates nested cargo calls and locking issues while providing flexibility for both programmatic and standalone usage.

## Features

- **SASS/SCSS Compilation** - Compiles SCSS files using dart-sass (if available) or built-in grass compiler. Supports CSS optimization with LightningCSS, string replacements, and outputs with browser compatibility targets.

- **WASM Building** - Compiles Rust packages to WebAssembly for web targets. Runs `cargo build --target wasm32-unknown-unknown`, generates JS bindings with wasm-bindgen, optimizes with wasm-opt in release mode, and includes smart caching to skip unchanged builds.

- **Uniffi Bindings** - Generates Swift and Kotlin language bindings from UniFFI definition files (.udl). Features intelligent caching that compares UDL files, config files, and CLI parameters to avoid regeneration. Automatically fixes Swift modulemap files for framework usage.

- **Swift Package Generation** - Creates Swift packages using the swift-package crate. Configures build settings based on release/debug mode and respects global verbose settings.

- **FontForge Integration** - Processes SFD (Spline Font Database) files using FontForge to generate WOFF2 and OTF formats. Includes content-based caching via seahash, and on macOS automatically installs OTF fonts to `~/Library/Fonts/`.

- **Asset Assembly** - Scans asset directories and generates Rust code for asset management using the `builder-assets` crate. Creates static AssetSet variables with content negotiation support. Supports both embedded assets (using rust-embed) and filesystem-based loading. Generates formatted Rust code with comprehensive metadata preservation.

- **Localized Assets** - Handles internationalized content by scanning directories for language-specific files (e.g., `en.css`, `fr.css`). Parses ICU language identifiers and organizes content by locale for multi-language applications.

- **File Copying** - Simple file copying with extension filtering and optional recursive directory traversal. Integrates with the site filesystem for consistent output handling.

## Installation

### From Releases

Download pre-compiled binaries for your platform from [GitHub Releases](https://github.com/human-solutions/builder/releases).

You can install using `cargo binstall`:

```bash
cargo binstall builder
```

### From Source

```bash
git clone https://github.com/human-solutions/builder
cd builder
cargo build --release
```

## Usage

### Programmatic Usage (Build Scripts)

Add builder as a build dependency and use it in your `build.rs`:

```toml
[build-dependencies]
builder = "0.1"
```

```rust
use builder::builder_command::{BuilderCmd, DataProvider, DebugSymbolsMode, LogLevel, Output, Profile, SassCmd, WasmProcessingCmd};

fn main() {
    let cmd = BuilderCmd::new()
        .add_sass(SassCmd::new("styles/main.scss")
            .add_output(Output::new("dist")
                .asset_code_gen("src/assets.rs", DataProvider::Embed))) // Generate embedded asset code
        .add_wasm(
            WasmProcessingCmd::new("my-wasm-package", Profile::Release)
                // Four debug symbol options:
                .debug_symbols(DebugSymbolsMode::Strip)        // Strip debug symbols (default)
                // .debug_symbols(DebugSymbolsMode::Keep)       // Keep debug symbols in main WASM
                // .debug_symbols(DebugSymbolsMode::WriteAdjacent) // Write .debug.wasm next to main file
                // .debug_symbols(DebugSymbolsMode::write_to("debug/symbols.debug.wasm")) // Custom path
                .add_output(Output::new("dist/wasm")
                    .asset_code_gen("src/wasm_assets.rs", DataProvider::FileSystem)) // Generate filesystem asset code
        )
        .log_level(LogLevel::Verbose);

    builder::execute(cmd);  // Direct in-process execution
}
```

### CLI Usage

Builder can also be used directly with a YAML configuration file:

```bash
builder path/to/builder.yaml
```

The YAML configuration file defines which build commands to execute and their parameters. Each command type has its own configuration options and will be executed in the order specified.

### Asset Code Generation

Builder can automatically generate Rust code for asset management using the `builder-assets` crate. This provides type-safe access to assets with content negotiation support:

```rust
// Generated assets.rs
use builder_assets::*;
use icu_locid::langid;

pub static STYLE_CSS: AssetSet = AssetSet {
    url_path: "/style.css",
    // ... asset configuration
};

pub fn get_asset_catalog() -> AssetCatalog {
    AssetCatalog::from_assets(&ASSETS)
}
```

**Two Data Providers:**
- **`DataProvider::FileSystem`** - Loads assets from disk at runtime (requires runtime path configuration)
- **`DataProvider::Embed`** - Embeds assets in binary using rust-embed (no runtime setup needed)

**Configuration:**
```rust
// Code generation configuration
.add_output(Output::new("dist")
    .asset_code_gen("src/assets.rs", DataProvider::FileSystem))

// Runtime configuration (required for FileSystem provider)
use builder_assets::set_asset_base_path;

fn main() {
    // Set asset path for your deployment scenario
    set_asset_base_path("/opt/myapp/assets");  // Production
    // set_asset_base_path("./assets");        // Development
    // set_asset_base_path(exe_dir.join("assets")); // Relative to binary

    // ... rest of application
}
```

### WASM Debug Symbols

Builder provides four options for handling debug symbols in WASM builds:

1. **Strip** (default) - Removes debug symbols for smallest file size
2. **Keep** - Preserves debug symbols in the main WASM file
3. **WriteAdjacent** - Splits debug symbols into a separate `.debug.wasm` file next to the main file
4. **WriteTo(path)** - Splits debug symbols into a separate file at a custom path

Configuration examples:
```toml
# Strip debug symbols (default)
debug_symbols=strip

# Keep debug symbols in main file
debug_symbols=keep

# Adjacent debug file
debug_symbols=adjacent

# Custom debug path
debug_symbols=write_to:debug/my-app.debug.wasm
```

## Development

### Building and Testing

```bash
# Build the project (examples included in workspace)
cargo build

# Run all tests
cargo test

# Run specific test suites
cargo test -p common          # Common utilities
cargo test -p localized       # Localization
cargo test -p builder         # Integration tests

# Build examples (real-world usage)
cd crates/examples && cargo build
```

The project uses a library + binary architecture where `builder` crate provides both `builder::execute()` for programmatic use and a CLI binary. This eliminates nested cargo calls and allows the examples to be part of the workspace.

### External Dependencies

Some features require external tools to be installed:

- **FontForge**: Required for font processing commands
- **Sass**: Required for advanced SCSS features (dart-sass)
- **WASM target**: `rustup target add wasm32-unknown-unknown`

## Releasing a new version

Releases are pre-compiled with `cargo dist` for various platforms and uploaded to github releases.
These can be used by `cargo binstall` to install the binary.

1. Update the version in `Cargo.toml`.
2. Add a git tag with the version number. Ex: `git tag v0.0.1 -m"Version 0.0.1: message"`.
3. Push the tag to the repository. Ex: `git push --tags`.

## License

MIT