use fs_err as fs;
use serde_json::Value;
// use serde_json::Value;
use toml_edit::DocumentMut;

use crate::anyhow::{Context, Result};

use super::{assembly::Assembly, PostbuildArgs};

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

    pub fn try_parse(info: &PostbuildArgs) -> Result<Self> {
        let manifest_str = fs::read_to_string(info.manifest_dir.join("Cargo.toml"))?;
        let manifest = manifest_str.parse::<DocumentMut>()?;
        let val = &manifest
            .get("package")
            .context("Could not find package section in manifest")?
            .get("metadata")
            .context("Could not find package.metadata section in manifest")?
            .get("postbuild")
            .context("Could not find package.metadata.postbuild section in manifest")?;

        let names = val.as_table().context(
            "Could not find assembly name. Expected package.metadata.postbuild.<assembly>",
        )?;

        let mut assemblies = Vec::new();
        for (name, value) in names {
            for (profile, toml) in value.as_table().unwrap() {
                let ass = Assembly::try_parse(name, profile, toml)?;
                assemblies.push(ass)
            }
        }
        Ok(Self { assemblies })
    }

    // pub fn try_parse_metadata(value: &Value) {
    //     // serde_json::from_value(value)
    // }
    pub fn process(&self, info: &PostbuildArgs) -> Result<()> {
        for assembly in &self.assemblies {
            if assembly.profile == info.profile {
                assembly.process(info)?;
            }
        }
        Ok(())
    }
}
