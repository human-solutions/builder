use anyhow::{bail, Context, Result};
use fs_err as fs;
use unic_langid::LanguageIdentifier;

use crate::{config::Localized, RuntimeInfo};

impl Localized {
    pub fn process(&self, info: &RuntimeInfo) -> Result<Vec<(LanguageIdentifier, Vec<u8>)>> {
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
                .map(|ext| ext == self.file_ext)
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

        Ok(variants)
    }
}
