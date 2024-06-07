use anyhow::{bail, Result};
use camino::Utf8PathBuf;
use toml_edit::Value;

use crate::ext::TomlValueExt;

#[derive(Default, Debug)]
pub struct Out {
    pub brotli: bool,
    pub gzip: bool,
    pub uncompressed: bool,
    pub checksum: bool,
    /// sub-folder in generated site
    pub site_folder: Option<Utf8PathBuf>,
}

impl Out {
    pub fn try_parse(toml: &Value) -> Result<Self> {
        let mut me = Out::default();

        for (key, value) in toml.try_table()? {
            match key {
                "brotli" => me.brotli = value.try_bool()?,
                "gzip" => me.gzip = value.try_bool()?,
                "uncompressed" => me.uncompressed = value.try_bool()?,
                "checksum" => me.checksum = value.try_bool()?,
                "site-folder" => me.site_folder = Some(value.try_path()?),
                _ => bail!("Unexpected key: '{key}' with value: '{value}'"),
            }
        }

        Ok(me)
    }
}
