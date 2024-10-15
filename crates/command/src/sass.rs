use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};

use crate::Output;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SassCmd {
    pub in_scss: Utf8PathBuf,

    pub optimize: bool,

    pub output: Vec<Output>,
}

impl SassCmd {
    pub fn new<P: Into<Utf8PathBuf>>(in_scss: P) -> Self {
        Self {
            in_scss: in_scss.into(),
            optimize: false,
            output: Vec::new(),
        }
    }

    pub fn optimize(mut self, optimize: bool) -> Self {
        self.optimize = optimize;
        self
    }

    pub fn add_output(mut self, output: Output) -> Self {
        self.output.push(output);
        self
    }

    pub fn output(mut self, it: impl IntoIterator<Item = Output>) -> Self {
        self.output.extend(it);
        self
    }
}
