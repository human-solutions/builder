# Builder

A command-line tool for building web assets, WASM, and mobile libraries. Builder simplifies the build process by reading a configuration file and executing multiple build commands in sequence.

## Overview

Builder uses a two-phase architecture:

1. **Generation Phase**: Rust build scripts use the `BuilderCmd` struct with fluent builder pattern methods to configure build commands programmatically, then generate a `builder.toml` configuration file
2. **Execution Phase**: The `builder` CLI tool reads the configuration file and executes each build command in sequence

This design allows for both programmatic configuration from Rust build scripts and standalone CLI usage.

## Features

- **SASS/SCSS Compilation** - Compiles SCSS files using dart-sass (if available) or built-in grass compiler. Supports CSS optimization with LightningCSS, string replacements, and outputs with browser compatibility targets.

- **WASM Building** - Compiles Rust packages to WebAssembly for web targets. Runs `cargo build --target wasm32-unknown-unknown`, generates JS bindings with wasm-bindgen, optimizes with wasm-opt in release mode, and includes smart caching to skip unchanged builds.

- **Uniffi Bindings** - Generates Swift and Kotlin language bindings from UniFFI definition files (.udl). Features intelligent caching that compares UDL files, config files, and CLI parameters to avoid regeneration. Automatically fixes Swift modulemap files for framework usage.

- **Swift Package Generation** - Creates Swift packages using the swift-package crate. Configures build settings based on release/debug mode and respects global verbose settings.

- **FontForge Integration** - Processes SFD (Spline Font Database) files using FontForge to generate WOFF2 and OTF formats. Includes content-based caching via seahash, and on macOS automatically installs OTF fonts to `~/Library/Fonts/`.

- **Asset Assembly** - Scans asset directories and generates Rust code for asset management. Creates static variables, URL constants, and lookup functions. Generates formatted Rust code with rustfmt and includes change detection to avoid unnecessary regeneration.

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
builder-command = "0.1"
```

```rust
use builder_command::BuilderCmd;

fn main() {
    BuilderCmd::new()
        .add_sass(SassCmd::new("styles/main.scss", "dist/main.css"))
        .add_wasm(WasmProcessingCmd::new("src/lib.rs", "pkg"))
        .verbose(true)
        .run();
}
```

### CLI Usage

Builder can also be used directly with a configuration file:

```bash
builder path/to/builder.toml
```

The configuration file defines which build commands to execute and their parameters. Each command type has its own configuration options and will be executed in the order specified.

## Development

### Building and Testing

```bash
# Build the project
cargo build

# Run tests (requires external dependencies)
cargo nextest run

# Check code without building
cargo check
```

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