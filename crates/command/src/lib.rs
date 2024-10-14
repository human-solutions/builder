mod assemble;
mod fontforge;
mod localized;
mod out;
mod sass;
mod uniffi;

use std::{env, process::Command};

pub use assemble::AssembleCmd;
use camino::Utf8Path;
pub use fontforge::FontForgeCmd;
pub use localized::LocalizedCmd;
pub use out::{Encoding, Output};
pub use sass::SassCmd;
use serde::{Deserialize, Serialize};
use std::fs;
pub use uniffi::UniffiCmd;

#[derive(Debug, Serialize, Deserialize)]
pub struct BuilderCmd {
    pub cmds: Vec<Cmd>,
    pub verbose: bool,
    in_cargo: bool,
}

impl BuilderCmd {
    pub fn new() -> Self {
        Self {
            cmds: Vec::new(),
            verbose: false,
            in_cargo: env::var("CARGO").is_ok(),
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

    pub fn verbose(mut self, val: bool) -> Self {
        self.verbose = val;
        self
    }

    pub fn run(self) {
        let outdir = env::var("OUT_DIR").unwrap_or(".".to_string());
        let yaml = toml::to_string(&self).unwrap();
        let path = Utf8Path::new(&outdir).join("builder.yaml");
        self.log(&format!("Writing builder.yaml to {:?}", path));
        fs::write(&path, yaml.as_bytes()).unwrap();

        let cmd = Command::new("builder").arg(path.as_str()).status().unwrap();

        self.log(&format!("Processed {path:?}"));
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
            eprintln!("{}", msg);
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
}
