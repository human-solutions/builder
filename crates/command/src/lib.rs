mod assemble;
mod copy;
mod fontforge;
mod localized;
mod out;
mod out_integration_test;
mod out_snapshot_test;
mod sass;
mod swift_package;
mod uniffi;
mod wasm;

use std::{env, path::Path, process::Command};

pub use assemble::AssembleCmd;
use camino_fs::Utf8PathBuf;
pub use copy::CopyCmd;
pub use fontforge::FontForgeCmd;
use fs_err as fs;
pub use localized::LocalizedCmd;
use log::LevelFilter;
pub use out::{AssetMetadata, Encoding, Output};
pub use sass::SassCmd;
use serde::{Deserialize, Serialize};
pub use swift_package::SwiftPackageCmd;
pub use uniffi::UniffiCmd;
pub use wasm::{DebugSymbolsMode, Profile, WasmProcessingCmd};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogLevel {
    Normal,  // Info level + enhanced summaries
    Verbose, // Debug + detailed operations
    Trace,   // Everything including file-level operations
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogDestination {
    Cargo,             // via cargo::warning
    File(Utf8PathBuf), // given a path
    Terminal,          // standard output
    TerminalPlain, // standard output, designed for when run in a Command that adds it's own prefixes to the logs
}

impl LogLevel {
    pub fn to_level_filter(self) -> LevelFilter {
        match self {
            LogLevel::Normal => LevelFilter::Info,
            LogLevel::Verbose => LevelFilter::Debug,
            LogLevel::Trace => LevelFilter::Trace,
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct BuilderCmd {
    pub log_level: LogLevel,
    pub log_destination: LogDestination,
    pub release: bool,
    /// The directory where the builder.toml file is located
    /// Defaults to env OUT_DIR
    pub builder_toml: Utf8PathBuf,
    in_cargo: bool,
    pub cmds: Vec<Cmd>,
}

impl Default for BuilderCmd {
    fn default() -> Self {
        Self::new()
    }
}

impl BuilderCmd {
    pub fn new() -> Self {
        let default_log_destination = if env::var("CI").is_ok() {
            LogDestination::Cargo
        } else {
            LogDestination::Terminal
        };

        Self {
            cmds: Vec::new(),
            log_level: LogLevel::Normal,
            log_destination: default_log_destination,
            release: env::var("PROFILE").unwrap_or_default() == "release",
            in_cargo: env::var("CARGO").is_ok(),
            builder_toml: Utf8PathBuf::from(
                env::var("OUT_DIR").ok().unwrap_or_else(|| ".".to_string()),
            )
            .join("builder.toml"),
        }
    }

    /// Add a UniffiCmd using it's builder
    pub fn add_unffi(mut self, cmd: UniffiCmd) -> Self {
        self.cmds.push(Cmd::Uniffi(cmd));
        self
    }

    /// Add a SassCmd using it's builder
    pub fn add_sass(mut self, cmd: SassCmd) -> Self {
        self.cmds.push(Cmd::Sass(cmd));
        self
    }

    /// Add a LocalizedCmd using it's builder
    pub fn add_localized(mut self, cmd: LocalizedCmd) -> Self {
        self.cmds.push(Cmd::Localized(cmd));
        self
    }

    /// Add a FontForgeCmd using it's builder
    pub fn add_fontforge(mut self, cmd: FontForgeCmd) -> Self {
        self.cmds.push(Cmd::FontForge(cmd));
        self
    }

    /// Add a AssembleCmd using it's builder
    pub fn add_assemble(mut self, cmd: AssembleCmd) -> Self {
        self.cmds.push(Cmd::Assemble(cmd));
        self
    }

    /// Add a WasmCmd using it's builder
    pub fn add_wasm(mut self, cmd: WasmProcessingCmd) -> Self {
        self.cmds.push(Cmd::Wasm(cmd));
        self
    }

    /// Add a CopyCmd using it's builder
    pub fn add_copy(mut self, cmd: CopyCmd) -> Self {
        self.cmds.push(Cmd::Copy(cmd));
        self
    }

    /// Add a SwiftPackageCmd using it's builder
    pub fn add_swift_package(mut self, cmd: SwiftPackageCmd) -> Self {
        self.cmds.push(Cmd::SwiftPackage(cmd));
        self
    }

    pub fn log_level(mut self, level: LogLevel) -> Self {
        self.log_level = level;
        self
    }

    pub fn log_destination(mut self, destination: LogDestination) -> Self {
        self.log_destination = destination;
        self
    }

    pub fn release(mut self, val: bool) -> Self {
        self.release = val;
        self
    }

    pub fn builder_toml<P: AsRef<Path>>(mut self, val: P) -> Self {
        self.builder_toml = Utf8PathBuf::from_path_buf(val.as_ref().to_path_buf()).unwrap();
        self
    }

    pub fn run(self) {
        let path = &self.builder_toml;

        if let Some(parent) = path.parent()
            && !parent.exists()
        {
            fs::create_dir_all(parent).unwrap();
        }

        self.log(&format!("Writing builder.json to {path}"));
        let json_content = serde_json::to_string_pretty(&self).unwrap();
        fs::write(path, json_content).unwrap();

        let cmd = Command::new("builder")
            .arg(self.builder_toml.as_str())
            .status()
            .unwrap();

        self.log(&format!("Processed {path}"));
        if cmd.success() {
            self.log("Command succeeded");
        } else {
            panic!("Command failed");
        }
    }

    fn log(&self, msg: &str) {
        let is_verbose = matches!(self.log_level, LogLevel::Verbose | LogLevel::Trace);
        if is_verbose {
            println!("{msg}");
        }
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Cmd {
    Uniffi(UniffiCmd),
    Sass(SassCmd),
    Localized(LocalizedCmd),
    FontForge(FontForgeCmd),
    Assemble(AssembleCmd),
    Wasm(WasmProcessingCmd),
    Copy(CopyCmd),
    SwiftPackage(SwiftPackageCmd),
}

#[test]
fn roundtrip() {
    let cmd = BuilderCmd::new()
        .add_unffi(UniffiCmd::default())
        .add_sass(SassCmd::default())
        .add_localized(LocalizedCmd::default())
        .add_fontforge(FontForgeCmd::default())
        .add_assemble(AssembleCmd::default())
        .add_wasm(WasmProcessingCmd::default().debug_symbols(DebugSymbolsMode::Keep))
        .add_copy(CopyCmd::default())
        .add_swift_package(SwiftPackageCmd::default())
        .log_level(LogLevel::Verbose)
        .log_destination(LogDestination::File(camino_fs::Utf8PathBuf::from(
            "/tmp/builder.log",
        )))
        .release(true)
        .builder_toml("builder.toml");

    let json = serde_json::to_string(&cmd).unwrap();
    let cmd2 = serde_json::from_str::<BuilderCmd>(&json).unwrap();
    assert_eq!(cmd, cmd2);
}

#[test]
fn roundtrip_log_destinations() {
    // Test all log destination variants
    let destinations = [
        LogDestination::Cargo,
        LogDestination::File(camino_fs::Utf8PathBuf::from("/path/to/log.txt")),
        LogDestination::Terminal,
        LogDestination::TerminalPlain,
    ];

    for destination in destinations {
        let cmd = BuilderCmd::new()
            .log_destination(destination)
            .log_level(LogLevel::Normal);

        let json = serde_json::to_string(&cmd).unwrap();
        let cmd2 = serde_json::from_str::<BuilderCmd>(&json).unwrap();
        assert_eq!(cmd, cmd2);
    }
}
