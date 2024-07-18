use std::{fs::File, io::Write};

use anyhow::{Context, Result};
use serde::Serialize;

use crate::{parser, BuilderArgs};

use super::{
    envs::Envs,
    githook::GitHook,
    output::Outputs,
    plugin::{Plugin, Setup},
    profiles::Profiles,
    table::{ConfigKey, InstallKey, TableEntry},
};

#[derive(Serialize)]
pub struct Input {
    envs: Envs,
    plugins: Vec<Plugin>,
    githooks: Vec<GitHook>,
    outputs: Outputs,
}

impl Input {
    pub fn gather(args: BuilderArgs) -> Result<Self> {
        let envs = Envs::gather();
        let profiles = Profiles::gather(&args.dir)?;

        let tables = parser::parse(format!("{}/Builder.toml", args.dir))?;

        let mut installs = Vec::new();
        let mut hook_tables = Vec::new();
        let mut configs = Vec::new();

        for table in tables.into_iter() {
            if table.key.starts_with("install") {
                installs.push(table);
            } else if table.key.starts_with("prebuild") || table.key.starts_with("postbuild") {
                configs.push(table);
            } else if table.key.starts_with("githook") {
                hook_tables.push(table)
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
                .with_target(key.target)
                .context(format!("{}: ", table.key))?;

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

            plugins[plugin_idx]
                .push_action(&phase, action, assembly, target, profile, value)
                .context(format!("Failed to add action to plugin '{plugin}'",))?;
        }

        let mut githooks = Vec::new();

        for TableEntry { key, value } in hook_tables.into_iter() {
            let (_, hook_phase) = key.split_once('.').context(format!(
                "Failed to retrieve githook stage from table key: {}",
                key
            ))?;

            let githook = GitHook::new(hook_phase, value, &plugins)
                .context("failed to process githook table entry")?;

            githooks.push(githook);
        }

        let outputs = Outputs::gather(&args.dir)?;

        Ok(Self {
            envs,
            plugins,
            githooks,
            outputs,
        })
    }

    pub fn save_file(&self) -> Result<()> {
        let yaml_string = serde_yaml::to_string(&self)?;

        let mut file = File::create("input.yaml")?;
        file.write_all(yaml_string.as_bytes())?;

        Ok(())
    }

    pub fn check_plugins(&self) -> Result<()> {
        let mut bins = Vec::new();

        for plugin in &self.plugins {
            bins.extend(plugin.check()?);
        }

        for bin in bins {
            println!("{bin}");
        }

        Ok(())
    }
}
