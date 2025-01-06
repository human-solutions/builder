use std::{fmt::Display, str::FromStr};

use camino_fs::Utf8PathBuf;

use crate::Output;

#[derive(Debug, Default, PartialEq, Eq)]
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

impl Display for LocalizedCmd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "input_dir={}", self.input_dir)?;
        writeln!(f, "file_extension={}", self.file_extension)?;
        for out in &self.output {
            writeln!(f, "output={}", out)?;
        }
        Ok(())
    }
}

impl FromStr for LocalizedCmd {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut me = Self::default();
        for line in s.lines() {
            let (key, value) = line.split_once('=').unwrap();
            match key {
                "input_dir" => me.input_dir = value.into(),
                "file_extension" => me.file_extension = value.into(),
                "output" => me.output.push(value.parse().unwrap()),
                _ => panic!("unknown key: {}", key),
            }
        }
        Ok(me)
    }
}
