use serde::{Deserialize, Serialize};

use crate::Output;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct WasmCmd {
    /// The package name
    pub package: String,

    pub output: Vec<Output>,
}

impl WasmCmd {
    pub fn new(package: &str) -> Self {
        Self {
            package: package.to_string(),
            output: Vec::new(),
        }
    }

    pub fn output(mut self, it: impl IntoIterator<Item = Output>) -> Self {
        self.output.extend(it);
        self
    }

    pub fn add_output(mut self, output: Output) -> Self {
        self.output.push(output);
        self
    }
}
