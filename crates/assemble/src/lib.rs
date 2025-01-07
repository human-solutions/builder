mod asset_ext;
// #[allow(dead_code)]
// mod asset_incl;
mod generator;
mod mime;

use asset_ext::AssetExt;
use builder_command::AssembleCmd;
use camino_fs::*;
use common::site_fs::parse_site;
use generator::generate_code;
use std::process::Command;
use tempfile::NamedTempFile;

pub fn run(cmd: &AssembleCmd) {
    log::info!("Running builder-assemble");

    let assets = parse_site(&cmd.site_root).unwrap();

    let out = generate_code(&assets);

    let tmp_file = NamedTempFile::new().unwrap();
    let tmp_path = Utf8PathBuf::from_path(tmp_file.path()).unwrap();
    tmp_path.write(out).unwrap();

    log::debug!("Formatting {tmp_path}");
    let status = Command::new("rustfmt").arg(&tmp_path).status().unwrap();
    if !status.success() {
        log::warn!("Failed to format {tmp_path}");
    }

    let formatted = tmp_path.read_bytes().unwrap();
    if let Some(code_file) = &cmd.code_file {
        if code_file.exists() {
            let current = code_file.read_bytes().unwrap();
            if current == formatted {
                log::info!("No change detected, skipping {}", code_file);
                return;
            }
        } else if let Some(parent) = code_file.parent() {
            parent.mkdirs().unwrap();
        }
        code_file.write(formatted).unwrap();
    }
    if let Some(url_env_file) = &cmd.url_env_file {
        let mut envs = assets.iter().map(|a| a.url_const()).collect::<Vec<_>>();
        envs.sort();

        log::info!("Writing URL envs to {url_env_file}");
        url_env_file.write(envs.join("\n")).unwrap();
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
