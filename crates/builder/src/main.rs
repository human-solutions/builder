use core::panic;
use std::env;

use builder::builder_command::BuilderCmd;
use camino_fs::*;

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
    let builder_cmd: BuilderCmd = serde_yaml::from_str(&content).unwrap();

    // Version check
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

    // Execute commands using the library
    builder::execute(builder_cmd);
}
