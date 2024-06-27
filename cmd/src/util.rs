use std::time::{Duration, SystemTime};

use crate::anyhow::{bail, Context, Result};
use base64::engine::general_purpose::URL_SAFE;
use base64::prelude::*;
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

pub fn timehash() -> String {
    let epoch_to_y2k: Duration = Duration::from_secs(946_684_800);
    let epoch = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap();
    let secs = (epoch - epoch_to_y2k).as_secs();

    let bytes = secs.to_be_bytes();
    let mut start = 0;
    while bytes[start] == 0 {
        start += 1;
    }
    URL_SAFE.encode(&bytes[start..])
}
