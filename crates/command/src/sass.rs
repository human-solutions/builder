use std::{convert::Infallible, fmt::Display, str::FromStr};

use camino_fs::Utf8PathBuf;

use crate::Output;

#[derive(Debug, Default, PartialEq, Eq)]
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

impl Display for SassCmd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "in_scss={}", self.in_scss)?;
        writeln!(f, "optimize={}", self.optimize)?;
        for out in &self.output {
            writeln!(f, "output={}", out)?;
        }
        for (from, to) in &self.replacements {
            writeln!(f, "replacement={}:{}", from, to)?;
        }
        Ok(())
    }
}

impl FromStr for SassCmd {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut cmd = SassCmd::default();
        for line in s.lines() {
            let (key, value) = line.split_once('=').unwrap();
            match key {
                "in_scss" => cmd.in_scss = value.into(),
                "optimize" => cmd.optimize = value.parse().unwrap(),
                "output" => cmd.output.push(value.parse().unwrap()),
                "replacement" => {
                    let (from, to) = value.split_once(':').unwrap();
                    cmd.replacements.push((from.to_string(), to.to_string()));
                }
                _ => panic!("unknown key: {}", key),
            }
        }
        Ok(cmd)
    }
}
