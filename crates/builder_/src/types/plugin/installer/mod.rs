mod binary;

use anyhow::Result;
pub use binary::{BinStatus, Binary};
use serde::Serialize;
use which::which;

use crate::cmd_runner::CmdRunner;

#[derive(Default, Serialize)]
pub enum Installer {
    Binstall(String),
    #[default]
    Cargo,
    Shell(String),
    Plugin(String),
}

impl Installer {
    pub fn check<'a>(
        &'a self,
        package: &str,
        cmd_alias: Option<&String>,
        version_cmd: Option<&String>,
        target: &'a Option<String>,
    ) -> Result<Binary> {
        let cmd = if let Some(alias) = cmd_alias {
            alias
        } else {
            package
        };

        if let Ok(path) = which(cmd) {
            let cmd_result = CmdRunner::run(
                path.to_str()
                    .unwrap_or_else(|| panic!("Failed to read binary path : {:?}", path)),
                &[version_cmd.unwrap_or(&"--version".to_string())],
            )
            .output();

            if let Ok(version) = cmd_result {
                let version = String::from_utf8(version.stdout)?.trim().to_string();
                return Ok(Binary {
                    name: package.to_string(),
                    status: BinStatus::Installed { version },
                    target,
                });
            }
        }

        Ok(Binary {
            name: package.to_string(),
            status: BinStatus::NotInstalled,
            target,
        })
    }
}
