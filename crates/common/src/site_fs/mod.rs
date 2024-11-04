mod asset;
mod asset_path;
mod encoding;
#[cfg(test)]
mod tests;

use crate::debug;
pub use anyhow::Result;
pub use asset::Asset;
pub use asset_path::{AssetPath, SiteFile, TranslatedAssetPath};
use base64::{engine::general_purpose::URL_SAFE, Engine};
use builder_command::Output;
use camino_fs::*;
pub use encoding::AssetEncodings;
use icu_locid::LanguageIdentifier;
use seahash::SeaHasher;
use std::{collections::BTreeMap, hash::Hasher};

pub fn parse_site(root: &Utf8Path) -> Result<Vec<Asset>> {
    let mut assets: BTreeMap<String, Asset> = Default::default();

    debug!("Parsing site {root}");
    for path in root.ls().recurse() {
        if path.file_name() == Some(".DS_Store") {
            continue;
        }
        let rel_path = path.relative_to(&root).unwrap();
        debug!("Parsing asset from {rel_path}");
        if let Some(asset) = Asset::from_site_path(&rel_path) {
            let url = asset.to_url();
            if let Some(current) = assets.get_mut(&url) {
                current.join(asset);
            } else {
                assets.insert(url, asset);
            }
        }
    }
    Ok(assets.into_iter().map(|(_, v)| v).collect())
}

/// Copies all files recursively maintaining the relative
/// folder structure.
pub fn copy_files_to_site<F: Fn(&Utf8PathBuf) -> bool>(
    folder: &Utf8Path,
    predicate: F,
    output: &[Output],
) {
    for file in folder.ls().filter(predicate) {
        if !file.is_file() {
            debug!("Skipping non-file {file}");
            continue;
        }
        let bytes = file.read_bytes().unwrap();
        let rel_path = file.relative_to(folder).unwrap();
        let site_file = SiteFile::from_file(&rel_path);
        debug!("Copying {file} to {site_file}");
        write_file_to_site(&site_file, &bytes, output);
    }
}

pub fn write_file_to_site(site_file: &SiteFile, bytes: &[u8], output: &[Output]) {
    for out in output {
        let mut subdir = Utf8PathBuf::new();
        if let Some(dir) = &out.site_dir {
            subdir.push(dir);
        }
        if let Some(dir) = &site_file.site_dir {
            subdir.push(dir);
        }

        let checksum = if out.checksum {
            Some(checksum_from(&bytes))
        } else {
            None
        };
        let asset = AssetPath {
            subdir: subdir.into(),
            name_ext: site_file.clone(),
            checksum,
        };
        let path = asset.absolute_path(&out.dir);
        debug!("Writing to {path}");
        let encodings = AssetEncodings::from_output(out);
        encodings.write(&path, &bytes).unwrap()
    }
}

/// The relative path of the file is the path relative to the source folder,
/// It needs to be composed as `[<subdir>/]<name>.<ext>`
pub fn write_translations<P: Into<Utf8PathBuf>>(
    rel_path: P,
    lang_and_bytes: &[(LanguageIdentifier, Vec<u8>)],
    output: &[Output],
) {
    let rel_path = rel_path.into();
    debug!("Writing translations for {rel_path}");

    for out in output {
        let mut site_dir = Utf8PathBuf::new();
        // add any dir that is defined in out

        if let Some(dir) = &out.site_dir {
            site_dir.push(dir);
        }
        // add any relative dirs that comes from the source
        if let Some(dir) = rel_path.parent() {
            site_dir.push(dir)
        }

        let site_file = SiteFile::from_file(&rel_path).with_dir(&site_dir);

        out.dir
            .join(site_dir)
            .rm_matching(|p| {
                p.file_name()
                    .map_or(false, |f| f.starts_with(&site_file.name))
                    && p.extension().map_or(false, |e| e == &site_file.ext)
            })
            .unwrap();

        let checksum = if out.checksum {
            Some(checksum_for_all(
                lang_and_bytes.iter().map(|(_, b)| b.as_slice()),
            ))
        } else {
            None
        };
        let mut asset = TranslatedAssetPath {
            site_file,
            checksum,
            lang: "".to_string(),
        };
        for (lang, bytes) in lang_and_bytes {
            asset.lang = lang.to_string();
            let path = asset.absolute_path(&out.dir);
            debug!("Writing to {path}");
            let encodings = AssetEncodings::from_output(out);
            encodings.write(&path, &bytes).unwrap()
        }
    }
}

fn checksum_for_all<'a>(bytes_it: impl Iterator<Item = &'a [u8]>) -> String {
    let mut checksummer = SeaHasher::new();
    bytes_it.for_each(|bytes| checksummer.write(bytes));
    URL_SAFE.encode(checksummer.finish().to_be_bytes())
}

pub fn checksum_from(bytes: &[u8]) -> String {
    let sum = seahash::hash(bytes);
    URL_SAFE.encode(sum.to_be_bytes())
}
