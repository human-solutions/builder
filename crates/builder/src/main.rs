use core::panic;
use std::env;

use builder_command::{BuilderCmd, Cmd};
use camino::Utf8Path;
use common::{setup_logging, RELEASE};
use fs_err as fs;

fn main() {
    let args = std::env::args().collect::<Vec<_>>();

    if args[1] == "-V" {
        println!("builder {}", env!("CARGO_PKG_VERSION"));
        return;
    }

    let file = Utf8Path::new(&args[1]);
    if !file.is_file() {
        panic!("File not found: {:?}", file);
    }
    let bytes = fs::read_to_string(file).unwrap();
    let builder: BuilderCmd = toml::from_str(&bytes).unwrap();

    RELEASE.set(builder.release).unwrap();

    let is_ci = env::var("CI").is_ok();
    setup_logging(builder.verbose || is_ci);
    run(builder);
}

pub fn run(builder: BuilderCmd) {
    for cmd in &builder.cmds {
        match cmd {
            Cmd::Uniffi(cmd) => builder_uniffi::run(&cmd),
            Cmd::Sass(cmd) => builder_sass::run(&cmd),
            Cmd::Localized(cmd) => builder_localized::run(&cmd),
            Cmd::FontForge(cmd) => builder_fontforge::run(&cmd),
            Cmd::Assemble(cmd) => builder_assemble::run(&cmd),
            Cmd::Wasm(cmd) => builder_wasm::run(&cmd),
        }
    }
}
