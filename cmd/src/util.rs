use anyhow::Result;
use camino::Utf8Path;
use fs_err as fs;

pub fn filehash(file: &Utf8Path) -> Result<String> {
    let content = fs::read(file)?;
    let hash = seahash::hash(&content);
    Ok(hash.to_string())
}
