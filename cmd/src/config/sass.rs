use anyhow::{bail, Result};
use camino::Utf8PathBuf;
use toml_edit::TableLike;

use crate::ext::TomlValueExt;

use super::Out;

#[derive(Debug, Default)]
pub struct Sass {
    pub file: Utf8PathBuf,
    pub optimize: bool,
    pub out: Out,
}

impl Sass {
    pub fn try_parse(table: &dyn TableLike) -> Result<Self> {
        let mut me = Sass::default();
        for (key, value) in table.iter() {
            let value = value.as_value().unwrap();
            match key {
                "file" => me.file = value.try_path()?,
                "optimize" => me.optimize = value.try_bool()?,
                "out" => me.out = Out::try_parse(value)?,
                _ => bail!("Invalid key: {key} (value: '{value}'"),
            }
        }
        Ok(me)
    }
}
