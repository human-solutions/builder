mod asset;
mod file_name_parts;
mod generator;

use asset::Asset;
use builder_command::AssembleCmd;
use camino::Utf8PathBuf;
use common::Utf8PathExt;
use file_name_parts::FileNameParts;
use fs_err as fs;
use generator::generate_code;
use std::vec;

pub fn run(cmd: &AssembleCmd) {
    let mut assets = vec![];
    for file in &cmd.files {
        let asset = asset_for_file(file);
        assets.push(asset);
    }
    for localized in &cmd.localized {
        let asset = asset_for_localised(localized);
        assets.push(asset);
    }

    let out = generate_code(&assets, "module_path");
    if let Some(out_file) = &cmd.out_file {
        fs::write(out_file, out).unwrap();
    } else {
        println!("{out}");
    }
}

fn asset_for_file(file: &Utf8PathBuf) -> Asset {
    let filename = file.file_name().unwrap();
    let dir = file.parent().unwrap();
    let paths = dir.ls_files();

    let files = paths
        .iter()
        .map(|f| f.file_name().unwrap())
        .collect::<Vec<_>>();

    let mut encodings = vec![];
    let mut base: Option<FileNameParts> = None;

    for file in files {
        if !file.contains(filename) {
            continue;
        }
        let parts = FileNameParts::from(file);

        if let Some(base) = &base {
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
