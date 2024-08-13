use anyhow::Context;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{anyhow::Result, Config};

use super::assembly::Assembly;

#[derive(Debug, Serialize, Deserialize)]
pub struct PostbuildConfig {
    assemblies: Vec<Assembly>,
}

impl PostbuildConfig {
    pub fn from_json(json: &Value) -> Result<Self> {
        let mut assemblies = Vec::new();
        for (target, target_val) in json.as_object().context("Invalid assembly")? {
            for (assembly_name, assembly_val) in target_val
                .as_object()
                .context(format!("Failed to access assembly for target {target}",))?
            {
                for (key, val) in assembly_val.as_object().context(format!(
                    "Failed to access profile for assembly {assembly_name}",
                ))? {
                    let mut assembly: Assembly = serde_json::from_value(val.clone()).unwrap();
                    assembly.name.clone_from(assembly_name);
                    assembly.profile.clone_from(key);
                    assembly.target.clone_from(target);
                    assemblies.push(assembly);
                }
            }
        }
        Ok(Self { assemblies })
    }

    pub fn process(&self, info: &Config) -> Result<()> {
        for assembly in &self.assemblies {
            if assembly.profile == info.args.profile
                && (assembly.target == "*" || assembly.target == info.args.target)
            {
                assembly.process(info)?;
            }
        }
        Ok(())
    }
}
