mod asset;
#[allow(dead_code)]
mod asset_incl;
mod file_name_parts;
mod generator;
mod mime;

use asset::Asset;
use builder_command::AssembleCmd;
use camino::Utf8PathBuf;
use common::Utf8PathExt;
use file_name_parts::FileNameParts;
use fs_err as fs;
use generator::generate_code;
use std::{collections::HashMap, process::Command, vec};
use tempfile::NamedTempFile;

pub fn run(cmd: &AssembleCmd) {
    log::info!("Running builder-assemble");

    let mut assets = vec![];
    for file in &cmd.files {
        log::info!("Processing reference file: {file}");
        let asset = asset_for_file(file);
        assets.push(asset);
    }
    for dir in &cmd.dirs {
        log::info!("Processing dir: {dir}");
        let wasm_assets = assets_for_dir(dir);
        assets.extend(wasm_assets);
    }
    for localized in &cmd.localized {
        let asset = asset_for_localised(localized);
        assets.push(asset);
    }

    let out = generate_code(&assets, &cmd.url_prefix);

    let tmp_file = NamedTempFile::new().unwrap();
    let tmp_path = Utf8PathBuf::from_path_buf(tmp_file.path().to_path_buf()).unwrap();
    fs::write(&tmp_path, out).unwrap();

    log::debug!("Formatting {tmp_path}");
    let status = Command::new("rustfmt").arg(&tmp_path).status().unwrap();
    if !status.success() {
        log::warn!("Failed to format {tmp_path}");
    }

    let formatted = fs::read(&tmp_path).unwrap();
    if let Some(code_file) = &cmd.code_file {
        if code_file.exists() {
            let current = fs::read(code_file).unwrap();
            if current == formatted {
                log::info!("No change detected, skipping {}", code_file);
                return;
            }
        }
        fs::write(code_file, formatted).unwrap();
    }
    if let Some(url_env_file) = &cmd.url_env_file {
        let envs = assets
            .iter()
            .map(|a| a.url_const(&cmd.url_prefix))
            .collect::<Vec<_>>()
            .join("\n");
        log::info!("Writing URL envs to {url_env_file}");
        fs::write(url_env_file, envs).unwrap();
    }
}

/// List the files in the wasm directory and generate assets for them
/// by identifying each file's base and encoding.
fn assets_for_dir(dir: &Utf8PathBuf) -> Vec<Asset> {
    log::debug!("Listing files in {dir}");
    let paths = dir.ls_files();
    let filenames = paths.iter().map(|f| f.file_name().unwrap());

    // file names, with a list of extensions
    let mut files: HashMap<String, Vec<FileNameParts>> = HashMap::new();
    for filename in filenames {
        let parts = FileNameParts::from(filename);
        let name = format!("{}.{}", parts.name, parts.ext);
        log::debug!("{filename} -> {}", parts.compression);
        files.entry(name).or_default().push(parts);
    }

    let last_folder = dir.file_name().unwrap();

    files
        .into_iter()
        .map(|(file, parts)| {
            let mut encodings = parts
                .iter()
                .map(|p| p.compression.to_string())
                .collect::<Vec<_>>();
            let mime = parts[0].mime.to_string();
            encodings.sort();
            let asset = Asset {
                name: file.to_string(),
                url: format!("{last_folder}/{file}"),
                mime,
                encodings,
                localizations: vec![],
            };
            asset
        })
        .collect()
}

fn asset_for_file(file: &Utf8PathBuf) -> Asset {
    log::debug!("finding assets for {file}");
    let filename = file.file_name().unwrap();
    let dir = file.parent().unwrap();
    let paths = dir.ls_files();

    let files = paths.iter().map(|f| f.file_name().unwrap());

    let mut encodings = vec![];
    let mut base: Option<FileNameParts> = None;

    for file in files {
        if !file.contains(filename) {
            continue;
        }

        let parts = FileNameParts::from(file);

        if let Some(base) = &base {
            log::debug!("Found base {base} for {file}");
            if base.ext != parts.ext {
                panic!("Extension mismatch: {} != {}", base.ext, parts.ext);
            }
            if base.checksum != parts.checksum {
                panic!(
                    "Checksum mismatch: {:?} != {:?}",
                    base.checksum, parts.checksum
                );
            }
        } else {
            log::debug!("Setting base to {file}");
            base = Some(parts.clone());
        }

        if !encodings.contains(&parts.compression.to_string()) {
            encodings.push(parts.compression.to_string());
        }
    }

    let base = base.unwrap();
    let checksum = &base.checksum;

    let ext = base.ext.to_string();
    let name = base.name.to_string();

    Asset {
        url: format!("{sum}{name}.{ext}", sum = checksum.unwrap_or("")),
        name,
        encodings,
        mime: base.mime.to_string(),
        localizations: vec![],
    }
}
fn asset_for_localised(dir: &Utf8PathBuf) -> Asset {
    let paths = dir.ls_files();
    let files = paths
        .iter()
        .map(|f| FileNameParts::from(f.file_name().unwrap()))
        .collect::<Vec<_>>();

    let checksum = files[0].checksum.clone();
    let ext = files[0].ext.to_string();

    let mut encodings = vec![];
    let mut localizations = vec![];

    for file in &files {
        if file.checksum != checksum {
            panic!("Checksum mismatch: {checksum:?} != {:?}", file.checksum);
        }
        if file.ext != ext {
            panic!("Extension mismatch: {ext:?} != {:?}", file.ext);
        }
        if !encodings.contains(&file.compression.to_string()) {
            encodings.push(file.compression.to_string());
        }
        localizations.push(file.name.to_owned());
    }
    localizations.sort();
    encodings.sort();
    let name = dir.file_name().unwrap();

    Asset {
        url: format!("{sum}{name}.{ext}", sum = checksum.unwrap_or("")),
        name: name.to_string(),
        encodings,
        mime: files[0].mime.to_string(),
        localizations,
    }
}

#[test]
fn test_encode() {
    use base64::{engine::general_purpose::URL_SAFE, Engine as _};

    let val = URL_SAFE.encode(0_u64.to_be_bytes());
    assert_eq!(val, "AAAAAAAAAAA=");

    let val = URL_SAFE.encode((u64::MAX / 2).to_be_bytes());
    assert_eq!(val, "f_________8=");

    let val = URL_SAFE.encode(u64::MAX.to_be_bytes());
    assert_eq!(val, "__________8=");
}
