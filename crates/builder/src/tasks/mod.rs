mod setup;
mod wasm;

use std::{fmt::Display, str::FromStr};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use setup::Config;
use wasm::WasmParams;

use crate::ext::value::IntoVecString;

#[derive(Serialize, Deserialize)]
struct Task {
    pub tool: Tool,
    pub targets: Vec<String>,
    pub profiles: Vec<String>,
}

impl Task {
    fn from_value(key: &str, value: &Value) -> Result<Self> {
        let tool = Tool::from_str(key).context(format!("Invalid tool '{key}'"))?;

        let targets = value.into_vec_string("target");
        let profiles = value.into_vec_string("profile");

        match tool {
            Tool::FontForge => todo!(),
            Tool::WasmBindgen(_) => {
                let params: WasmParams = serde_json::from_value(value.clone()).context(format!(
                    "Failed to parse wasm-bindgen params for task '{key}'"
                ))?;
                Ok(Task {
                    tool,
                    targets,
                    profiles,
                })
            }
            Tool::Uniffi => todo!(),
        }
    }

    fn run(&self, config: &Config) -> Result<()> {
        let target = &config.args.target;
        let profile = &config.args.profile;
        if (self.targets.is_empty() || self.targets.contains(target))
            && (self.profiles.is_empty() || self.profiles.contains(profile))
        {
            log::info!(
                "Running task for {} with target {} and profile {}",
                self.tool,
                target,
                profile
            );
            match &self.tool {
                Tool::FontForge => todo!(),
                Tool::WasmBindgen(wasm) => wasm.process(config)?,
                Tool::Uniffi => todo!(),
            }
        } else {
            log::info!("Skipping task for {}", self.tool);
        }

        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
enum Tool {
    FontForge,
    WasmBindgen(WasmParams),
    Uniffi,
}

impl Display for Tool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Tool::FontForge => write!(f, "font-forge"),
            Tool::WasmBindgen(_) => write!(f, "wasm-bindgen"),
            Tool::Uniffi => write!(f, "uniffi"),
        }
    }
}

impl FromStr for Tool {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "font-forge" => Ok(Self::FontForge),
            "wasm-bindgen" => Ok(Self::WasmBindgen(WasmParams::default())),
            "uniffi" => Ok(Self::Uniffi),
            _ => anyhow::bail!("Invalid tool: {}", s),
        }
    }
}

#[derive(Default, Serialize, Deserialize)]
struct Tasks(Vec<Task>);

impl Tasks {
    fn from_value(value: &Value) -> Result<Self> {
        let mut tasks = Vec::new();

        for (tool, tool_val) in value.as_object().context("Invalid builder metadata")? {
            for _ in tool_val
                .as_array()
                .context(format!("Invalid tasks for tool '{tool}'"))?
            {
                tasks.push(Task::from_value(tool, tool_val)?);
            }
        }

        Ok(Self(tasks))
    }
    fn run(&self, config: &Config) -> Result<()> {
        for task in &self.0 {
            task.run(config)?;
        }

        Ok(())
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
