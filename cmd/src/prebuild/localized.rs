use fs_err as fs;
use unic_langid::LanguageIdentifier;

use crate::anyhow::{bail, Context, Result};
use crate::generate::Output;
use camino::Utf8PathBuf;
use toml_edit::TableLike;

use crate::ext::TomlValueExt;

use super::args::PrebuildArgs;

#[derive(Debug, Default)]
pub struct Localized {
    /// the path to the folder containing the localised files
    pub path: Utf8PathBuf,
    /// The file extension (file type)
    pub file_ext: String,
    /// output options
    pub out: Output,
}

impl Localized {
    pub fn try_parse(table: &dyn TableLike) -> Result<Self> {
        let mut me = Localized::default();
        for (key, value) in table.iter() {
            let value = value.as_value().unwrap();
            match key {
                "path" => me.path = value.try_path()?,
                "file-extension" => me.file_ext = value.try_string()?,
                "out" => me.out = Output::try_parse(value)?,
                _ => bail!("Invalid key: {key} (value: '{value}'"),
            }
        }
        Ok(me)
    }

    pub fn url(&self, checksum: Option<String>) -> String {
        let ext = &self.file_ext;
        let filename = format!("{}.{ext}", self.path.iter().last().unwrap());
        self.out.url(&filename, checksum)
    }

    pub fn process(&self, info: &PrebuildArgs) -> Result<Vec<(LanguageIdentifier, Vec<u8>)>> {
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
