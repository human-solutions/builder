use anyhow::{Context, Result};
use base64::engine::general_purpose::URL_SAFE;
use base64::prelude::*;
use camino::Utf8PathBuf;
use toml_edit::{InlineTable, Value};

pub trait TomlValueExt {
    fn try_path(&self) -> Result<Utf8PathBuf>;
    fn try_bool(&self) -> Result<bool>;
    fn try_table(&self) -> Result<&InlineTable>;
    fn try_string(&self) -> Result<String>;
}

impl TomlValueExt for Value {
    fn try_path(&self) -> Result<Utf8PathBuf> {
        let s = self
            .as_str()
            .with_context(|| format!("Expected a string, not '{self}'"))?;
        Ok(Utf8PathBuf::from(s))
    }

    fn try_bool(&self) -> Result<bool> {
        self.as_bool()
            .with_context(|| format!("Expected a bool, not '{self}'"))
    }

    fn try_table(&self) -> Result<&InlineTable> {
        self.as_inline_table()
            .with_context(|| format!("Expected a table {{ }}, not '{self}'"))
    }
    fn try_string(&self) -> Result<String> {
        self.as_str()
            .map(|s| s.to_string())
            .with_context(|| format!("Expected a string, not '{self}'"))
    }
}

pub trait ByteVecExt {
    fn base64_checksum(&self) -> String;
}

impl ByteVecExt for [u8] {
    fn base64_checksum(&self) -> String {
        let hash = seahash::hash(self);
        URL_SAFE.encode(hash.to_be_bytes())
    }
}

pub trait RustNaming {
    fn to_rust_module(&self) -> String;
    fn to_rust_const(&self) -> String;
}

impl RustNaming for str {
    fn to_rust_module(&self) -> String {
        self.replace('-', "_")
    }

    fn to_rust_const(&self) -> String {
        let mut s = String::with_capacity(self.len());
        for (i, char) in self.chars().enumerate() {
            if char == '.' {
                s.push('_');
                continue;
            } else if char == '_' {
                // allowed
            } else if !char.is_ascii_alphanumeric() {
                panic!("Only ascii chars and '.' allowed in rust constant names, not {char}")
            }
            if char.is_ascii_uppercase() && i != 0 {
                s.push('_');
                s.push(char);
            } else {
                s.push(char.to_ascii_uppercase());
            }
        }
        s
    }
}
