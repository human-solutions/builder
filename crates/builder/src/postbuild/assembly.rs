use crate::anyhow::Result;
use crate::Config;
use serde::Deserialize;

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
    pub fn process(&self, info: &Config) -> Result<()> {
        for wasm in &self.wasm {
            wasm.process(info, &self.name)?;
        }

        Ok(())
    }
}
