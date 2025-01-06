use core::panic;
use std::env;

use builder_command::{BuilderCmd, Cmd};
use camino_fs::*;
use common::{setup_logging, RELEASE, VERBOSE};

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

    let is_ci = env::var("CI").is_ok();
    setup_logging(builder.verbose || is_ci);
    VERBOSE.set(builder.verbose).unwrap();

    let bin_version = env!("CARGO_PKG_VERSION");
    let metadata = cargo_metadata::MetadataCommand::new().exec().unwrap();

    let lib_version = metadata
        .packages
        .iter()
        .find(|pack| pack.name == "builder-command")
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
