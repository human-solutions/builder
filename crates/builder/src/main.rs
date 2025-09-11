use core::panic;
use std::env;

use builder_command::{BuilderCmd, Cmd};
use camino_fs::*;
use common::{LOG_LEVEL, RELEASE, setup_logging};

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
    let content = file.read_string().unwrap();
    let builder: BuilderCmd = content.parse().unwrap();

    RELEASE.set(builder.release).unwrap();

    setup_logging(builder.log_level, builder.log_destination.clone());
    LOG_LEVEL.set(builder.log_level).unwrap();

    let bin_version = env!("CARGO_PKG_VERSION");
    let metadata = cargo_metadata::MetadataCommand::new().exec().unwrap();

    let lib_version = metadata
        .packages
        .iter()
        .find(|pack| pack.name.as_str() == "builder-command")
        .unwrap()
        .version
        .to_string();
    if bin_version != lib_version {
        panic!(
            "Version mismatch: builder-command binary is {bin_version} but library is {lib_version}",
        );
    }
    run(builder);
}

pub fn run(builder: BuilderCmd) {
    for cmd in &builder.cmds {
        match cmd {
            Cmd::Uniffi(cmd) => builder_uniffi::run(cmd),
            Cmd::Sass(cmd) => builder_sass::run(cmd),
            Cmd::Localized(cmd) => builder_localized::run(cmd),
            Cmd::FontForge(cmd) => builder_fontforge::run(cmd),
            Cmd::Assemble(cmd) => builder_assemble::run(cmd),
            Cmd::Wasm(cmd) => builder_wasm::run(cmd),
            Cmd::Copy(cmd) => builder_copy::run(cmd),
            Cmd::SwiftPackage(cmd) => builder_swift_package::run(cmd),
        }
    }
}
