use std::{fmt::Display, str::FromStr};

use camino_fs::Utf8PathBuf;

use crate::Output;

#[derive(Debug, Default, PartialEq, Eq)]
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

impl Display for CopyCmd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "src_dir={}", self.src_dir)?;
        writeln!(f, "recursive={}", self.recursive)?;
        for ext in &self.file_extensions {
            writeln!(f, "file_extensions={}", ext)?;
        }
        for out in &self.output {
            writeln!(f, "output={}", out)?;
        }
        Ok(())
    }
}

impl FromStr for CopyCmd {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut cmd = CopyCmd::default();
        for line in s.lines() {
            let (key, value) = line.split_once('=').unwrap();
            match key {
                "src_dir" => cmd.src_dir = value.into(),
                "recursive" => cmd.recursive = value.parse().unwrap(),
                "file_extensions" => {
                    cmd.file_extensions.push(value.into());
                }
                "output" => {
                    cmd.output.push(value.parse().unwrap());
                }
                _ => panic!("unknown key: {}", key),
            }
        }
        Ok(cmd)
    }
}
