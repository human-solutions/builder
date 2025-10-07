use builder_mtimes::{InputFiles, OutputFiles};
use camino_fs::Utf8PathBuf;
use serde::{Deserialize, Serialize};

use crate::Output;

#[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
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

impl InputFiles for SassCmd {
    fn input_files(&self) -> Vec<Utf8PathBuf> {
        vec![self.in_scss.clone()]
    }
}

impl OutputFiles for SassCmd {
    fn output_files(&self) -> Vec<Utf8PathBuf> {
        self.output
            .iter()
            .flat_map(|out| {
                let stem = self.in_scss.file_stem().unwrap_or_default();
                let css_path = out.dir.join(format!("{}.css", stem));
                let mut paths = vec![];

                // Add uncompressed if enabled
                if out.uncompressed() {
                    paths.push(css_path.clone());
                }
                // Add gzip variant if enabled
                if out.gzip() {
                    paths.push(css_path.with_extension("css.gz"));
                }
                // Add brotli variant if enabled
                if out.brotli() {
                    paths.push(css_path.with_extension("css.br"));
                }

                paths
            })
            .collect()
    }
}

impl crate::CommandMetadata for SassCmd {
    fn output_dir(&self) -> &camino_fs::Utf8Path {
        &self
            .output
            .first()
            .expect("SASS command must have output")
            .dir
    }

    fn name(&self) -> &'static str {
        "sass"
    }
}
