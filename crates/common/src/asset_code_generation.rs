use anyhow;
use builder_command::{AssetMetadata, DataProvider};
use camino_fs::{Utf8Path, Utf8PathBuf};
use std::collections::{BTreeMap, HashSet};
use std::sync::{Mutex, OnceLock};

// Global storage for asset metadata from all outputs
#[derive(Debug, Clone)]
pub struct ProviderConfig {
    pub metadata: Vec<AssetMetadata>,
    pub base_path: Utf8PathBuf,
}

#[derive(Debug, Clone)]
pub struct AssetCodeConfig {
    pub embed_config: Option<ProviderConfig>,
    pub filesystem_config: Option<ProviderConfig>,
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
            embed_config: None,
            filesystem_config: None,
        });

    match provider {
        DataProvider::Embed => {
            let embed_config = config.embed_config.get_or_insert_with(|| ProviderConfig {
                metadata: Vec::new(),
                base_path: base_path.to_path_buf(),
            });
            embed_config.metadata.extend(metadata);
        }
        DataProvider::FileSystem => {
            let filesystem_config =
                config
                    .filesystem_config
                    .get_or_insert_with(|| ProviderConfig {
                        metadata: Vec::new(),
                        base_path: base_path.to_path_buf(),
                    });
            filesystem_config.metadata.extend(metadata);
        }
    }
}

/// Finalizes asset code generation and writes all accumulated metadata to their respective output files
pub fn finalize_asset_code_outputs() -> anyhow::Result<()> {
    let configs = get_asset_code_configs().lock().unwrap();
    for (output_path, config) in configs.iter() {
        // Check if we have any metadata to generate
        let has_embed = config
            .embed_config
            .as_ref()
            .is_some_and(|c| !c.metadata.is_empty());
        let has_filesystem = config
            .filesystem_config
            .as_ref()
            .is_some_and(|c| !c.metadata.is_empty());

        if has_embed || has_filesystem {
            let code = generate_multi_provider_asset_code(config);

            // Ensure parent directory exists
            if let Some(parent) = output_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            std::fs::write(output_path, code)?;
            crate::log_trace!(
                "ASSET_CODE",
                "Wrote multi-provider asset code to: {}",
                output_path
            );
        }
    }
    Ok(())
}

