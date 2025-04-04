mod assemble;
mod copy;
mod fontforge;
mod localized;
mod out;
mod sass;
mod swift_package;
mod uniffi;
mod wasm;

use std::{convert::Infallible, env, fmt::Display, path::Path, process::Command, str::FromStr};

pub use assemble::AssembleCmd;
use camino_fs::Utf8PathBuf;
pub use copy::CopyCmd;
pub use fontforge::FontForgeCmd;
pub use localized::LocalizedCmd;
pub use out::{Encoding, Output};
pub use sass::SassCmd;
use std::fs;
pub use swift_package::SwiftPackageCmd;
pub use uniffi::UniffiCmd;
pub use wasm::WasmCmd;

#[derive(Debug, PartialEq)]
pub struct BuilderCmd {
    pub verbose: bool,
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
        Self {
            cmds: Vec::new(),
            verbose: false,
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
    pub fn add_wasm(mut self, cmd: WasmCmd) -> Self {
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

    pub fn verbose(mut self, val: bool) -> Self {
        self.verbose = val;
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

        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).unwrap();
            }
        }

        self.log(&format!("Writing builder.yaml to {path}"));
        fs::write(path, self.to_string().as_bytes()).unwrap();

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
        if self.verbose && self.in_cargo {
            println!("cargo::warning={msg}");
        } else if self.verbose {
            println!("{msg}");
        }
    }
}

impl Display for BuilderCmd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "verbose={}", self.verbose)?;
        writeln!(f, "release={}", self.release)?;
        writeln!(f, "builder_toml={}", self.builder_toml)?;
        for cmd in &self.cmds {
            writeln!(f, "{}", cmd)?;
        }
        Ok(())
    }
}
impl FromStr for BuilderCmd {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = s.lines();
        let mut builder = BuilderCmd::new();
        for line in lines.by_ref().take(3) {
            let (key, value) = line.split_once('=').unwrap();
            match key {
                "verbose" => builder.verbose = value.parse().unwrap(),
                "release" => builder.release = value.parse().unwrap(),
                "builder_toml" => builder.builder_toml = value.parse().unwrap(),
                _ => panic!("Unknown key: {}", key),
            }
        }
        let rest = lines.collect::<Vec<_>>().join("\n");
        for cmd in rest.split('>') {
            if cmd.is_empty() {
                continue;
            }
            builder.cmds.push(cmd.parse().unwrap());
        }
        Ok(builder)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Cmd {
    Uniffi(UniffiCmd),
    Sass(SassCmd),
    Localized(LocalizedCmd),
    FontForge(FontForgeCmd),
    Assemble(AssembleCmd),
    Wasm(WasmCmd),
    Copy(CopyCmd),
    SwiftPackage(SwiftPackageCmd),
}

impl Display for Cmd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Cmd::Uniffi(cmd) => write!(f, ">Uniffi\n{}", cmd),
            Cmd::Sass(cmd) => write!(f, ">Sass\n{}", cmd),
            Cmd::Localized(cmd) => write!(f, ">Localized\n{}", cmd),
            Cmd::FontForge(cmd) => write!(f, ">FontForge\n{}", cmd),
            Cmd::Assemble(cmd) => write!(f, ">Assemble\n{}", cmd),
            Cmd::Wasm(cmd) => write!(f, ">Wasm\n{}", cmd),
            Cmd::Copy(cmd) => write!(f, ">Copy\n{}", cmd),
            Cmd::SwiftPackage(cmd) => write!(f, ">SwiftPackage\n{}", cmd),
        }
    }
}

impl FromStr for Cmd {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = s.lines();

        let cmd = lines.next().unwrap();
        let rest = lines.collect::<Vec<_>>().join("\n");
        match cmd {
            "Uniffi" => Ok(Cmd::Uniffi(rest.parse().unwrap())),
            "Sass" => Ok(Cmd::Sass(rest.parse().unwrap())),
            "Localized" => Ok(Cmd::Localized(rest.parse().unwrap())),
            "FontForge" => Ok(Cmd::FontForge(rest.parse().unwrap())),
            "Assemble" => Ok(Cmd::Assemble(rest.parse().unwrap())),
            "Wasm" => Ok(Cmd::Wasm(rest.parse().unwrap())),
            "Copy" => Ok(Cmd::Copy(rest.parse().unwrap())),
            "SwiftPackage" => Ok(Cmd::SwiftPackage(rest.parse().unwrap())),
            _ => panic!("Unknown command: {}", cmd),
        }
    }
}

#[test]
fn roundtrip() {
    let cmd = BuilderCmd::new()
        .add_unffi(UniffiCmd::default())
        .add_sass(SassCmd::default())
        .add_localized(LocalizedCmd::default())
        .add_fontforge(FontForgeCmd::default())
        .add_assemble(AssembleCmd::default())
        .add_wasm(WasmCmd::default())
        .add_copy(CopyCmd::default())
        .add_swift_package(SwiftPackageCmd::default())
        .verbose(true)
        .release(true)
        .builder_toml("builder.toml");

    let s = cmd.to_string();
    let cmd2 = s.parse::<BuilderCmd>().unwrap();
    assert_eq!(cmd, cmd2);
}
