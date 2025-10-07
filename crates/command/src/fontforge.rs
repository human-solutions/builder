use builder_mtimes::{InputFiles, OutputFiles};
use camino_fs::Utf8PathBuf;
use serde::{Deserialize, Serialize};

use crate::Output;

#[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
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

impl InputFiles for FontForgeCmd {
    fn input_files(&self) -> Vec<Utf8PathBuf> {
        vec![self.font_file.clone()]
    }
}

impl OutputFiles for FontForgeCmd {
    fn output_files(&self) -> Vec<Utf8PathBuf> {
        let stem = self.font_file.file_stem().unwrap_or_default();
        self.output
            .iter()
            .map(|out| out.dir.join(format!("{}.woff2", stem)))
            .collect()
    }
}

impl crate::CommandMetadata for FontForgeCmd {
    fn output_dir(&self) -> &camino_fs::Utf8Path {
        &self
            .output
            .first()
            .expect("FontForge command must have output")
            .dir
    }

    fn name(&self) -> &'static str {
        "fontforge"
    }
}
