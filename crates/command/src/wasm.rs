use std::{convert::Infallible, fmt::Display, str::FromStr};

use crate::Output;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct WasmProcessingCmd {
    /// The package name
    pub package: String,

    pub output: Vec<Output>,
}

impl WasmProcessingCmd {
    pub fn new(package: &str) -> Self {
        Self {
            package: package.to_string(),
            output: Vec::new(),
        }
    }

    pub fn output(mut self, it: impl IntoIterator<Item = Output>) -> Self {
        self.output.extend(it);
        self
    }

    pub fn add_output(mut self, output: Output) -> Self {
        self.output.push(output);
        self
    }
}

impl Display for WasmProcessingCmd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "package={}", self.package)?;
        for out in &self.output {
            writeln!(f, "output={}", out)?;
        }
        Ok(())
    }
}

impl FromStr for WasmProcessingCmd {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut me = Self::default();
        for line in s.lines() {
            let (key, value) = line.split_once('=').unwrap();
            match key {
                "package" => me.package = value.to_string(),
                "output" => me.output.push(value.parse().unwrap()),
                _ => panic!("unknown key: {}", key),
            }
        }
        Ok(me)
    }
}
