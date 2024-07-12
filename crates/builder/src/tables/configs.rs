use std::{collections::HashMap, str::FromStr};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Default, Hash, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum Phase {
    #[default]
    Prebuild,
    Postbuild,
}

impl FromStr for Phase {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "prebuild" => Ok(Self::Prebuild),
            "postbuild" => Ok(Self::Postbuild),
            _ => anyhow::bail!("Invalid phase: {}", s),
        }
    }
}

#[derive(Default, Hash, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct Configuration {
    phase: Phase,
    assembly: Option<String>,
    target_triple: Option<String>,
    profile: Option<String>,
    name: String,
    action: Option<String>,
}

#[derive(Default, Serialize, Deserialize)]
pub struct Configs(pub HashMap<Configuration, Value>);

impl Configs {
    pub fn from_iter<'a, I>(phases_iter: I) -> Result<Self>
    where
        I: Iterator<Item = (&'a String, &'a Value)>,
    {
        let mut cfg = HashMap::new();

        for (phase, val) in phases_iter {
            configs(phase, val, &mut cfg)?;
        }

        Ok(Self(cfg))
    }

    pub fn insert(&mut self, phase: &str, obj: &Value) -> Result<()> {
        configs(phase, obj, &mut self.0)
    }

    pub fn extend(&mut self, cfg: Self) {
        self.0.extend(cfg.0)
    }
}

fn configs(phase: &str, obj: &Value, cfg: &mut HashMap<Configuration, Value>) -> Result<()> {
    for (assembly, val) in obj
        .as_object()
        .context("Failed to retrieve assembly object")?
    {
        for (target, val) in val
            .as_object()
            .context("Failed to retrieve target object")?
        {
            for (profile, val) in val
                .as_object()
                .context("Failed to retrieve profile object")?
            {
                for (name, val) in val.as_object().context("Failed to retrieve name object")? {
                    for (action, val) in val
                        .as_object()
                        .context("Failed to retrieve action object")?
                    {
                        let assembly = (assembly != "*").then_some(assembly.to_owned());
                        let target = (target != "*").then_some(target.to_owned());
                        let profile = (profile != "*").then_some(profile.to_owned());
                        let action = (action != "*").then_some(action.to_owned());

                        cfg.insert(
                            Configuration {
                                phase: phase.parse()?,
                                assembly,
                                target_triple: target,
                                profile,
                                name: name.to_owned(),
                                action,
                            },
                            val.clone(),
                        );
                    }
                }
            }
        }
    }

    Ok(())
}
