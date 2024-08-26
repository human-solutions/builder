mod fontforge;
mod localized;
mod sass;
mod setup;
mod wasm;

use std::{collections::HashSet, fmt::Display, str::FromStr};

use anyhow::{Context, Result};
use fontforge::FontForgeParams;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use wasm::WasmParams;

pub use localized::LocalizedParams;
pub use sass::SassParams;
pub use setup::Config;

use crate::{ext::value::IntoVecString, generate::Generator};

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

        let tool = match tool {
            Tool::FontForge(_) => {
                let params: FontForgeParams = serde_json::from_value(value.clone()).context(
                    format!("Failed to parse font-forge params for task '{key}'"),
                )?;
                Tool::FontForge(params)
            }
            Tool::WasmBindgen(_) => {
                let params: WasmParams = serde_json::from_value(value.clone()).context(format!(
                    "Failed to parse wasm-bindgen params for task '{key}'"
                ))?;
                Tool::WasmBindgen(params)
            }
            Tool::Sass(_) => {
                let params: SassParams = serde_json::from_value(value.clone())
                    .context(format!("Failed to parse sass params for task '{key}'"))?;
                Tool::Sass(params)
            }
            Tool::Localized(_) => {
                let params: LocalizedParams = serde_json::from_value(value.clone())
                    .context(format!("Failed to parse localized params for task '{key}'"))?;
                Tool::Localized(params)
            }
            Tool::Uniffi => todo!(),
        };

        Ok(Task {
            tool,
            targets,
            profiles,
        })
    }

    fn run(
        &self,
        config: &Config,
        generator: &mut Generator,
        watched: &mut HashSet<String>,
    ) -> Result<()> {
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
                Tool::FontForge(fontforge) => fontforge.process(config)?,
                Tool::WasmBindgen(wasm) => wasm.process(config)?,
                Tool::Sass(sass) => sass.process(config, generator, watched)?,
                Tool::Localized(localized) => localized.process(config, generator, watched)?,
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
    FontForge(FontForgeParams),
    WasmBindgen(WasmParams),
    Sass(SassParams),
    Localized(LocalizedParams),
    Uniffi,
}

impl Display for Tool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Tool::FontForge(_) => write!(f, "font-forge"),
            Tool::WasmBindgen(_) => write!(f, "wasm-bindgen"),
            Tool::Sass(_) => write!(f, "sass"),
            Tool::Localized(_) => write!(f, "localized"),
            Tool::Uniffi => write!(f, "uniffi"),
        }
    }
}

impl FromStr for Tool {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "font-forge" => Ok(Self::FontForge(FontForgeParams::default())),
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
        let mut generator = Generator::default();
        let mut watched = HashSet::new();
        watched.insert("Cargo.toml".to_string());
        watched.insert("src".to_string());

        for task in &self.0 {
            task.run(config, &mut generator, &mut watched)?;
        }

        generator.write(config)?;

        Ok(())
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
