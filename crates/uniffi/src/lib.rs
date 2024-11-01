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

        let prev_cli: UniffiCmd =
            serde_json::from_slice(cli_copy.read_bytes().unwrap().as_slice()).unwrap();
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
    cli_copy
        .write(serde_json::to_vec_pretty(&cmd).unwrap())
        .unwrap();

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
            true,
        )
        .unwrap();
    }
}
