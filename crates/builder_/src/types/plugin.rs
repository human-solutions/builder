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
    pub fn push_action(
        &mut self,
        phase: &Phase,
        action: Option<String>,
        assembly: Option<String>,
        target: Option<String>,
        profile: Option<String>,
        output: ValueWrapper,
    ) -> Result<()> {
        if phase.is_pre_build() {
            if let Some(pos) = self.has_prebuild_action(&action) {
                if let Some(a_pos) = self.prebuild[pos].has_assembly(&assembly) {
                    if let Some(t_pos) = self.prebuild[pos].assemblies[a_pos].has_target(&target) {
                        //
                        if self.prebuild[pos].assemblies[a_pos].targets[t_pos]
                            .has_profile(&profile)
                            .is_some()
                        {
                            anyhow::bail!(format!(
                                "Profile '{}' already exists for plugin '{}':'{}'.'{}'",
                                profile.unwrap_or_default(),
                                self.name,
                                assembly.unwrap_or_default(),
                                target.unwrap_or_default(),
                            ));
                        } else {
                            self.prebuild[pos].assemblies[a_pos].targets[t_pos]
                                .push(profile, output)
                                .context(format!(
                                    "Plugin '{}':'{}'.'{}'",
                                    self.name,
                                    assembly.unwrap_or_default(),
                                    target.unwrap_or_default(),
                                ))?;
                        }
                    } else {
                        self.prebuild[pos].assemblies[a_pos]
                            .push(target, profile, output)
                            .context(format!(
                                "Failed to add target to plugin '{}':'{}'",
                                self.name,
                                assembly.unwrap_or_default(),
                            ))?;
                    }
                } else {
                    self.prebuild[pos]
                        .push(assembly, target, profile, output)
                        .context(format!("Failed to add assembly to plugin '{}'", self.name))?;
                }
            } else {
                let action = Action::new(action, assembly, target, profile, output)
                    .context(format!("Failed to add action to plugin '{}'", self.name))?;

                self.prebuild.push(action);
            }
        } else if phase.is_postbuild() {
            if let Some(pos) = self.has_postbuild_action(&action) {
                if let Some(a_pos) = self.postbuild[pos].has_assembly(&assembly) {
                    if let Some(t_pos) = self.postbuild[pos].assemblies[a_pos].has_target(&target) {
                        if self.postbuild[pos].assemblies[a_pos].targets[t_pos]
                            .has_profile(&profile)
                            .is_some()
                        {
                            anyhow::bail!(format!(
                                "Profile '{}' already exists for plugin '{}':'{}'.'{}'",
                                profile.unwrap_or_default(),
                                self.name,
                                assembly.unwrap_or_default(),
                                target.unwrap_or_default(),
                            ));
                        } else {
                            self.postbuild[pos].assemblies[a_pos].targets[t_pos]
                                .push(profile, output)
                                .context(format!(
                                    "Plugin '{}':'{}'.'{}'",
                                    self.name,
                                    assembly.unwrap_or_default(),
                                    target.unwrap_or_default(),
                                ))?;
                        }
                    } else {
                        self.postbuild[pos].assemblies[a_pos]
                            .push(target, profile, output)
                            .context(format!(
                                "Failed to add target to plugin '{}':'{}'",
                                self.name,
                                assembly.unwrap_or_default(),
                            ))?;
                    }
                } else {
                    self.postbuild[pos]
                        .push(assembly, target, profile, output)
                        .context(format!("Failed to add assembly to plugin '{}'", self.name))?;
                }
            } else {
                let action = Action::new(action, assembly, target, profile, output)
                    .context(format!("Failed to add action  to plugin '{}'", self.name))?;

                self.postbuild.push(action);
            }
        }

        Ok(())
    }

    fn has_prebuild_action(&self, action_name: &Option<String>) -> Option<usize> {
        self.prebuild.iter().position(|a| a.name == *action_name)
    }

    fn has_postbuild_action(&self, action_name: &Option<String>) -> Option<usize> {
        self.postbuild.iter().position(|a| a.name == *action_name)
    }
}

