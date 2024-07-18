use anyhow::{Context, Result};
use serde::Serialize;

use crate::types::ValueWrapper;

use super::installer::{Binary, Installer};

#[derive(Serialize)]
pub struct Setup {
    target: Option<String>,
    #[serde(rename = "cmd-alias")]
    cmd_alias: Option<String>,
    version: Option<String>,
    version_cmd: Option<String>,
    installer: Installer,
    watch: Vec<String>,
    args: Option<String>,
}

impl Setup {
    pub fn try_from_value(value: &ValueWrapper) -> Result<Self> {
        if let ValueWrapper::Single(value) = value {
            let mut cmd_alias = None;
            let mut version = None;
            let mut version_cmd = None;
            let mut installer = Installer::default();
            let mut watch = Vec::new();
            let mut args = None;

            for (key, val) in value
                .as_object()
                .context("Failed to read install data as object")?
            {
                if key == "binstall" {
                    installer = Installer::Binstall(
                        val.as_str()
                            .context("Failed to read binstall value as string")?
                            .to_string(),
                    );
                } else if key == "version" {
                    version = Some(
                        val.as_str()
                            .context("Failed to read version value as string")?
                            .to_string(),
                    );
                } else if key == "version-cmd" {
                    version_cmd = Some(
                        val.as_str()
                            .context("Failed to read version-cmd value as string")?
                            .to_string(),
                    );
                } else if key == "install" {
                    installer = Installer::Cargo;
                } else if key == "watch" {
                    for watch_val in val
                        .as_array()
                        .context("Failed to read watch value as array")?
                    {
                        watch.push(
                            watch_val
                                .as_str()
                                .context("Failed to read watch value as string")?
                                .to_string(),
                        );
                    }
                } else if key == "args" {
                    args = Some(
                        val.as_str()
                            .context("Failed to read args value as string")?
                            .to_string(),
                    );
                } else if key == "cmd-alias" {
                    cmd_alias = Some(
                        val.as_str()
                            .context("Failed to read cmd-alias value as string")?
                            .to_string(),
                    );
                }
            }

            Ok(Self {
                cmd_alias,
                target: None,
                version,
                version_cmd,
                installer,
                watch,
                args,
            })
        } else {
            Err(anyhow::Error::msg(
                "Expected install data from table but install data defined as table array",
            ))
        }
    }

    pub fn with_target(mut self, target: Option<String>) -> Result<Self> {
        if target.is_some() && self.cmd_alias.is_none() {
            anyhow::bail!("cmd-alias is required when target is defined");
        }

        self.target = target;

        Ok(self)
    }

    pub fn check(&self, name: &str) -> Result<Binary> {
        self.installer
            .check(
                name,
                self.cmd_alias.as_ref(),
                self.version_cmd.as_ref(),
                &self.target,
            )
            .context(format!(
                "Failed to check for '{}' target",
                self.target.as_ref().unwrap_or(&"".to_owned())
            ))
    }
}
