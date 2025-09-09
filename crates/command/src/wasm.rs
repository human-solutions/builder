use std::{
    convert::Infallible,
    fmt::{Debug, Display},
    str::FromStr,
};

use camino_fs::Utf8PathBuf;

use crate::Output;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DebugSymbolsMode {
    /// Strip debug symbols without preserving them
    Strip,
    /// Keep debug symbols in the main WASM file
    Keep,
    /// Write debug symbols to a custom path and strip from main file
    WriteTo(Utf8PathBuf),
    /// Write debug symbols next to the main WASM file with .debug.wasm extension and strip from main file
    WriteAdjacent,
}

impl Default for DebugSymbolsMode {
    fn default() -> Self {
        Self::Strip
    }
}

impl Display for DebugSymbolsMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DebugSymbolsMode::Strip => write!(f, "strip"),
            DebugSymbolsMode::Keep => write!(f, "keep"),
            DebugSymbolsMode::WriteTo(path) => write!(f, "write_to:{}", path),
            DebugSymbolsMode::WriteAdjacent => write!(f, "adjacent"),
        }
    }
}

impl FromStr for DebugSymbolsMode {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "strip" => Ok(DebugSymbolsMode::Strip),
            "keep" => Ok(DebugSymbolsMode::Keep),
            "adjacent" => Ok(DebugSymbolsMode::WriteAdjacent),
            _ if s.starts_with("write_to:") => {
                let path = &s[9..]; // Skip "write_to:" prefix
                Ok(DebugSymbolsMode::WriteTo(path.parse().unwrap()))
            }
            _ => panic!("Invalid debug symbols mode: {}", s),
        }
    }
}
impl DebugSymbolsMode {
    pub fn write_to(path: impl Into<Utf8PathBuf>) -> Self {
        Self::WriteTo(path.into())
    }
}

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

    /// Debug symbol handling strategy
    pub debug_symbols: DebugSymbolsMode,
}

impl WasmProcessingCmd {
    pub fn new(package: &str, profile: Profile) -> Self {
        Self {
            package: package.to_string(),
            profile,
            output: Vec::new(),
            debug_symbols: DebugSymbolsMode::default(),
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

    /// Strip debug symbols (default behavior)
    pub fn debug_symbols(mut self, mode: DebugSymbolsMode) -> Self {
        self.debug_symbols = mode;
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
        writeln!(f, "debug_symbols={}", self.debug_symbols)?;
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
                "debug_symbols" => me.debug_symbols = value.parse().unwrap(),
                _ => panic!("unknown key: {}", key),
            }
        }
        Ok(me)
    }
}
