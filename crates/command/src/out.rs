use camino_fs::*;
use icu_locid::LanguageIdentifier;
use std::{collections::BTreeMap, fmt::Display, str::FromStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Encoding {
    Brotli,
    Gzip,
    Identity,
}

impl Display for Encoding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Encoding {
    pub fn name(&self) -> &'static str {
        match self {
            Encoding::Brotli => "Brotli",
            Encoding::Gzip => "Gzip",
            Encoding::Identity => "Identity",
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Encoding::Brotli => "br",
            Encoding::Gzip => "gzip",
            Encoding::Identity => "",
        }
    }

    pub fn add_encoding(&self, path: &Utf8Path) -> Utf8PathBuf {
        if let Some(enc) = self.file_ending() {
            let ext = path.extension().unwrap_or_default();
            if !ext.ends_with(enc) {
                return path.with_extension(format!("{ext}.{enc}"));
            }
        }
        path.to_path_buf()
    }

    pub fn file_ending(&self) -> Option<&str> {
        match self {
            Encoding::Brotli => Some("br"),
            Encoding::Gzip => Some("gzip"),
            Encoding::Identity => None,
        }
    }
}

/// Metadata collected during file writing operations for asset code generation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetMetadata {
    pub url_path: String,
    pub folder: Option<String>,
    pub name: String,
    pub hash: Option<String>,
    pub ext: String,
    pub available_encodings: Vec<Encoding>,
    pub available_languages: Option<Vec<LanguageIdentifier>>,
    pub mime: String,
}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct Output {
    /// Folder where the output files should be written
    pub dir: Utf8PathBuf,

    pub site_dir: Option<Utf8PathBuf>,

    brotli: bool,

    gzip: bool,

    uncompressed: bool,

    /// Overrides the encoding settings and writes all possible encodings
    all_encodings: bool,

    pub checksum: bool,

    /// Optional path to write file hashes as a Rust file
    pub hash_output_path: Option<Utf8PathBuf>,

    /// Collected asset metadata during file operations
    pub asset_metadata: Vec<AssetMetadata>,
}

impl Output {
    pub fn new<P: Into<Utf8PathBuf>>(dir: P) -> Self {
        Self {
            dir: dir.into(),
            site_dir: None,
            brotli: false,
            gzip: false,
            uncompressed: false,
            all_encodings: false,
            checksum: false,
            hash_output_path: None,
            asset_metadata: Vec::new(),
        }
    }

    pub fn new_compress_and_sum<P: Into<Utf8PathBuf>>(dir: P) -> Self {
        Self {
            dir: dir.into(),
            site_dir: None,
            brotli: true,
            gzip: true,
            uncompressed: true,
            all_encodings: true,
            checksum: true,
            hash_output_path: None,
            asset_metadata: Vec::new(),
        }
    }

    pub fn new_compress<P: Into<Utf8PathBuf>>(dir: P) -> Self {
        Self {
            dir: dir.into(),
            site_dir: None,
            brotli: true,
            gzip: true,
            uncompressed: true,
            all_encodings: true,
            checksum: false,
            hash_output_path: None,
            asset_metadata: Vec::new(),
        }
    }

    pub fn site_dir<P: Into<Utf8PathBuf>>(mut self, dir: P) -> Self {
        self.site_dir = Some(dir.into());
        self
    }

    pub fn hash_output_path<P: Into<Utf8PathBuf>>(mut self, path: P) -> Self {
        self.hash_output_path = Some(path.into());
        self
    }

    pub fn uncompressed(&self) -> bool {
        // if none are set, then default to uncompressed
        let default_uncompressed = !self.uncompressed && !self.brotli && !self.gzip;
        self.uncompressed || default_uncompressed || self.all_encodings
    }

    pub fn brotli(&self) -> bool {
        self.brotli || self.all_encodings
    }

    pub fn gzip(&self) -> bool {
        self.gzip || self.all_encodings
    }

    pub fn encodings(&self) -> Vec<Encoding> {
        let mut encodings = Vec::new();
        if self.gzip() {
            encodings.push(Encoding::Gzip);
        }
        if self.brotli() {
            encodings.push(Encoding::Brotli);
        }
        if self.uncompressed() {
            encodings.push(Encoding::Identity);
        }
        encodings
    }

