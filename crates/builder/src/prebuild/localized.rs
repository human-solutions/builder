use fs_err as fs;
use serde::Deserialize;
use unic_langid::LanguageIdentifier;

use crate::anyhow::{bail, Context, Result};
use crate::generate::Output;
use crate::Config;
use camino::Utf8PathBuf;

#[derive(Debug, Default, Deserialize)]
pub struct Localized {
    /// the path to the folder containing the localised files
    pub path: Utf8PathBuf,
    /// The file extension (file type)

    #[serde(rename = "file-extension")]
    pub file_extension: String,
    /// output options
    pub out: Output,
}

impl Localized {
    pub fn url(&self, checksum: Option<String>) -> String {
        let ext = &self.file_extension;
        let filename = format!("{}.{ext}", self.path.iter().last().unwrap());
        self.out.url(&filename, checksum)
    }

    pub fn process(&self, info: &Config) -> Result<Vec<(LanguageIdentifier, Vec<u8>)>> {
        let folder = info
            .existing_manifest_dir_path(&self.path)
            .context("localized path not found")?;
        if !folder.is_dir() {
            bail!("localized path is not a directory");
        }

        let mut variants: Vec<(LanguageIdentifier, Vec<u8>)> = Vec::new();

        // list all file names in folder
        for file in folder.read_dir_utf8()? {
            let file = file?;
            let file_type = file.file_type()?;

            let file_extension_match = file
                .path()
                .extension()
                .map(|ext| ext == self.file_extension)
                .unwrap_or_default();

            if file_type.is_file() && file_extension_match {
                let loc = file.path().file_stem().unwrap();
                let langid: LanguageIdentifier = loc.parse().with_context(|| {
                    format!("Not a valid language identifier {loc} for {}", file.path())
                })?;
                let content = fs::read(file.path())?;
                variants.push((langid, content));
            }
        }

        variants.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));

        Ok(variants)
    }
}

trait LanguageIdExt {
    fn to_lowercase(&self) -> String;
}

impl LanguageIdExt for LanguageIdentifier {
    fn to_lowercase(&self) -> String {
        self.to_string().to_lowercase()
    }
}
