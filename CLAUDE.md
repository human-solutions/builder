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

The tool works by:
1. Reading a YAML configuration file (builder.yaml format)
2. Parsing it into a `BuilderCmd` structure containing multiple command types using serde
3. Executing each command in sequence through their respective modules

## Development Commands

### Building and Testing
```bash
# Clean build workflow (build builder binary first, then everything else)
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
```

### External Dependencies Required for Testing
- **FontForge**: `sudo apt-get install fontforge` (Linux) or equivalent
- **Sass**: Download dart-sass from GitHub releases
- **WASM target**: `rustup target add wasm32-unknown-unknown`

### Running the Tool
The builder binary expects a YAML configuration file as its first argument:
```bash
./target/debug/builder path/to/builder.yaml
```

### Release Process
1. Update version in `Cargo.toml`
2. Create and push git tag: `git tag v0.1.20 -m"Version 0.1.20: description"`
3. CI automatically builds and releases via cargo-dist

## Key Design Patterns

- **Command Pattern**: Each build operation is implemented as a separate command struct with its own module
- **Configuration-driven**: The tool is entirely driven by TOML configuration files
- **Workspace architecture**: Modular design with separate crates for different responsibilities
- **Error handling**: Uses `anyhow` for error handling throughout
- **File system abstraction**: Uses `camino-fs` for UTF-8 path handling

## Working with Command Modules

When adding new commands or modifying existing ones:
1. Each command has its own module in `crates/command/src/`
2. Commands must implement `Display` and `FromStr` for serialization
3. Add the command variant to the `Cmd` enum in `lib.rs`
4. Update the match statements in both the enum implementation and main dispatcher
5. Create a corresponding crate in `crates/` for the actual implementation

The builder uses YAML serialization via serde for configuration files, providing human-readable and standard format handling with automatic field serialization.
