use std::{collections::HashSet, fs};

use crate::{
    anyhow::{bail, Context, Result},
    generate::{Asset, Generator},
};
use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};
use unic_langid::LanguageIdentifier;

use crate::generate::Output;

use super::Config;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct LocalizedParams {
    /// the path to the folder containing the localised files
    pub path: Utf8PathBuf,
    /// The file extension (file type)

    #[serde(rename = "file-extension")]
    pub file_extension: String,
    /// output options
    pub out: Output,
    /// path (relative to the crate directory) to the generated asset rust module
    #[serde(rename = "generated-module")]
    pub generated_mod: Option<Utf8PathBuf>,
}

impl LocalizedParams {
    pub fn url(&self, checksum: Option<String>) -> String {
        let ext = &self.file_extension;
        let filename = format!("{}.{ext}", self.path.iter().last().unwrap());
        self.out.url(&filename, checksum)
    }

    pub fn process(
        &self,
        config: &Config,
        generator: &mut Generator,
        watched: &mut HashSet<String>,
    ) -> Result<()> {
        let variants = self.process_inner(config)?;
        let localizations = variants.iter().map(|(lang, _)| lang.clone()).collect();
        let site_dir = config.site_dir("localized");

        let filename = self.path.iter().last().unwrap();
        let ext = &self.file_extension;
        let hash = self
            .out
            .write_localized(&site_dir, filename, ext, variants)?;

        generator.add_asset(
            Asset::from_localized(self, hash, localizations),
            self.generated_mod.clone(),
        );
        watched.insert(self.path.to_string());

        Ok(())
    }

    fn process_inner(&self, config: &Config) -> Result<Vec<(LanguageIdentifier, Vec<u8>)>> {
        let folder = config
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
