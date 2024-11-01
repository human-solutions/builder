mod assemble;
mod copy;
mod fontforge;
mod localized;
mod out;
mod sass;
mod uniffi;
mod wasm;

use std::{env, path::Path, process::Command};

pub use assemble::AssembleCmd;
use camino_fs::Utf8PathBuf;
pub use copy::CopyCmd;
pub use fontforge::FontForgeCmd;
pub use localized::LocalizedCmd;
pub use out::{Encoding, Output};
pub use sass::SassCmd;
use serde::{Deserialize, Serialize};
use std::fs;
pub use uniffi::UniffiCmd;
pub use wasm::WasmCmd;

#[derive(Debug, Serialize, Deserialize)]
pub struct BuilderCmd {
    pub cmds: Vec<Cmd>,
    pub verbose: bool,
    pub release: bool,
    /// The directory where the builder.toml file is located
    /// Defaults to env OUT_DIR
    pub builder_toml: Utf8PathBuf,
    in_cargo: bool,
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
        let yaml = toml::to_string(&self).unwrap();

        let path = &self.builder_toml;

        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).unwrap();
            }
        }

        self.log(&format!("Writing builder.yaml to {path}"));
        fs::write(path, yaml.as_bytes()).unwrap();

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

#[derive(Debug, Serialize, Deserialize)]
pub enum Cmd {
    Uniffi(UniffiCmd),
    Sass(SassCmd),
    Localized(LocalizedCmd),
    FontForge(FontForgeCmd),
    Assemble(AssembleCmd),
    Wasm(WasmCmd),
    Copy(CopyCmd),
}
