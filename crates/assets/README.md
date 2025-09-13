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

// Generated provider function
fn load(path: &str) -> Option<Vec<u8>> {
    // Implementation depends on backing store (filesystem/embedded)
}

// Generated static asset sets
pub static STYLE: AssetSet = AssetSet::new(
    "/assets/style.jLsQ8S_Iyso=.css",
    FilePathParts {
        folder: Some("assets"),
        name: "style",
        hash: Some("jLsQ8S_Iyso="),
        ext: "css",
    },
    &[Encoding::Identity, Encoding::Brotli],
    None, // No translations
    "text/css",
    &load,
);

// Content negotiation
let asset = STYLE.asset_for("br, gzip", "en");
let data = asset.data_for();
```

## Next Steps

Migration Step 2: Add `.generate_asset_code(dest: &str)` method to `Output` struct.

## Dependencies

- `icu_locid`: Language identifier support
- `fluent_langneg`: Language negotiation algorithms