use crate::{ext::TomlValueExt, generate::Output};

use crate::anyhow::{bail, Result};
use camino::Utf8PathBuf;
use toml_edit::TableLike;

#[derive(Debug, Default)]
pub struct File {
    pub path: Utf8PathBuf,
    pub out: Output,
}

impl File {
    pub fn try_parse(table: &dyn TableLike) -> Result<Self> {
        let mut me = Self::default();
        for (key, value) in table.iter() {
            let value = value.as_value().unwrap();
            match key {
                "path" => me.path = value.try_path()?,
                "out" => me.out = Output::try_parse(value)?,
                _ => bail!("Invalid key: {key} (value: '{value}'"),
            }
        }
        Ok(me)
    }

    pub fn url(&self, checksum: Option<String>) -> String {
        let filename = self.path.file_name().unwrap();
        self.out.url(filename, checksum)
    }
}
