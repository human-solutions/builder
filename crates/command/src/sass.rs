use camino_fs::Utf8PathBuf;
use serde::{Deserialize, Serialize};

use crate::Output;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SassCmd {
    pub in_scss: Utf8PathBuf,

    pub optimize: bool,

    pub output: Vec<Output>,
    pub replacements: Vec<(String, String)>,
}

impl SassCmd {
    pub fn new<P: Into<Utf8PathBuf>>(in_scss: P) -> Self {
        Self {
            in_scss: in_scss.into(),
            optimize: false,
            output: Vec::new(),
            replacements: Vec::new(),
        }
    }

    pub fn add_css_replacement<S1: AsRef<str>, S2: AsRef<str>>(mut self, from: S1, to: S2) -> Self {
        self.replacements
            .push((from.as_ref().to_string(), to.as_ref().to_string()));
        self
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
