use std::{convert::Infallible, fmt::Display, str::FromStr};

use camino_fs::Utf8PathBuf;

use crate::Output;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Profile {
    Release,
    #[default]
    Debug,
    Custom(String),
}
impl Profile {
    pub fn as_cargo_profile_arg(&self) -> String {
        match self {
            Profile::Release => "release".to_string(),
            Profile::Debug => "dev".to_string(),
            Profile::Custom(s) => s.to_string(),
        }
    }
    pub fn as_target_folder(&self) -> String {
        match self {
            Profile::Release => "release".to_string(),
            Profile::Debug => "debug".to_string(),
            Profile::Custom(s) => s.to_string(),
        }
    }
    pub fn from_target_folder(s: &str) -> Self {
        match s {
            "release" => Profile::Release,
            "debug" => Profile::Debug,
            _ => Profile::Custom(s.to_string()),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct WasmProcessingCmd {
    /// The package name
    pub package: String,

    /// The cargo build profile
    pub profile: Profile,

    pub output: Vec<Output>,

    /// Wether to extract debug symbols and write them to the given path
    /// Also removes them from the original wasm file
    pub write_debug_symbols_to: Option<Utf8PathBuf>,
}

impl WasmProcessingCmd {
    pub fn new(package: &str, profile: Profile) -> Self {
        Self {
            package: package.to_string(),
            profile,
            output: Vec::new(),
            write_debug_symbols_to: None,
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

    pub fn write_debug_symbols<P: Into<Utf8PathBuf>>(mut self, path: P) -> Self {
        self.write_debug_symbols_to = Some(path.into());
        self
    }
}

impl Default for WasmProcessingCmd {
    fn default() -> Self {
        Self::new("", Profile::default())
    }
}

impl Display for WasmProcessingCmd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "package={}", self.package)?;
        writeln!(f, "profile={}", self.profile.as_target_folder())?;
        for out in &self.output {
            writeln!(f, "output={}", out)?;
        }
        if let Some(path) = &self.write_debug_symbols_to {
            writeln!(f, "write_debug_symbols_to={}", path)?;
        }
        Ok(())
    }
}

impl FromStr for WasmProcessingCmd {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut me = Self::default();
        for line in s.lines() {
            if line.is_empty() {
                continue;
            }
            let (key, value) = line.split_once('=').unwrap();
            match key {
                "package" => me.package = value.to_string(),
                "profile" => me.profile = Profile::from_target_folder(value),
                "output" => me.output.push(value.parse().unwrap()),
                "write_debug_symbols_to" => {
                    me.write_debug_symbols_to = Some(value.parse().unwrap())
                }
                _ => panic!("unknown key: {}", key),
            }
        }
        Ok(me)
    }
}
