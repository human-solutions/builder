use fs_err as fs;
use toml_edit::DocumentMut;

use crate::anyhow::{Context, Result};

use super::{assembly::Assembly, PostbuildArgs};

#[derive(Debug)]
pub struct PostbuildConfig {
    assemblies: Vec<Assembly>,
}

impl PostbuildConfig {
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

    pub fn process(&self, info: &PostbuildArgs) -> Result<()> {
        for assembly in &self.assemblies {
            if assembly.profile == info.profile {
                assembly.process(info)?;
            }
        }
        Ok(())
    }
}
