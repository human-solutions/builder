use serde_json::Value;

use crate::{anyhow::Result, Config};

use super::assembly::Assembly;

#[derive(Debug)]
pub struct PostbuildConfig {
    assemblies: Vec<Assembly>,
}

impl PostbuildConfig {
    pub fn from_json(json: &Value) -> Result<Self> {
        let mut assemblies = Vec::new();
        for (assembly_name, assembly_val) in json.as_object().unwrap() {
            if let Some(assembly_obj) = assembly_val.as_object() {
                for (key, val) in assembly_obj.iter() {
                    let mut assembly: Assembly = serde_json::from_value(val.clone()).unwrap();
                    assembly.name.clone_from(assembly_name);
                    assembly.profile.clone_from(key);
                    assemblies.push(assembly);
                }
            } else {
                panic!("invalid assembly")
            }
        }
        Ok(Self { assemblies })
    }

    pub fn process(&self, info: &Config) -> Result<()> {
        for assembly in &self.assemblies {
            if assembly.profile == info.args.profile {
                assembly.process(info)?;
            }
        }
        Ok(())
    }
}
