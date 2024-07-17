use std::str::FromStr;

use anyhow::{Context, Result};
use target_lexicon::Triple;

use super::{phase::Phase, plugin::Plugin, profiles::Profiles};

#[derive(Debug)]
enum ConfigKeyPart {
    Assembly,
    Target,
    Profile,
}

fn validate_key_part(part: &str, expected: &ConfigKeyPart, profiles: &Profiles) -> bool {
    match expected {
        ConfigKeyPart::Target => Triple::from_str(part).is_ok(),
        ConfigKeyPart::Profile => profiles.contains(part),
        ConfigKeyPart::Assembly => true,
    }
}

pub(super) struct ConfigKey {
    pub phase: Phase,
    pub assembly: Option<String>,
    pub target: Option<String>,
    pub profile: Option<String>,
    pub plugin: String,
    pub action: Option<String>,
}

impl ConfigKey {
    pub fn try_from(str: &str, plugins: &[Plugin], profiles: &Profiles) -> Result<Self> {
        let parts: Vec<&str> = str.split('.').collect();

        if parts.len() > 1 {
            if let Ok(phase) = Phase::from_str(parts[0]) {
                let mut assembly = None;
                let mut target = None;
                let mut profile = None;

                let is_plugin = |s: &str| -> bool { plugins.iter().any(|p| p.name == s) };

                let plugin_pos = parts
                    .iter()
                    .position(|p| is_plugin(p))
                    .context(format!("No plugin set in [{str}]"))?;
                let plugin = parts[plugin_pos].to_string();

                let action = parts.get(plugin_pos + 1).map(|s| s.to_string());

                let expected_parts = vec![
                    ConfigKeyPart::Profile,
                    ConfigKeyPart::Target,
                    ConfigKeyPart::Assembly,
                ];
                let mut expected_idx = 0_usize;

                let mut part_idx = plugin_pos as isize - 1;

                loop {
                    if part_idx < 1 {
                        break;
                    }
                    let part = parts[part_idx as usize];

                    loop {
                        if expected_idx >= expected_parts.len() {
                            break;
                        }

                        if validate_key_part(part, &expected_parts[expected_idx], profiles) {
                            match expected_parts[expected_idx] {
                                ConfigKeyPart::Assembly => {
                                    assembly = Some(part.to_string());
                                }
                                ConfigKeyPart::Target => {
                                    target = Some(part.to_string());
                                }
                                ConfigKeyPart::Profile => {
                                    profile = Some(part.to_string());
                                }
                            }
                            expected_idx += 1;
                            break;
                        } else {
                            expected_idx += 1;
                        }
                    }

                    part_idx -= 1;
                }

                Ok(Self {
                    phase,
                    assembly,
                    target,
                    profile,
                    plugin,
                    action,
                })
            } else {
                anyhow::bail!("Invalid config phase: '{str}'");
            }
        } else {
            anyhow::bail!("Invalid config key: '{str}'");
        }
    }
}

pub struct InstallKey {
    pub target: Option<String>,
    pub plugin: String,
}

impl InstallKey {
    pub fn try_from(str: &str) -> Result<Self> {
        let parts: Vec<&str> = str.split('.').collect();

        if parts.len() == 2 {
            if parts[0] == "install" {
                Ok(Self {
                    target: None,
                    plugin: parts[1].to_owned(),
                })
            } else {
                anyhow::bail!("Invalid install key: '{str}'");
            }
        } else if parts.len() == 3 {
            if parts[0] == "install" {
                if Triple::from_str(parts[1]).is_ok() {
                    Ok(Self {
                        target: Some(parts[1].to_owned()),
                        plugin: parts[2].to_owned(),
                    })
                } else {
                    anyhow::bail!("Invalid host triple: '{}'", parts[1])
                }
            } else {
                anyhow::bail!("Invalid install key: '{str}'");
            }
        } else {
            anyhow::bail!("Can't determine install key from: '{str}'");
        }
    }
}
