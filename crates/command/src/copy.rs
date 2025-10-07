use builder_mtimes::{InputFiles, OutputFiles};
use camino_fs::Utf8PathBuf;
use serde::{Deserialize, Serialize};

use crate::Output;

#[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CopyCmd {
    pub src_dir: Utf8PathBuf,

    pub recursive: bool,

    /// File extensions that should be processed when searching for files in the input directory
    pub file_extensions: Vec<String>,

    pub output: Vec<Output>,
}

impl CopyCmd {
    pub fn new<P: Into<Utf8PathBuf>>(input_dir: P) -> Self {
        Self {
            src_dir: input_dir.into(),
            recursive: false,
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

    pub fn recursive(mut self, val: bool) -> Self {
        self.recursive = val;
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

impl InputFiles for CopyCmd {
    fn input_files(&self) -> Vec<Utf8PathBuf> {
        use camino_fs::*;
        let mut files = Vec::new();
        if self.src_dir.exists() {
            let recursive = self.recursive;
            let iter = self
                .src_dir
                .ls()
                .recurse_if(move |_| recursive)
                .filter(|p| p.is_file());
            for entry in iter {
                if self.file_extensions.is_empty()
                    || entry
                        .extension()
                        .is_some_and(|e| self.file_extensions.contains(&e.to_string()))
                {
                    files.push(entry);
                }
            }
        }
        files
    }
}

impl OutputFiles for CopyCmd {
    fn output_files(&self) -> Vec<Utf8PathBuf> {
        self.output.iter().map(|out| out.dir.clone()).collect()
    }
}

impl crate::CommandMetadata for CopyCmd {
    fn output_dir(&self) -> &camino_fs::Utf8Path {
        &self
            .output
            .first()
            .expect("Copy command must have output")
            .dir
    }

    fn name(&self) -> &'static str {
        "copy"
    }
}
