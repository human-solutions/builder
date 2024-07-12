use std::{
    fs::File,
    io::{Read, Write},
    str::FromStr,
};

use anyhow::{Context, Result};
use cargo_metadata::{camino::Utf8PathBuf, MetadataCommand};
use serde::{Deserialize, Serialize};

use crate::{
    envs::Envs,
    output::Output,
    tables::{Binaries, Configs, Phase, Tables},
};

#[derive(Serialize, Deserialize)]
pub struct Input {
    envs: Envs,
    pub configs: Configs,
    pub binaries: Binaries,
    dependencies: Vec<Output>,
}

impl Input {
    pub fn save_file(&self) -> Result<()> {
        let yaml_string = serde_yaml::to_string(&self)?;

        let mut file = File::create("input.yaml")?;
        file.write_all(yaml_string.as_bytes())?;

        Ok(())
    }

    pub fn gather_all(dir: &Utf8PathBuf) -> Result<Self> {
        let metadata = MetadataCommand::new()
            .manifest_path(dir.join("Cargo.toml"))
            .exec()?;
        let package = metadata.root_package().context("root package not found")?;

        let obj_data = package
            .metadata
            .as_object()
            .context("Failed to retrieve metadata object")?;

        let phases_iter = obj_data
            .iter()
            .filter(|(key, _)| Phase::from_str(key).is_ok());
        let bins_iter = obj_data.iter().filter(|(key, _)| key == &"install");

        let mut phases_cfg = Configs::from_iter(phases_iter)?;
        let mut bins_cfg = Binaries::from_iter(bins_iter)?;

        let mut file = File::open(dir.join("Builder.toml"))?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let builder_tables: Tables = toml::from_str(&contents)?;

        phases_cfg.extend(builder_tables.configs);
        bins_cfg.extend(builder_tables.binaries);

        Ok(Input {
            envs: Envs::gather(),
            configs: phases_cfg,
            binaries: bins_cfg,
            dependencies: Vec::new(),
        })
    }
}
