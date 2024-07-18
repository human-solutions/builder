mod stage;

use std::str::FromStr;

use anyhow::{Context, Result};
use serde::Serialize;
use stage::HookStage;

use super::{plugin::Plugin, ValueWrapper};

#[derive(Serialize)]
pub struct GitHook {
    stage: HookStage,
    plugins: Vec<String>,
}

impl GitHook {
    pub fn new(stage_str: &str, value: ValueWrapper, plugins: &[Plugin]) -> Result<Self> {
        let stage = HookStage::from_str(stage_str).context("failed to create githook")?;

        if let ValueWrapper::Single(val) = value {
            let found_plugins = plugins;
            let mut plugins = Vec::new();

            for (key, _) in val
                .as_object()
                .context("failed to retrieve githook plugin data")?
            {
                if found_plugins.iter().any(|p| p.name == *key) {
                    plugins.push(key.to_string());
                } else {
                    anyhow::bail!("plugin {} not found", key);
                }
            }

            Ok(Self { stage, plugins })
        } else {
            anyhow::bail!("expected githook to be a table, found an array of table")
        }
    }
}
