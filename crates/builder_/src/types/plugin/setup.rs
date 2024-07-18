use anyhow::{Context, Result};
use serde::Serialize;

use crate::types::ValueWrapper;

use super::installer::Installer;

#[derive(Serialize)]
pub struct Setup {
    target: Option<String>,
    version: Option<String>,
    version_cmd: Option<String>,
    installer: Installer,
    watch: Vec<String>,
    args: Option<String>,
}

impl Setup {
    pub fn try_from_value(value: &ValueWrapper) -> Result<Self> {
        if let ValueWrapper::Single(value) = value {
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
                }
            }

            Ok(Self {
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

    pub fn with_target(mut self, target: Option<String>) -> Self {
        self.target = target;
        self
    }
}
