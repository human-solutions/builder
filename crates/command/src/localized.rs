use builder_mtimes::{InputFiles, OutputFiles};
use camino_fs::Utf8PathBuf;
use serde::{Deserialize, Serialize};

use crate::Output;

#[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
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

impl InputFiles for LocalizedCmd {
    fn input_files(&self) -> Vec<Utf8PathBuf> {
        use camino_fs::*;
        let mut files = Vec::new();
        if self.input_dir.exists() {
            for file in self.input_dir.ls() {
                if file.is_file()
                    && file
                        .extension()
                        .is_some_and(|ext| ext == self.file_extension)
                {
                    files.push(file);
                }
            }
        }
        files
    }
}

impl OutputFiles for LocalizedCmd {
    fn output_files(&self) -> Vec<Utf8PathBuf> {
        let name = format!(
            "{}.{}",
            self.input_dir.file_name().unwrap_or_default(),
            self.file_extension
        );
        self.output.iter().map(|out| out.dir.join(&name)).collect()
    }
}

impl crate::CommandMetadata for LocalizedCmd {
    fn output_dir(&self) -> &camino_fs::Utf8Path {
        &self
            .output
            .first()
            .expect("Localized command must have output")
            .dir
    }

    fn name(&self) -> &'static str {
        "localized"
    }
}
