# Builder Assets Crate

This crate implements the unified asset system for the builder tool as defined in [Issue #109](https://github.com/human-solutions/builder/issues/109).

## Migration Step 1 ✅ COMPLETE

Created the `crates/assets` crate implementing the complete API specification from issue 109:

- **Encoding enum**: File encoding types (Brotli, Gzip, Identity)
- **FilePathParts struct**: Building blocks for file path construction
- **Asset struct**: Specific asset variant (encoding + language + provider)
- **AssetSet struct**: All variants of a logical asset with content negotiation
- **AssetCatalog struct**: Efficient URL-based asset lookups
- **Content negotiation**: HTTP-style language and encoding negotiation

## Key Features

- **No breakage**: Pure additive change, existing functionality unaffected
- **Content negotiation**: Uses `fluent-langneg` for language negotiation
- **File path construction**: Handles both regular and translated file patterns
- **Static lifetime ready**: Designed for generated code patterns
- **Comprehensive API**: All functionality specified in issue 109

## Usage

The crate is designed to be used by generated code from `AssembleCmd`:

```rust
use builder_assets::*;

// Generated for DataProvider::FileSystem
fn load_asset(path: &str) -> Option<Vec<u8>> {
    let base_path = builder_assets::get_asset_base_path_or_panic();
    let full_path = base_path.join(path);
    std::fs::read(full_path).ok()
}

// Generated for DataProvider::Embed
use rust_embed::Embed;

#[derive(Embed)]
#[folder = "/dist"]
pub struct AssetFiles;

fn load_asset(path: &str) -> Option<Vec<u8>> {
    AssetFiles::get(path).map(|f| f.data.into_owned())
}

// Generated static asset sets (same for both providers)
pub static STYLE_CSS: AssetSet = AssetSet {
    url_path: "/assets/style.jLsQ8S_Iyso=.css",
    file_path_parts: FilePathParts {
        folder: Some("assets"),
        name: "style",
        hash: Some("jLsQ8S_Iyso="),
        ext: "css",
    },
    available_encodings: &[Encoding::Identity, Encoding::Brotli],
    available_languages: None,
    mime: "text/css",
    provider: &load_asset,
};

// Content negotiation
let asset = STYLE_CSS.asset_for("br, gzip", "en");
let data = asset.data_for();
```

## Configuration

Configure asset code generation in your build scripts:

```rust
use builder_command::{BuilderCmd, DataProvider, Output, SassCmd};

// Build script configuration
BuilderCmd::new()
    .add_sass(SassCmd::new("styles/main.scss")
        .add_output(Output::new("dist")
            .asset_code_gen("src/assets.rs", DataProvider::FileSystem))) // Filesystem assets
    .run();
```

```rust
// Runtime configuration (required for DataProvider::FileSystem)
use builder_assets::set_asset_base_path;

fn main() {
    // Configure asset path for your deployment scenario
    set_asset_base_path("/opt/myapp/assets");    // Production
    // set_asset_base_path("./assets");         // Development
    // set_asset_base_path(exe_dir.join("assets")); // Relative to binary

    // Now you can use the generated assets
    let catalog = get_asset_catalog();
    let asset = catalog.get_asset("/style.css", "br", "en");
}
```

Asset code is automatically generated during `BuilderCmd.run()` after all file operations complete.

## Dependencies

- `icu_locid`: Language identifier support
- `fluent_langneg`: Language negotiation algorithms
- `rust-embed`: Asset embedding support (for generated code)