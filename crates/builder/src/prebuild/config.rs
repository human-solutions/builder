use std::collections::HashSet;

use anyhow::Context;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{anyhow::Result, Config};

use crate::generate::Generator;

use super::{fontforge::FontForge, Assembly};

#[derive(Debug, Serialize, Deserialize)]
pub struct PrebuildConfig {
    pub assemblies: Vec<Assembly>,
    pub fontforge: Option<FontForge>,
}

impl PrebuildConfig {
    pub fn from_json(json: &Value) -> Result<Self> {
        let mut assemblies = Vec::new();
        let mut fontforge = None;
        for (target, target_val) in json.as_object().context("Invalid assembly")? {
            for (assembly_name, assembly_val) in target_val
                .as_object()
                .context(format!("Failed to access assembly for target {target}",))?
            {
                if assembly_name == "fontforge" {
                    fontforge = Some(serde_json::from_value(assembly_val.clone())?);
                } else {
                    for (key, val) in assembly_val.as_object().context(format!(
                        "Failed to access profile for {target}.{assembly_name}",
                    ))? {
                        let mut assembly: Assembly = serde_json::from_value(val.clone()).unwrap();
                        assembly.name = if assembly_name == "*" {
                            None
                        } else {
                            Some(assembly_name.to_owned())
                        };
                        assembly.target.clone_from(target);
                        assembly.profile.clone_from(key);
                        assemblies.push(assembly);
                    }
                }
            }
        }
        Ok(Self {
            assemblies,
            fontforge,
        })
    }
}

impl PrebuildConfig {
    pub fn process(&self, conf: &Config) -> Result<()> {
        let mut watched = HashSet::new();
        watched.insert("Cargo.toml".to_string());
        watched.insert("src".to_string());

        let mut assembly_names = HashSet::new();

        if let Some(ff) = self.fontforge.as_ref() {
            ff.process(conf)?;
            watched.insert(ff.file.to_string());
        }
        let mut generator = Generator::default();

        log::trace!("Processing {:#?}", self.assemblies);

        // go through all named assemblies
        for assembly in &self.assemblies {
            let Some(name) = assembly.name.as_ref() else {
                continue;
            };
            assembly_names.insert(name.to_string());
            if conf.args.profile == assembly.profile
                && (assembly.target == conf.args.target || assembly.target == "*")
            {
                let change = assembly.process(conf, &mut generator, name, true)?;
                watched.extend(change.into_iter());
            }
        }

        // go through wildcard assemblies
        for assembly in self.assemblies.iter().filter(|a| a.name.is_none()) {
            for name in &assembly_names {
                if conf.args.profile == assembly.profile
                    && (assembly.target == conf.args.target || assembly.target == "*")
                {
                    let change = assembly.process(conf, &mut generator, name, false)?;
                    watched.extend(change.into_iter());
                }
            }
        }
        generator.write(conf)?;
        for change in watched {
            println!("cargo::rerun-if-changed={}", change);
        }
        Ok(())
    }
}
