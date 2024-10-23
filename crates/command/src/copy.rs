use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};

use crate::Output;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CopyCmd {
    pub src_dir: Utf8PathBuf,

    /// File extensions that should be processed when searching for files in the input directory
    pub file_extensions: Vec<String>,

    pub output: Vec<Output>,
}

impl CopyCmd {
    pub fn new<P: Into<Utf8PathBuf>>(input_dir: P) -> Self {
        Self {
            src_dir: input_dir.into(),
            file_extensions: Default::default(),
            output: Vec::new(),
        }
    }
    pub fn file_extensions<It, S>(mut self, it: It) -> Self
    where
        It: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.file_extensions.extend(it.into_iter().map(Into::into));
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
