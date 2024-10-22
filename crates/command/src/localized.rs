use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};

use crate::Output;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct LocalizedCmd {
    pub input_dir: Utf8PathBuf,

    /// File extensions that should be processed when searching for files in the input directory
    pub file_extension: String,

    pub output: Vec<Output>,
}

impl LocalizedCmd {
    pub fn new<P: Into<Utf8PathBuf>, S: AsRef<str>>(input_dir: P, file_extension: S) -> Self {
        Self {
            input_dir: input_dir.into(),
            file_extension: file_extension.as_ref().to_string(),
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
