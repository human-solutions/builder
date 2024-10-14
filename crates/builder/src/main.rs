use core::panic;

use builder_command::{BuilderCmd, Cmd};
use camino::Utf8Path;
use common::setup_logging;
use fs_err as fs;

fn main() {
    let args = std::env::args().collect::<Vec<_>>();

    let file = Utf8Path::new(&args[1]);
    if !file.is_file() {
        panic!("File not found: {:?}", file);
    }
    let bytes = fs::read_to_string(file).unwrap();
    let builder: BuilderCmd = toml::from_str(&bytes).unwrap();

    setup_logging(builder.verbose);
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
        }
    }
}
