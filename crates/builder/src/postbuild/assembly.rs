use crate::anyhow::Result;
use crate::Config;
use serde::{Deserialize, Serialize};

use super::wasm::WasmBindgen;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Assembly {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
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
