use std::time::{Duration, SystemTime};

use crate::anyhow::Result;
use base64::engine::general_purpose::URL_SAFE;
use base64::prelude::*;
use camino::Utf8Path;
use fs_err as fs;

pub fn filehash(file: &Utf8Path) -> Result<String> {
    let content = fs::read(file)?;
    let hash = seahash::hash(&content);
    Ok(hash.to_string())
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
