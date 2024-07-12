mod installer;
mod status;

use std::collections::HashMap;

use anyhow::{Context, Result};
use installer::Installer;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use status::{BinariesStatus, BinaryState};
use which::which;

use crate::cmd_runner::CmdRunner;

#[derive(Debug, Hash, Eq, PartialEq, Serialize, Deserialize, Clone)]
pub struct Binary {
    name: String,
    version: Option<String>,
}

impl Binary {
    fn check(&self, version_arg: &str) -> BinaryState {
        if let Ok(path) = which(&self.name) {
            let cmd_result = CmdRunner::run(
                path.to_str()
                    .unwrap_or_else(|| panic!("Failed to read binary path : {:?}", path)),
                &[version_arg],
            )
            .output();

            if let Ok(version) = cmd_result {
                return BinaryState::Installed {
                    version: String::from_utf8(version.stdout)
                        .unwrap_or_else(|e| panic!("{}: Failed to read version : {e}", self.name)),
                };
            }
        }

        BinaryState::NotInstalled
    }
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct BinaryCfg {
    version_arg: String,
    installer: Installer,
    watch: Vec<String>,
    args: Option<String>,
}

#[derive(Default, Serialize, Deserialize)]
pub struct Binaries(pub HashMap<Binary, BinaryCfg>);

impl Binaries {
    pub fn from_iter<'a, I>(bins_iter: I) -> Result<Self>
    where
        I: Iterator<Item = (&'a String, &'a Value)>,
    {
        let mut bins = HashMap::new();

        for (_, val) in bins_iter {
            binaries(val, &mut bins)?;
        }

        Ok(Self(bins))
    }

    pub fn insert_batch_obj(&mut self, obj: &Value) -> Result<()> {
        binaries(obj, &mut self.0)
    }

    pub fn extend(&mut self, cfg: Self) {
        self.0.extend(cfg.0)
    }

    fn check_status(&self) -> BinariesStatus {
        BinariesStatus::from(
            self.0
                .iter()
                .map(|(binary, bin_cfg)| (binary.clone(), binary.check(&bin_cfg.version_arg)))
                .collect::<HashMap<Binary, BinaryState>>(),
        )
    }
}

fn binaries(obj: &Value, bins: &mut HashMap<Binary, BinaryCfg>) -> Result<()> {
    for (bin, val) in obj
        .as_object()
        .context("Failed to retrieve binary object")?
    {
        let mut version = "";
        let mut version_arg = "--version";
        let mut installer = None;
        let mut watch = Vec::new();
        let mut args = None;

        for (key, val) in val
            .as_object()
            .context("Failed to retrieve binary installation data")?
        {
            match key.as_str() {
                "binstall" => {
                    let (name, ver) = val
                        .as_str()
                        .context(format!("{bin}: Failed to retrieve binstall package name"))?
                        .split_once('@')
                        .context(format!("{bin}: No version found for binstall package"))?;
                    version = ver;
                    installer = Some(Installer::Binstall(name.to_owned()));
                }
                "install" => installer = Some(Installer::Cargo),
                "shell" => {
                    installer = Some(Installer::Custom(
                        val.as_str()
                            .context(format!("{bin}: Failed to retrieve shell installer command"))?
                            .to_owned(),
                    ))
                }
                "version" => {
                    version = val
                        .as_str()
                        .context(format!("{bin}: Failed to retrieve version"))?
                }
                "version-arg" => {
                    version_arg = val.as_str().context(format!(
                        "{bin}: Failed to retrieve version command argument"
                    ))?
                }
                "arg" => {
                    args = Some(
                        val.as_str()
                            .context(format!("{bin}: Failed to retrieve args"))?
                            .to_owned(),
                    )
                }
                "watch" => {
                    watch = val
                        .as_array()
                        .context(format!("{bin}: Failed to retrieve watch list"))?
                        .iter()
                        .map(|v| {
                            v.as_str()
                                .unwrap_or_else(|| panic!("{bin}: Failed to retrieve watch entry"))
                                .to_owned()
                        })
                        .collect()
                }
                _ => {
                    installer = Some(Installer::Custom(
                        val.as_str()
                            .context(format!("{bin}: Failed to retrieve shell installer command"))?
                            .to_owned(),
                    ))
                }
            }
        }

        bins.insert(
            Binary {
                name: bin.to_owned(),
                version: Some(version.to_owned()),
            },
            BinaryCfg {
                version_arg: version_arg.to_owned(),
                installer: installer.context(format!("{bin}: No installer provided"))?,
                watch,
                args,
            },
        );
    }

    Ok(())
}
