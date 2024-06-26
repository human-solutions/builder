use anyhow::{bail, Result};
use camino::Utf8PathBuf;
use toml_edit::TableLike;

use crate::ext::TomlValueExt;

use super::OutputOptions;

#[derive(Debug, Default)]
pub struct Sass {
    pub file: Utf8PathBuf,
    pub optimize: bool,
    pub out: OutputOptions,
}

impl Sass {
    pub fn try_parse(table: &dyn TableLike) -> Result<Self> {
        let mut me = Sass::default();
        for (key, value) in table.iter() {
            let value = value.as_value().unwrap();
            match key {
                "file" => me.file = value.try_path()?,
                "optimize" => me.optimize = value.try_bool()?,
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
        let filename = self.file.file_name().unwrap();
        format!("{folder}/{}{filename}", checksum.unwrap_or_default())
    }
}
