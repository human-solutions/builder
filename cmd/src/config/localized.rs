use anyhow::{bail, Result};
use camino::Utf8PathBuf;
use toml_edit::TableLike;

use crate::ext::TomlValueExt;

use super::OutputOptions;

#[derive(Debug, Default)]
pub struct Localized {
    /// the path to the folder containing the localised files
    pub path: Utf8PathBuf,
    /// The file extension (file type)
    pub file_ext: String,
    /// output options
    pub out: OutputOptions,
}

impl Localized {
    pub fn try_parse(table: &dyn TableLike) -> Result<Self> {
        let mut me = Localized::default();
        for (key, value) in table.iter() {
            let value = value.as_value().unwrap();
            match key {
                "path" => me.path = value.try_path()?,
                "file-extension" => me.file_ext = value.try_string()?,
                "out" => me.out = OutputOptions::try_parse(value)?,
                _ => bail!("Invalid key: {key} (value: '{value}'"),
            }
        }
        Ok(me)
    }

    pub fn url(&self, checksum: Option<String>) -> String {
        let folder = if let Some(folder) = self.out.folder.as_ref() {
            format!("/{folder}")
        } else {
            "".to_string()
        };
        let filename = self.path.iter().last().unwrap();
        let ext = &self.file_ext;
        let checksum = checksum.unwrap_or_default();
        format!("{folder}/{checksum}{filename}.{ext}")
    }
}
