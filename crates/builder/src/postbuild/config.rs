use std::{
    fs::{self, File},
    io::{Read, Write},
};

use anyhow::Context;
use camino::Utf8PathBuf;
use fs4::fs_std::FileExt;
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

    pub fn save(&self, path: &Utf8PathBuf) -> Result<()> {
        let string = serde_yaml::to_string(self)?;

        let mut file = File::create(path)?;
        file.write_all(string.as_bytes())?;

        Ok(())
    }

    pub fn load(path: &Utf8PathBuf) -> Result<Self> {
        let mut file = File::open(path)?;
        file.try_lock_exclusive()?;

        let mut string = String::new();
        file.read_to_string(&mut string)?;

        let postbuild: Self = serde_yaml::from_str(&string)
            .context(format!("Failed to parse output file '{}'", path))?;

        fs::remove_file(path)?;

        Ok(postbuild)
    }
}
