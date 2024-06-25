use fs_err as fs;
use toml_edit::DocumentMut;

use anyhow::{Context, Result};

use super::PostbuildArgs;

#[derive(Debug)]
pub struct PostbuildManifest {}

impl PostbuildManifest {
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

        // let mut assemblies = Vec::new();
        // let mut fontforge = None;
        for (_name, _value) in names {}
        Ok(Self {})
    }
}
