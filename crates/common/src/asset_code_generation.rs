use anyhow;
use builder_command::{AssetMetadata, DataProvider};
use camino_fs::{Utf8Path, Utf8PathBuf};
use std::collections::BTreeMap;
use std::sync::{Mutex, OnceLock};

// Global storage for asset metadata from all outputs
#[derive(Debug, Clone)]
struct AssetCodeConfig {
    metadata: Vec<AssetMetadata>,
    provider: DataProvider,
    base_path: Utf8PathBuf,
}

static ASSET_CODE_CONFIGS: OnceLock<Mutex<BTreeMap<Utf8PathBuf, AssetCodeConfig>>> =
    OnceLock::new();

fn get_asset_code_configs() -> &'static Mutex<BTreeMap<Utf8PathBuf, AssetCodeConfig>> {
    ASSET_CODE_CONFIGS.get_or_init(|| Mutex::new(BTreeMap::new()))
}

/// Registers asset metadata for a specific output file path
pub fn register_asset_metadata_for_output(
    output_path: &Utf8Path,
    metadata: Vec<AssetMetadata>,
    provider: DataProvider,
    base_path: &Utf8Path,
) {
    let mut configs = get_asset_code_configs().lock().unwrap();
    let config = configs
        .entry(output_path.to_path_buf())
        .or_insert_with(|| AssetCodeConfig {
            metadata: Vec::new(),
            provider,
            base_path: base_path.to_path_buf(),
        });
    config.metadata.extend(metadata);
}

/// Finalizes asset code generation and writes all accumulated metadata to their respective output files
pub fn finalize_asset_code_outputs() -> anyhow::Result<()> {
    let configs = get_asset_code_configs().lock().unwrap();
    for (output_path, config) in configs.iter() {
        if !config.metadata.is_empty() {
            let code = generate_asset_code_content_with_provider(
                &config.metadata,
                config.provider,
                &config.base_path,
            );

            // Ensure parent directory exists
            if let Some(parent) = output_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            std::fs::write(output_path, code)?;
            crate::log_trace!("ASSET_CODE", "Wrote asset code to: {}", output_path);
        }
    }
    Ok(())
}

/// Generates the complete asset code content from metadata with provider support
pub fn generate_asset_code_content_with_provider(
    metadata: &[AssetMetadata],
    provider: DataProvider,
    base_path: &Utf8Path,
) -> String {
    let (imports, provider_fn, rust_embed) = match provider {
        DataProvider::Embed => {
            let imports = "use builder_assets::*;\nuse icu_locid::langid;\nuse rust_embed::Embed;"
                .to_string();
            let rust_embed = format!(
                r#"
#[derive(Embed)]
#[folder = "{}"]
pub struct AssetFiles;
"#,
                base_path
            );
            let provider_fn = r#"/// Provider function for loading embedded asset data
fn load_asset(path: &str) -> Option<Vec<u8>> {
    AssetFiles::get(path).map(|f| f.data.into_owned())
}"#
            .to_string();
            (imports, provider_fn, rust_embed)
        }
        DataProvider::FileSystem => {
            let imports =
                "use builder_assets::*;\nuse icu_locid::langid;\nuse std::path::Path;".to_string();
            let provider_fn = r#"/// Provider function for loading asset data from filesystem
///
/// # Panics
/// Panics if the asset base path has not been configured using set_asset_base_path().
fn load_asset(path: &str) -> Option<Vec<u8>> {
    let base_path = builder_assets::get_asset_base_path_or_panic();
    let full_path = base_path.join(path);
    std::fs::read(full_path).ok()
}"#
            .to_string();
            (imports, provider_fn, String::new())
        }
    };

    let asset_sets = generate_asset_sets(metadata);
    let catalog = generate_asset_catalog(metadata);

    format!(
        r#"// Generated asset code using builder-assets crate
// This file is auto-generated. Do not edit manually.

{imports}{rust_embed}

{provider_fn}

{asset_sets}

{catalog}
"#
    )
}

