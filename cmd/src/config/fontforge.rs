use anyhow::{Context, Result};
use camino::Utf8PathBuf;
use toml_edit::Item;

#[derive(Debug, Default)]
pub struct FontForge {
    pub file: Utf8PathBuf,
}

impl FontForge {
    pub fn try_parse(value: &Item) -> Result<Self> {
        let s = value.as_str().context("expected string")?;
        let path = Utf8PathBuf::from(s);
        Ok(Self { file: path })
    }
}
