use anyhow::{Context, Result};
use camino::Utf8PathBuf;
use toml_edit::{InlineTable, Value};

pub trait TomlValueExt {
    fn try_path(&self) -> Result<Utf8PathBuf>;
    fn try_bool(&self) -> Result<bool>;
    fn try_table(&self) -> Result<&InlineTable>;
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
}
