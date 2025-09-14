mod asset;
mod asset_generation_integration_test;
mod asset_path;
mod encoding;
#[cfg(test)]
mod tests;

use crate::hash_output::HashCollector;
use crate::{debug, is_trace, log_trace};
pub use anyhow::Result;
pub use asset::Asset;
pub use asset_path::{AssetPath, SiteFile, TranslatedAssetPath};
use base64::{Engine, engine::general_purpose::URL_SAFE};
use builder_command::{AssetMetadata, Encoding as CmdEncoding, Output};
use camino_fs::*;
pub use encoding::AssetEncodings;
use icu_locid::LanguageIdentifier;
use seahash::SeaHasher;
use std::sync::OnceLock;
use std::{collections::BTreeMap, hash::Hasher, sync::Mutex};

// Global hash collector for tracking file hashes across all output operations
static HASH_COLLECTORS: OnceLock<Mutex<BTreeMap<Utf8PathBuf, HashCollector>>> = OnceLock::new();

fn get_hash_collectors() -> &'static Mutex<BTreeMap<Utf8PathBuf, HashCollector>> {
    HASH_COLLECTORS.get_or_init(|| Mutex::new(BTreeMap::new()))
}

/// Finalizes hash collection and writes all accumulated hashes to their respective output files
pub fn finalize_hash_outputs() -> Result<()> {
    let collectors = get_hash_collectors().lock().unwrap();
    for (output_path, collector) in collectors.iter() {
        collector.write_to_rust_file(output_path)?;
        log_trace!("SITE_FS", "Wrote hash file to: {}", output_path);
    }
    Ok(())
}

pub fn parse_site(root: &Utf8Path) -> Result<Vec<Asset>> {
    let mut assets: BTreeMap<String, Asset> = Default::default();
    let mut file_count = 0;

    debug!("Parsing site {root}");
    for path in root.ls().recurse() {
        if path.file_name() == Some(".DS_Store") {
            if is_trace() {
                log_trace!("SITE_FS", "Skipping .DS_Store file: {}", path);
            }
            continue;
        }
        file_count += 1;
        let rel_path = path.relative_to(root).unwrap();
        log_trace!("SITE_FS", "Parsing asset from: {}", rel_path);
        if let Some(asset) = Asset::from_site_path(rel_path) {
            let url = asset.to_url();
            if let Some(current) = assets.get_mut(&url) {
                log_trace!("SITE_FS", "Merging asset with existing URL: {}", url);
                current.join(asset);
            } else {
                log_trace!("SITE_FS", "New asset URL: {}", url);
                assets.insert(url, asset);
            }
        }
    }

    debug!(
        "Parsed {} files into {} unique assets from site root",
        file_count,
        assets.len()
    );
    Ok(assets.into_values().collect())
}

/// Copies all files recursively maintaining the relative
/// folder structure.
pub fn copy_files_to_site<F: Fn(&Utf8PathBuf) -> bool>(
    folder: &Utf8Path,
    recursive: bool,
    predicate: F,
    output: &mut [Output],
) {
    let mut copied_count = 0;
    let mut total_size = 0u64;

    for file in folder.ls().recurse_if(move |_| recursive).filter(predicate) {
        if !file.is_file() {
            log_trace!("SITE_FS", "Skipping non-file: {}", file);
            continue;
        }
        let bytes = file.read_bytes().unwrap();
        total_size += bytes.len() as u64;
        let rel_path = file.relative_to(folder).unwrap();
        let site_file = SiteFile::from_relative_path(rel_path);
        log_trace!(
            "SITE_FS",
            "Copying {} ({} bytes) to {}",
            file,
            bytes.len(),
            site_file
        );
        write_file_to_site(&site_file, &bytes, output);
        copied_count += 1;
    }

    if copied_count > 0 {
        debug!(
            "Copied {} files ({} bytes total) from {}",
            copied_count, total_size, folder
        );
    }
}