/// Generates asset code with multiple provider support
pub fn generate_multi_provider_asset_code(config: &AssetCodeConfig) -> String {
    let mut parts = Vec::new();

    // Header
    parts.push("// Generated asset code using builder-assets crate\n// This file is auto-generated. Do not edit manually.\n\n#[allow(unused_imports)]\nuse builder_assets::*;".to_string());

    // Generate provider functions and RustEmbed structs
    let (embed_provider, embed_struct) = if let Some(embed_config) = &config.embed_config {
        let embed_struct = format!(
            r#"#[derive(Embed)]
#[folder = "{}"]
pub struct EmbedAssetFiles;"#,
            embed_config.base_path
        );

        let provider = r#"/// Provider function for loading embedded asset data
fn load_embed_asset(path: &str) -> Option<Vec<u8>> {
    EmbedAssetFiles::get(path).map(|f| f.data.into_owned())
}
static LOAD_EMBED_ASSET: fn(&str) -> Option<Vec<u8>> = load_embed_asset;"#
            .to_string();

        (Some(provider), Some(embed_struct))
    } else {
        (None, None)
    };

    let filesystem_provider = if config.filesystem_config.is_some() {
        Some(
            r#"/// Provider function for loading asset data from filesystem
///
/// # Panics
/// Panics if the asset base path has not been configured using set_asset_base_path().
fn load_filesystem_asset(path: &str) -> Option<Vec<u8>> {
    let base_path = builder_assets::get_asset_base_path_or_panic();
    let clean_path = path.trim_start_matches('/');
    let full_path = base_path.join(clean_path);
    std::fs::read(full_path).ok()
}
static LOAD_FILESYSTEM_ASSET: fn(&str) -> Option<Vec<u8>> = load_filesystem_asset;"#
                .to_string(),
        )
    } else {
        None
    };

    // Add embed struct if needed
    if let Some(embed_struct) = embed_struct {
        parts.push(embed_struct);
    }

    // Add provider functions
    if let Some(embed_prov) = embed_provider {
        parts.push(embed_prov);
    }
    if let Some(fs_prov) = filesystem_provider {
        parts.push(fs_prov);
    }

    // Generate asset sets for each provider
    let mut all_metadata = Vec::new();

    // Collect all metadata first for cross-provider conflict detection
    if let Some(embed_config) = &config.embed_config {
        all_metadata.extend(embed_config.metadata.iter());
    }
    if let Some(fs_config) = &config.filesystem_config {
        all_metadata.extend(fs_config.metadata.iter());
    }

    // Check for naming conflicts across all providers
    check_global_naming_conflicts(&all_metadata);

    if let Some(embed_config) = &config.embed_config {
        let embed_assets =
            generate_provider_asset_sets(&embed_config.metadata, "&LOAD_EMBED_ASSET");
        if !embed_assets.is_empty() {
            parts.push(format!("// Embedded assets\n{}", embed_assets));
        }
    }

    if let Some(fs_config) = &config.filesystem_config {
        let fs_assets = generate_provider_asset_sets(&fs_config.metadata, "&LOAD_FILESYSTEM_ASSET");
        if !fs_assets.is_empty() {
            parts.push(format!("// Filesystem assets\n{}", fs_assets));
        }
    }

    // Generate unified catalog
    if !all_metadata.is_empty() {
        let owned_metadata: Vec<AssetMetadata> = all_metadata.into_iter().cloned().collect();
        let catalog = generate_asset_catalog(&owned_metadata);
        parts.push(catalog);
    }

    parts.join("\n\n")
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

/// Generates static AssetSet declarations for a specific provider
/// Note: Global conflict detection is handled separately in generate_multi_provider_asset_code
fn generate_provider_asset_sets(metadata: &[AssetMetadata], provider_ref: &str) -> String {
    let mut deduplicated: BTreeMap<String, &AssetMetadata> = BTreeMap::new();

    // Deduplicate by URL path (translations generate multiple metadata entries)
    for meta in metadata {
        deduplicated.insert(meta.url_path.clone(), meta);
    }

    deduplicated
        .values()
        .map(|metadata| generate_single_asset_set_with_provider(metadata, provider_ref))
        .collect::<Vec<_>>()
        .join("\n\n")
}

/// Generates a single static AssetSet with custom provider reference
fn generate_single_asset_set_with_provider(metadata: &AssetMetadata, provider_ref: &str) -> String {
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
    provider: {provider_ref},
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
        provider_ref = provider_ref,
    )
}

/// Checks for naming conflicts across all providers in a unified file
pub fn check_global_naming_conflicts(all_metadata: &[&AssetMetadata]) {
    let mut deduplicated: BTreeMap<String, &AssetMetadata> = BTreeMap::new();
    let mut used_names: HashSet<String> = HashSet::new();

    // Deduplicate by URL path first (same as existing logic)
    for meta in all_metadata {
        deduplicated.insert(meta.url_path.clone(), meta);
    }

    // Check for naming conflicts across all assets from all providers
    for metadata in deduplicated.values() {
        let const_name = generate_const_name(&metadata.name, &metadata.ext);
        if !used_names.insert(const_name.clone()) {
            panic!(
                "Asset constant name conflict across providers: '{}' would be generated by multiple assets.\n\
                 This conflict exists between assets from different providers (embed vs filesystem).\n\
                 Consider renaming one of the assets to avoid this conflict.\n\
                 Conflicting asset: {} ({})",
                const_name, metadata.name, metadata.url_path
            );
        }
    }
}
