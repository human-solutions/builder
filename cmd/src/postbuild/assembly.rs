use crate::anyhow::{bail, Context, Result};
use crate::util::parse_vec;
use crate::PostbuildArgs;
use serde::Deserialize;
use toml_edit::Item;

use super::wasm::WasmBindgen;

#[derive(Debug, Default, Deserialize)]
pub struct Assembly {
    #[serde(skip)]
    pub name: String,
    #[serde(skip)]
    pub profile: String,
    #[serde(rename = "wasmbindgen")]
    pub wasm: Vec<WasmBindgen>,
}

impl Assembly {
    pub fn try_parse(name: &str, profile: &str, toml: &Item) -> Result<Self> {
        let name = name.to_string();

        let profile = profile.to_string();
        let table = toml.as_table().context("no content")?;

        let mut wasm = Vec::new();

        for (process, toml) in table {
            match process {
                "wasmbindgen" => {
                    wasm = parse_vec(toml, WasmBindgen::try_parse)
                        .context("Could not parse sass values")?;
                }
                _ => bail!("Invalid processing type: {process}"),
            }
        }
        Ok(Self {
            name,
            profile,
            wasm,
        })
    }
    pub fn process(&self, info: &PostbuildArgs) -> Result<()> {
        for wasm in &self.wasm {
            wasm.process(info, &self.name)?;
        }

        Ok(())
    }
}