/// Generates the complete asset code content from metadata (backward compatibility)
pub fn generate_asset_code_content(metadata: &[AssetMetadata], _sample_url: &str) -> String {
    // Default to FileSystem provider for backward compatibility
    generate_asset_code_content_with_provider(
        metadata,
        DataProvider::FileSystem,
        &Utf8PathBuf::from(""),
    )
}

/// Generates static AssetSet declarations
fn generate_asset_sets(metadata: &[AssetMetadata]) -> String {
    let mut deduplicated: BTreeMap<String, &AssetMetadata> = BTreeMap::new();

    // Deduplicate by URL path (translations generate multiple metadata entries)
    for meta in metadata {
        deduplicated.insert(meta.url_path.clone(), meta);
    }

    deduplicated
        .values()
        .map(|metadata| generate_single_asset_set(metadata))
        .collect::<Vec<_>>()
        .join("\n\n")
}

/// Generates a single static AssetSet
fn generate_single_asset_set(metadata: &AssetMetadata) -> String {
    let const_name = generate_const_name(&metadata.name, &metadata.ext);

    let encodings = metadata
        .available_encodings
        .iter()
        .map(|e| format!("Encoding::{:?}", e))
        .collect::<Vec<_>>()
        .join(", ");

    let languages = if let Some(langs) = &metadata.available_languages {
        let lang_list = langs
            .iter()
            .map(|lang| format!(r#"langid!("{}")"#, lang))
            .collect::<Vec<_>>()
            .join(", ");
        format!("Some(&[{}])", lang_list)
    } else {
        "None".to_string()
    };

    let folder = metadata
        .folder
        .as_ref()
        .map(|f| format!(r#"Some("{}")"#, f))
        .unwrap_or_else(|| "None".to_string());

    let hash = metadata
        .hash
        .as_ref()
        .map(|h| format!(r#"Some("{}")"#, h))
        .unwrap_or_else(|| "None".to_string());

    format!(
        r#"pub static {const_name}: AssetSet = AssetSet {{
    url_path: "{url_path}",
    file_path_parts: FilePathParts {{
        folder: {folder},
        name: "{name}",
        hash: {hash},
        ext: "{ext}",
    }},
    available_encodings: &[{encodings}],
    available_languages: {languages},
    mime: "{mime}",
    provider: &load_asset,
}};"#,
        const_name = const_name,
        url_path = metadata.url_path,
        folder = folder,
        name = metadata.name,
        hash = hash,
        ext = metadata.ext,
        encodings = encodings,
        languages = languages,
        mime = metadata.mime,
    )
}

/// Generates the AssetCatalog
fn generate_asset_catalog(metadata: &[AssetMetadata]) -> String {
    let mut deduplicated: BTreeMap<String, &AssetMetadata> = BTreeMap::new();

    // Deduplicate by URL path
    for meta in metadata {
        deduplicated.insert(meta.url_path.clone(), meta);
    }

    let asset_refs = deduplicated
        .values()
        .map(|metadata| {
            let const_name = generate_const_name(&metadata.name, &metadata.ext);
            format!("        &{}", const_name)
        })
        .collect::<Vec<_>>()
        .join(",\n");

    if asset_refs.is_empty() {
        return "/// No assets available\npub static ASSETS: [&AssetSet; 0] = [];".to_string();
    }

    format!(
        r#"/// All available assets as a static array
pub static ASSETS: [&AssetSet; {}] = [
{}
];

/// Asset catalog for efficient URL-based lookups
pub fn get_asset_catalog() -> AssetCatalog {{
    AssetCatalog::from_assets(&ASSETS)
}}"#,
        deduplicated.len(),
        asset_refs
    )
}

/// Generates a constant name from an asset name and extension
pub fn generate_const_name(name: &str, ext: &str) -> String {
    format!("{}_{}", name, ext)
        .to_uppercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect()
}
