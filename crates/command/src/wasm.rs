use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};

use crate::Output;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct WasmCmd {
    pub package_dir: Utf8PathBuf,
    pub output_dir: Utf8PathBuf,
    pub name: String,
    pub optimize: bool,

    pub output: Vec<Output>,
}

impl WasmCmd {
    pub fn new(
        name: &str,
        package_dir: impl Into<Utf8PathBuf>,
        output_dir: impl Into<Utf8PathBuf>,
    ) -> Self {
        Self {
            package_dir: package_dir.into(),
            output_dir: output_dir.into(),
            name: name.to_string(),
            optimize: false,
            output: Vec::new(),
        }
    }

    pub fn optimize(mut self, optimize: bool) -> Self {
        self.optimize = optimize;
        self
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
