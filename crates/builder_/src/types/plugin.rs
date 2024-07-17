use anyhow::{Context, Result};
use serde::Serialize;
use serde_json::Value;

use super::{phase::Phase, ValueWrapper};

#[derive(Default, Serialize)]
pub struct Plugin {
    pub name: String,
    pub prebuild: Vec<Action>,
    pub postbuild: Vec<Action>,
    pub setup: Vec<Setup>,
}

impl Plugin {
    pub fn push_action(&mut self, phase: &Phase, action: Option<String>, spec: Spec) -> Result<()> {
        if phase.is_pre_build() {
            if let Some(pos) = self.has_prebuild_action(&action) {
                if self.prebuild[pos].has_target(&spec.target) {
                    anyhow::bail!(format!(
                        "Prebuild action '{}' with target '{}' already exists",
                        action.unwrap_or_default(),
                        spec.target.unwrap_or_default()
                    ));
                } else {
                    self.prebuild[pos].specs.push(spec);
                }
            } else {
                self.prebuild.push(Action {
                    action,
                    specs: vec![spec],
                });
            }
        } else if phase.is_postbuild() {
            if let Some(pos) = self.has_postbuild_action(&action) {
                if self.postbuild[pos].has_target(&spec.target) {
                    anyhow::bail!(format!(
                        "Postbuild action '{}' with target '{}' already exists",
                        action.unwrap_or_default(),
                        spec.target.unwrap_or_default()
                    ));
                } else {
                    self.postbuild[pos].specs.push(spec);
                }
            } else {
                self.postbuild.push(Action {
                    action,
                    specs: vec![spec],
                });
            }
        }

        Ok(())
    }

    fn has_prebuild_action(&self, action_name: &Option<String>) -> Option<usize> {
        self.prebuild.iter().position(|a| a.action == *action_name)
    }

    fn has_postbuild_action(&self, action_name: &Option<String>) -> Option<usize> {
        self.prebuild.iter().position(|a| a.action == *action_name)
    }
}

#[derive(Serialize)]
pub struct Action {
    action: Option<String>,
    specs: Vec<Spec>,
}

impl Action {
    pub fn has_target(&self, target: &Option<String>) -> bool {
        self.specs.iter().any(|spec| spec.target == *target)
    }
}

#[derive(Serialize)]
pub struct Spec {
    assembly: Option<String>,
    target: Option<String>,
    profile: Option<String>,
    output: Value,
}

impl Spec {
    pub fn new(
        assembly: Option<String>,
        target: Option<String>,
        profile: Option<String>,
        output: ValueWrapper,
    ) -> Result<Self> {
        if let ValueWrapper::Single(val) = output {
            Ok(Self {
                assembly,
                target,
                profile,
                output: val,
            })
        } else {
            anyhow::bail!("Expected output value as single value but found as array");
        }
    }
}

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

#[derive(Default, Serialize)]
pub enum Installer {
    Binstall(String),
    #[default]
    Cargo,
    Shell(String),
    Plugin(String),
}

impl Installer {
    fn install(&self) {
        todo!()
    }
}
