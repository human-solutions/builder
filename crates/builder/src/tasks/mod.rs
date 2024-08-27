mod file;
mod fontforge;
mod localized;
mod postbuild;
mod prebuild;
mod sass;
mod setup;
mod wasm;

use std::{collections::HashSet, fmt::Display, str::FromStr};

use anyhow::{Context, Result};
use fontforge::FontForgeParams;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use wasm::WasmParams;

pub use file::FilesParams;
pub use localized::LocalizedParams;
pub use sass::SassParams;
pub use setup::{BuildStep, Config, Setup};

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
                let params: FontForgeParams = serde_json::from_value(value.clone())
                    .context(format!("Failed to parse font-forge metadata: '{value}'"))?;
                Tool::FontForge(params)
            }
            Tool::WasmBindgen(_) => {
                let params: WasmParams = serde_json::from_value(value.clone())
                    .context(format!("Failed to parse wasm-bindgen metadata: '{value}'"))?;
                Tool::WasmBindgen(params)
            }
            Tool::Sass(_) => {
                let params: SassParams = serde_json::from_value(value.clone())
                    .context(format!("Failed to parse sass metadata: '{value}'"))?;
                Tool::Sass(params)
            }
            Tool::Localized(_) => {
                let params: LocalizedParams = serde_json::from_value(value.clone()).context(
                    format!("Failed to parse localized assets metadata: '{value}'"),
                )?;
                Tool::Localized(params)
            }
            Tool::Files(_) => {
                let params: FilesParams = serde_json::from_value(value.clone())
                    .context(format!("Failed to parse file metadata: '{value}'"))?;
                Tool::Files(params)
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
                Tool::Files(file) => file.process(config, generator, watched)?,
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
    Files(FilesParams),
    Uniffi,
}

impl Display for Tool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Tool::FontForge(_) => write!(f, "font-forge"),
            Tool::WasmBindgen(_) => write!(f, "wasm-bindgen"),
            Tool::Sass(_) => write!(f, "sass"),
            Tool::Localized(_) => write!(f, "localized"),
            Tool::Files(_) => write!(f, "files"),
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
            "sass" => Ok(Self::Sass(SassParams::default())),
            "localized" => Ok(Self::Localized(LocalizedParams::default())),
            "files" => Ok(Self::Files(FilesParams::default())),
            "uniffi" => Ok(Self::Uniffi),
            _ => anyhow::bail!("Invalid tool: {}", s),
        }
    }
}