#[derive(Serialize)]
pub struct Action {
    #[serde(rename = "action")]
    name: Option<String>,
    assemblies: Vec<Assembly>,
}

impl Action {
    fn has_assembly(&self, assembly_name: &Option<String>) -> Option<usize> {
        self.assemblies
            .iter()
            .position(|a| a.name == *assembly_name)
    }

    fn push(
        &mut self,
        assembly: Option<String>,
        target: Option<String>,
        profile: Option<String>,
        output: ValueWrapper,
    ) -> Result<()> {
        let assembly = Assembly::new(assembly, target, profile, output).context(format!(
            "action-add-assembly:'{}'",
            self.name.as_ref().unwrap_or(&"".to_string())
        ))?;

        self.assemblies.push(assembly);

        Ok(())
    }

    fn new(
        name: Option<String>,
        assembly: Option<String>,
        target: Option<String>,
        profile: Option<String>,
        output: ValueWrapper,
    ) -> Result<Self> {
        let assembly = Assembly::new(assembly, target, profile, output).context(format!(
            "create_action:'{}'",
            name.as_ref().unwrap_or(&"".to_string())
        ))?;

        Ok(Self {
            name,
            assemblies: vec![assembly],
        })
    }
}

#[derive(Default, Serialize)]
pub struct Assembly {
    #[serde(rename = "assembly")]
    name: Option<String>,
    targets: Vec<Target>,
}

impl Assembly {
    fn has_target(&self, target_name: &Option<String>) -> Option<usize> {
        self.targets.iter().position(|t| t.name == *target_name)
    }

    fn push(
        &mut self,
        target: Option<String>,
        profile: Option<String>,
        output: ValueWrapper,
    ) -> Result<()> {
        let target = Target::new(target, profile, output).context(format!(
            "assembly-add-target:'{}'",
            self.name.as_ref().unwrap_or(&"".to_string())
        ))?;

        self.targets.push(target);

        Ok(())
    }

    fn new(
        name: Option<String>,
        target: Option<String>,
        profile: Option<String>,
        output: ValueWrapper,
    ) -> Result<Self> {
        let target = Target::new(target, profile, output).context(format!(
            "create-assembly:'{}'",
            name.as_ref().unwrap_or(&"".to_string())
        ))?;

        Ok(Self {
            name,
            targets: vec![target],
        })
    }
}

#[derive(Default, Serialize)]
struct Target {
    #[serde(rename = "triple")]
    name: Option<String>,
    profiles: Vec<Profile>,
}

impl Target {
    fn has_profile(&self, profile_name: &Option<String>) -> Option<usize> {
        self.profiles.iter().position(|p| p.name == *profile_name)
    }

    fn push(&mut self, profile: Option<String>, output: ValueWrapper) -> Result<()> {
        let profile = Profile::new(profile, output).context(format!(
            "target-add-profile:'{}'",
            self.name.as_ref().unwrap_or(&"".to_string())
        ))?;

        self.profiles.push(profile);

        Ok(())
    }

    fn new(name: Option<String>, profile: Option<String>, output: ValueWrapper) -> Result<Self> {
        let profile = Profile::new(profile, output).context(format!(
            "create-target:'{}'",
            name.as_ref().unwrap_or(&"".to_string())
        ))?;

        Ok(Self {
            name,
            profiles: vec![profile],
        })
    }
}

#[derive(Default, Serialize)]
struct Profile {
    #[serde(rename = "profile")]
    name: Option<String>,
    output: Value,
}

impl Profile {
    fn new(name: Option<String>, output: ValueWrapper) -> Result<Self> {
        if let ValueWrapper::Single(output) = output {
            Ok(Self { name, output })
        } else {
            Err(anyhow::Error::msg(
                "Expected output data from table but output data defined as table array",
            ))
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
