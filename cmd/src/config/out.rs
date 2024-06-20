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
    pub folder: Option<Utf8PathBuf>,
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
                "folder" => me.folder = Some(value.try_path()?),
                _ => bail!("Unexpected key: '{key}' with value: '{value}'"),
            }
        }

        Ok(me)
    }

    /// Encodings according to https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Accept-Encoding
    pub fn encodings(&self) -> Vec<String> {
        let mut encodings = vec![];
        if self.brotli {
            encodings.push("br".to_string());
        }
        if self.gzip {
            encodings.push("gzip".to_string());
        }
        if self.uncompressed {
            encodings.push("identity".to_string());
        }
        encodings
    }
}