    /// Generates asset code from collected metadata and writes it to the specified destination
    pub fn generate_asset_code(&self, dest: &str) -> Result<(), std::io::Error> {
        if self.asset_metadata.is_empty() {
            return Ok(()); // No assets to generate
        }

        let code = self.generate_asset_code_content();
        std::fs::write(dest, code)
    }

    /// Generates the asset code content as a string
    pub fn generate_asset_code_content(&self) -> String {
        let provider_fn = self.generate_provider_function();
        let asset_sets = self.generate_asset_sets();
        let catalog = self.generate_asset_catalog();

        format!(
            r#"// Generated asset code using builder-assets crate
// This file is auto-generated. Do not edit manually.

use builder_assets::*;
use icu_locid::langid;

{provider_fn}

{asset_sets}

{catalog}
"#
        )
    }

    /// Generates the provider function based on the output directory
    fn generate_provider_function(&self) -> String {
        let base_path = self.dir.as_str();
        format!(
            r#"/// Provider function for loading asset data from filesystem
fn load_asset(path: &str) -> Option<Vec<u8>> {{
    let full_path = format!("{base_path}/{{path}}");
    std::fs::read(full_path).ok()
}}"#
        )
    }

    /// Generates static AssetSet declarations
    fn generate_asset_sets(&self) -> String {
        let mut deduplicated: BTreeMap<String, &AssetMetadata> = BTreeMap::new();

        // Deduplicate by URL path (translations generate multiple metadata entries)
        for metadata in &self.asset_metadata {
            deduplicated.insert(metadata.url_path.clone(), metadata);
        }

        deduplicated
            .values()
            .map(|metadata| self.generate_single_asset_set(metadata))
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    /// Generates a single static AssetSet
    fn generate_single_asset_set(&self, metadata: &AssetMetadata) -> String {
        let const_name = self.generate_const_name(&metadata.name, &metadata.ext);

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
    fn generate_asset_catalog(&self) -> String {
        let mut deduplicated: BTreeMap<String, &AssetMetadata> = BTreeMap::new();

        // Deduplicate by URL path
        for metadata in &self.asset_metadata {
            deduplicated.insert(metadata.url_path.clone(), metadata);
        }

        let asset_refs = deduplicated
            .values()
            .map(|metadata| {
                let const_name = self.generate_const_name(&metadata.name, &metadata.ext);
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
    pub fn generate_const_name(&self, name: &str, ext: &str) -> String {
        format!("{}_{}", name, ext)
            .to_uppercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '_' })
            .collect()
    }
}

impl Display for Output {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "dir={}\t", self.dir)?;
        if let Some(site_dir) = &self.site_dir {
            write!(f, "site_dir={}\t", site_dir)?;
        }
        write!(f, "brotli={}\t", self.brotli)?;
        write!(f, "gzip={}\t", self.gzip)?;
        write!(f, "uncompressed={}\t", self.uncompressed)?;
        write!(f, "all_encodings={}\t", self.all_encodings)?;
        write!(f, "checksum={}\t", self.checksum)?;
        if let Some(hash_output_path) = &self.hash_output_path {
            write!(f, "hash_output_path={}\t", hash_output_path)?;
        }
        Ok(())
    }
}

impl FromStr for Output {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut cmd = Output::default();
        for item in s.split('\t').filter(|s| !s.is_empty()) {
            let (key, value) = item.split_once('=').unwrap();

            match key {
                "dir" => cmd.dir = value.into(),
                "site_dir" => cmd.site_dir = Some(value.into()),
                "brotli" => cmd.brotli = value.parse().unwrap(),
                "gzip" => cmd.gzip = value.parse().unwrap(),
                "uncompressed" => cmd.uncompressed = value.parse().unwrap(),
                "all_encodings" => cmd.all_encodings = value.parse().unwrap(),
                "checksum" => cmd.checksum = value.parse().unwrap(),
                "hash_output_path" => cmd.hash_output_path = Some(value.into()),
                _ => panic!("unknown key: {}", key),
            }
        }
        Ok(cmd)
    }
}
