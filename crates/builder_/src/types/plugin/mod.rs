mod action;
mod assembly;
mod installer;
mod profile;
mod setup;
mod target;

use action::Action;
use anyhow::{Context, Result};
use serde::Serialize;
pub use setup::Setup;

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