pub fn write_file_to_site(site_file: &SiteFile, bytes: &[u8], output: &mut [Output]) {
    for out in output {
        let mut subdir = Utf8PathBuf::new();
        if let Some(dir) = &out.site_dir {
            subdir.push(dir);
        }
        if let Some(dir) = &site_file.site_dir {
            subdir.push(dir);
        }

        let checksum = if out.checksum {
            Some(checksum_from(bytes))
        } else {
            None
        };

        let asset = AssetPath {
            subdir,
            name_ext: site_file.clone(),
            checksum: checksum.clone(),
        };

        // Collect hash information if hash_output_path is configured
        if let Some(hash_output_path) = &out.hash_output_path
            && let Some(hash) = &checksum
        {
            let mut collectors = get_hash_collectors().lock().unwrap();
            let collector = collectors.entry(hash_output_path.clone()).or_default();

            // Create the file path for the hash entry (relative to site root)
            let file_path = if asset.subdir.as_str().is_empty() {
                format!("{}.{}", asset.name_ext.name, asset.name_ext.ext)
            } else {
                format!(
                    "{}/{}.{}",
                    asset.subdir, asset.name_ext.name, asset.name_ext.ext
                )
            };

            collector.add_entry(file_path, hash);
            log_trace!(
                "SITE_FS",
                "Added hash entry for: {} -> {}",
                asset.name_ext,
                hash
            );
        }

        // remove any files that have the same name and extension
        out.dir
            .join(&asset.subdir)
            .ls()
            .files()
            .filter(|path| {
                path.file_name()
                    .map(|name| asset.name_ext.match_base_name(name))
                    .unwrap_or(false)
            })
            .for_each(|f| {
                log_trace!("SITE_FS", "Removing existing file: {}", f);
                f.rm().unwrap();
            });

        let path = asset.absolute_path(&out.dir);
        let encodings = AssetEncodings::from_output(out);
        log_trace!(
            "SITE_FS",
            "Writing file: {} ({} bytes, encodings: {:?})",
            path,
            bytes.len(),
            encodings
        );
        encodings.write(&path, bytes).unwrap();

        // Collect asset metadata for code generation
        let url_path = if asset.subdir.as_str().is_empty() {
            format!("/{}.{}", asset.name_ext.name, asset.name_ext.ext)
        } else {
            format!(
                "/{}/{}.{}",
                asset.subdir, asset.name_ext.name, asset.name_ext.ext
            )
        };

        let metadata = AssetMetadata {
            url_path,
            folder: if asset.subdir.as_str().is_empty() {
                None
            } else {
                Some(asset.subdir.to_string())
            },
            name: asset.name_ext.name.clone(),
            hash: checksum.clone(),
            ext: asset.name_ext.ext.clone(),
            available_encodings: encodings
                .into_iter()
                .map(encoding_to_cmd_encoding)
                .collect(),
            available_languages: None,
            mime: crate::mime::mime_from_ext(&asset.name_ext.ext).to_string(),
        };
        out.asset_metadata.push(metadata.clone());

        // Register metadata for asset code generation if configured
        if let Some((asset_code_path, data_provider)) = &out.asset_code_generation {
            crate::asset_code_generation::register_asset_metadata_for_output(
                asset_code_path,
                vec![metadata],
                *data_provider,
                &out.dir,
            );
        }
    }
}

/// The relative path of the file is the path relative to the source folder,
/// It needs to be composed as `[<subdir>/]<name>.<ext>`
pub fn write_translations<P: Into<Utf8PathBuf>>(
    rel_path: P,
    lang_and_bytes: &[(LanguageIdentifier, Vec<u8>)],
    output: &mut [Output],
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
                    .is_some_and(|f| f.starts_with(&site_file.name))
                    && p.extension().is_some_and(|e| e == site_file.ext)
            })
            .unwrap();

        let checksum = if out.checksum {
            Some(checksum_for_all(
                lang_and_bytes.iter().map(|(_, b)| b.as_slice()),
            ))
        } else {
            None
        };

        // Collect hash information for translations if hash_output_path is configured
        if let Some(hash_output_path) = &out.hash_output_path
            && let Some(hash) = &checksum
        {
            let mut collectors = get_hash_collectors().lock().unwrap();
            let collector = collectors.entry(hash_output_path.clone()).or_default();

            for (lang, _) in lang_and_bytes {
                let file_path = if site_file.site_dir.is_some() {
                    format!(
                        "{}/{}.{}/{}.{}",
                        site_file.site_dir.as_ref().unwrap(),
                        site_file.name,
                        site_file.ext,
                        lang,
                        site_file.ext
                    )
                } else {
                    format!(
                        "{}.{}/{}.{}",
                        site_file.name, site_file.ext, lang, site_file.ext
                    )
                };
                collector.add_entry(file_path, hash);
                log_trace!(
                    "SITE_FS",
                    "Added translation hash entry for: {} ({}) -> {}",
                    site_file,
                    lang,
                    hash
                );
            }
        }

        let mut asset = TranslatedAssetPath {
            site_file: site_file.clone(),
            checksum: checksum.clone(),
            lang: "".to_string(),
        };
        for (lang, bytes) in lang_and_bytes {
            asset.lang = lang.to_string();
            let path = asset.absolute_path(&out.dir);
            debug!("Writing to {path}");
            let encodings = AssetEncodings::from_output(out);
            encodings.write(&path, bytes).unwrap()
        }

        // Collect translation metadata (one AssetSet for all languages)
        let languages: Vec<LanguageIdentifier> = lang_and_bytes
            .iter()
            .map(|(lang, _)| lang.clone())
            .collect();

        let url_path = if let Some(site_dir) = &site_file.site_dir {
            if site_dir.is_empty() {
                format!("/{}.{}", site_file.name, site_file.ext)
            } else {
                format!("/{}/{}.{}", site_dir, site_file.name, site_file.ext)
            }
        } else {
            format!("/{}.{}", site_file.name, site_file.ext)
        };

        let metadata = AssetMetadata {
            url_path,
            folder: site_file.site_dir.clone(),
            name: site_file.name.clone(),
            hash: checksum.clone(),
            ext: site_file.ext.clone(),
            available_encodings: AssetEncodings::from_output(out)
                .into_iter()
                .map(encoding_to_cmd_encoding)
                .collect(),
            available_languages: Some(languages),
            mime: crate::mime::mime_from_ext(&site_file.ext).to_string(),
        };
        out.asset_metadata.push(metadata.clone());

        // Register metadata for asset code generation if configured
        if let Some((asset_code_path, data_provider)) = &out.asset_code_generation {
            crate::asset_code_generation::register_asset_metadata_for_output(
                asset_code_path,
                vec![metadata],
                *data_provider,
                &out.dir,
            );
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

/// Converts internal Encoding enum to command Encoding enum
fn encoding_to_cmd_encoding(e: CmdEncoding) -> CmdEncoding {
    e // They're the same type
}
