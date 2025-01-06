use std::{fmt::Display, str::FromStr};

use camino_fs::Utf8PathBuf;

use crate::Output;

#[derive(Debug, Default, PartialEq, Eq)]
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

impl Display for FontForgeCmd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "font_file={}", self.font_file)?;
        for out in &self.output {
            writeln!(f, "output={}", out)?;
        }
        Ok(())
    }
}

impl FromStr for FontForgeCmd {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut cmd = FontForgeCmd::default();
        for line in s.lines() {
            let (key, value) = line.split_once('=').unwrap();
            match key {
                "font_file" => cmd.font_file = value.into(),
                "output" => cmd.output.push(value.parse().unwrap()),
                _ => panic!("unknown key: {}", key),
            }
        }
        Ok(cmd)
    }
}
