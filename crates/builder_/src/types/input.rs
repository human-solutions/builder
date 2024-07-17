use std::{fs::File, io::Write};

use anyhow::{Context, Result};
use cargo_metadata::MetadataCommand;
use serde::Serialize;

use crate::{parser, types::plugin::Spec, BuilderArgs};

use super::{
    envs::Envs,
    plugin::{Plugin, Setup},
    profiles::Profiles,
    table_keys::{ConfigKey, InstallKey},
    tables::TableEntry,
};

#[derive(Serialize)]
pub struct Input {
    envs: Envs,
    plugins: Vec<Plugin>,
    // githooks: Vec<GitHook>,
}

impl Input {
    pub fn gather(args: BuilderArgs) -> Result<Self> {
        let metadata = MetadataCommand::new()
            .manifest_path(args.dir.join("Cargo.toml"))
            .exec()?;
        let package = metadata.root_package().context("root package not found")?;

        let envs = Envs::gather();
        let profiles = Profiles::gather(&args.dir)?;

        let tables = parser::parse(format!("{}/Builder.toml", args.dir))?;

        let mut installs = Vec::new();
        let mut githooks = Vec::new();
        let mut configs = Vec::new();

        for table in tables.into_iter() {
            if table.key.starts_with("install") {
                installs.push(table);
            } else if table.key.starts_with("prebuild") || table.key.starts_with("postbuild") {
                configs.push(table);
            } else if table.key.starts_with("githook") {
                githooks.push(table)
            } else {
                // unknown key
                // NOTE: ignore for now
            }
        }

        let mut plugins = Vec::new();

        for table in installs {
            let key = InstallKey::try_from(&table.key)?;

            let setup = Setup::try_from_value(&table.value)
                .context(format!("{}: ", table.key))?
                .with_target(key.target);

            if let Some(pos) = plugins.iter().position(|p: &Plugin| p.name == key.plugin) {
                plugins[pos].setup.push(setup);
            } else {
                let mut plugin = Plugin::default();
                plugin.name = key.plugin;
                plugin.setup = vec![setup];

                plugins.push(plugin);
            }
        }

        for TableEntry { key, value } in configs.into_iter() {
            let ConfigKey {
                phase,
                assembly,
                target,
                profile,
                plugin,
                action,
            } = ConfigKey::try_from(&key, &plugins, &profiles)?;

            let plugin_idx = plugins
                .iter()
                .position(|p| p.name == plugin)
                .context(format!(
                    "Install configuration for plugin {} is not set",
                    plugin
                ))?;

            let spec = Spec::new(assembly, target, profile, value)
                .context(format!("Failed to create spec for plugin {}", plugin))?;

            plugins[plugin_idx]
                .push_action(&phase, action, spec)
                .context(format!("Failed to add action to plugin '{plugin}'",))?;
        }

        Ok(Self {
            envs,
            plugins,
            // githooks,
        })
    }

    pub fn save_file(&self) -> Result<()> {
        let yaml_string = serde_yaml::to_string(&self)?;

        let mut file = File::create("input.yaml")?;
        file.write_all(yaml_string.as_bytes())?;

        Ok(())
    }
}
