mod asset_ext;
// #[allow(dead_code)]
// mod asset_incl;
mod generator;
mod mime;

use asset_ext::AssetExt;
use builder_command::AssembleCmd;
use camino_fs::*;
use common::site_fs::parse_site;
use common::{Timer, log_command, log_operation, log_trace};
use generator::generate_code;
use std::process::Command;
use tempfile::NamedTempFile;

pub fn run(cmd: &AssembleCmd) {
    let _timer = Timer::new("ASSEMBLE processing");
    log_command!("ASSEMBLE", "Processing site root: {}", cmd.site_root);

    let assets = parse_site(&cmd.site_root).unwrap();
    log_operation!("ASSEMBLE", "Found {} assets", assets.len());

    let out = generate_code(&assets);
    log_operation!("ASSEMBLE", "Generated {} bytes of code", out.len());

    let tmp_file = NamedTempFile::new().unwrap();
    let tmp_path = Utf8PathBuf::from_path(tmp_file.path()).unwrap();
    tmp_path.write(out).unwrap();

    log_operation!("ASSEMBLE", "Formatting generated code with rustfmt");
    let status = Command::new("rustfmt").arg(&tmp_path).status().unwrap();
    if !status.success() {
        common::warn_cargo!("ASSEMBLE: rustfmt failed, using unformatted code");
    } else {
        log_operation!("ASSEMBLE", "Code formatting successful");
    }

    let formatted = tmp_path.read_bytes().unwrap();
    if let Some(code_file) = &cmd.code_file {
        if code_file.exists() {
            let current = code_file.read_bytes().unwrap();
            if current == formatted {
                log_command!("ASSEMBLE", "No changes detected, skipping code file write");
                return;
            }
            log_operation!("ASSEMBLE", "Code file changed, updating: {}", code_file);
        } else {
            if let Some(parent) = code_file.parent() {
                log_trace!("ASSEMBLE", "Creating parent directory: {}", parent);
                parent.mkdirs().unwrap();
            }
            log_operation!("ASSEMBLE", "Creating new code file: {}", code_file);
        }
        code_file.write(formatted).unwrap();
    }
    if let Some(url_env_file) = &cmd.url_env_file {
        let mut envs = assets.iter().map(|a| a.url_const()).collect::<Vec<_>>();
        envs.sort();

        log_operation!(
            "ASSEMBLE",
            "Writing {} URL constants to: {}",
            envs.len(),
            url_env_file
        );
        url_env_file.write(envs.join("\n")).unwrap();
    }
}

#[test]
fn test_encode() {
    use base64::{Engine as _, engine::general_purpose::URL_SAFE};

    let val = URL_SAFE.encode(0_u64.to_be_bytes());
    assert_eq!(val, "AAAAAAAAAAA=");

    let val = URL_SAFE.encode((u64::MAX / 2).to_be_bytes());
    assert_eq!(val, "f_________8=");

    let val = URL_SAFE.encode(u64::MAX.to_be_bytes());
    assert_eq!(val, "__________8=");
}
