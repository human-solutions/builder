use camino_fs::Utf8PathBuf;
use serde::{Deserialize, Serialize};

use crate::Output;

#[derive(Debug, Serialize, Deserialize)]
pub struct FontForgeCmd {
    /// Input sfd file path
    pub font_file: Utf8PathBuf,

    pub output: Vec<Output>,
}

impl FontForgeCmd {
    pub fn new<F: Into<Utf8PathBuf>>(font_file: F) -> Self {
        Self {
            font_file: font_file.into(),
            output: Vec::new(),
        }
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
