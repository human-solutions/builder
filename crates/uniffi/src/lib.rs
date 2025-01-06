use builder_command::UniffiCmd;
use camino_fs::*;
use uniffi_bindgen::{
    bindings::{KotlinBindingGenerator, SwiftBindingGenerator},
    generate_external_bindings,
};

pub fn run(cmd: &UniffiCmd) {
    log::info!("Running builder-uniffi");

    let udl_copy = cmd.out_dir.join(cmd.udl_file.file_name().unwrap());
    let cli_copy = cmd.out_dir.join("self.json");

    if udl_copy.exists() && cli_copy.exists() {
        let udl_ref_bytes = udl_copy.read_bytes().unwrap();
        let udl_src_bytes = cmd.udl_file.read_bytes().unwrap();
        let is_udl_same = udl_ref_bytes == udl_src_bytes;

        let prev_cli: UniffiCmd = cli_copy.read_string().unwrap().parse().unwrap();
        let is_cli_same = prev_cli == *cmd;

        match (is_udl_same, is_cli_same) {
            (true, true) => {
                log::info!("No changes to UDL file nor cli params, skipping generation");
                return;
            }
            (true, false) => {
                log::info!("CLI params changed, regenerating bindings")
            }
            (false, true) => {
                log::info!("UDL file changed, regenerating bindings")
            }
            (false, false) => {
                log::info!("UDL file and CLI params changed, regenerating bindings")
            }
        }
    }
    cmd.out_dir.mkdirs().unwrap();
    cmd.udl_file.cp(&udl_copy).unwrap();
    cli_copy.write(&cmd.to_string()).unwrap();

    if cmd.kotlin {
        log::info!("Generating Kotlin bindings for {}", cmd.library_name);
        generate_external_bindings(
            &KotlinBindingGenerator,
            &cmd.udl_file,
            None::<&Utf8PathBuf>,
            Some(&cmd.out_dir),
            // None::<&Utf8PathBuf>,
            Some(cmd.built_lib_file.clone()),
            Some(&cmd.library_name),
            true,
        )
        .unwrap();
    }
    if cmd.swift {
        log::info!("Generating Swift bindings for {}", cmd.library_name);
        generate_external_bindings(
            &SwiftBindingGenerator,
            &cmd.udl_file,
            None::<&Utf8PathBuf>,
            Some(&cmd.out_dir),
            Some(cmd.built_lib_file.clone()),
            Some(&cmd.library_name),
            false,
        )
        .unwrap();
        fix_modulemap_file(&cmd.out_dir);
    }
}

/// the generated module file starts with "module " but it should be "framework module "
fn fix_modulemap_file(out_dir: &Utf8Path) {
    let modulemap_file = out_dir
        .ls()
        .files()
        .find(|f| f.extension() == Some("modulemap"))
        .unwrap();

    let modulemap = modulemap_file.read_string().unwrap();

    if !modulemap.starts_with("module ") {
        panic!("modulemap file does not start with 'module '")
    }

    let mut new_modulemap = String::with_capacity(modulemap.len() + 10);
    new_modulemap.push_str("framework ");
    new_modulemap.push_str(&modulemap);

    modulemap_file.write(new_modulemap.as_bytes()).unwrap();
}
