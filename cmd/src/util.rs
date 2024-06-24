use anyhow::{bail, Context, Result};
use camino::Utf8Path;
use fs_err as fs;
use toml_edit::{Item, TableLike};

pub fn filehash(file: &Utf8Path) -> Result<String> {
    let content = fs::read(file)?;
    let hash = seahash::hash(&content);
    Ok(hash.to_string())
}

pub fn parse_vec<T, F: Fn(&dyn TableLike) -> Result<T>>(item: &Item, f: F) -> Result<Vec<T>> {
    let mut vals = Vec::new();
    if let Some(arr) = item.as_array() {
        for entry in arr {
            let table = entry
                .as_inline_table()
                .context("Expected an inline table")?;

            vals.push(f(table)?)
        }
    } else if let Some(arr_tbl) = item.as_array_of_tables() {
        for table in arr_tbl {
            vals.push(f(table)?)
        }
    } else {
        bail!("Expected an array of tables or an array")
    }
    Ok(vals)
}
