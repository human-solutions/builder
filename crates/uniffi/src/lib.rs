use builder_command::UniffiCmd;
use camino_fs::*;
use common::{Timer, log_command, log_operation, log_trace};
use uniffi_bindgen::{
    bindings::{KotlinBindingGenerator, SwiftBindingGenerator},
    generate_external_bindings,
};

pub fn run(cmd: &UniffiCmd) {
    let _timer = Timer::new("UNIFFI processing");
    log_command!("UNIFFI", "Processing library: {}", cmd.library_name);
    log_operation!("UNIFFI", "UDL file: {}", cmd.udl_file);
    log_operation!("UNIFFI", "Output directory: {}", cmd.out_dir);
    log_operation!("UNIFFI", "Kotlin: {}, Swift: {}", cmd.kotlin, cmd.swift);

    let udl_copy = cmd.out_dir.join(cmd.udl_file.file_name().unwrap());
    let cli_copy = cmd.out_dir.join("self.json");
    let conf_copy = cmd.out_dir.join("uniffi.toml");

    log_trace!(
        "UNIFFI",
        "Checking cache files: udl={}, cli={}, config={}",
        udl_copy,
        cli_copy,
        conf_copy
    );

    if udl_copy.exists() && cli_copy.exists() {
        let udl_ref_bytes = udl_copy.read_bytes().unwrap();
        let udl_src_bytes = cmd.udl_file.read_bytes().unwrap();
        let is_udl_same = udl_ref_bytes == udl_src_bytes;

        let prev_cli: UniffiCmd = serde_json::from_str(&cli_copy.read_string().unwrap()).unwrap();
        let is_cli_same = prev_cli == *cmd;

        // Check if config file content changed
        let is_config_same = match (conf_copy.exists(), &cmd.config_file) {
            (false, None) => true,
            (true, None) | (false, Some(_)) => false,
            (true, Some(current)) => {
                conf_copy.read_bytes().unwrap() == current.read_bytes().unwrap()
            }
        };

        match (is_udl_same, is_cli_same, is_config_same) {
            (true, true, true) => {
                log_command!("UNIFFI", "No changes detected, skipping generation");
                return;
            }
            (false, _, _) => {
                log_operation!("UNIFFI", "UDL file changed, regenerating bindings");
            }
            (_, false, _) => {
                log_operation!("UNIFFI", "CLI parameters changed, regenerating bindings");
            }
            (_, _, false) => {
                log_operation!(
                    "UNIFFI",
                    "Configuration file changed, regenerating bindings"
                );
            }
        }
    } else {
        log_operation!("UNIFFI", "First time processing, setting up cache files");
    }

    log_operation!("UNIFFI", "Setting up output directory and cache files");
    cmd.out_dir.mkdirs().unwrap();
    cmd.udl_file.cp(&udl_copy).unwrap();
    if let Some(config_file) = &cmd.config_file {
        log_trace!("UNIFFI", "Copying config file: {}", config_file);
        config_file.cp(&conf_copy).unwrap();
    }
    cli_copy.write(serde_json::to_string(cmd).unwrap()).unwrap();

    if cmd.kotlin {
        log_operation!(
            "UNIFFI",
            "Generating Kotlin bindings for library: {}",
            cmd.library_name
        );
        generate_external_bindings(
            &KotlinBindingGenerator,
            &cmd.udl_file,
            cmd.config_file.as_ref(),
            Some(&cmd.out_dir),
            Some(cmd.built_lib_file.clone()),
            Some(&cmd.library_name),
            true,
        )
        .unwrap();
        log_operation!("UNIFFI", "Kotlin bindings generation completed");
    }
    if cmd.swift {
        log_operation!(
            "UNIFFI",
            "Generating Swift bindings for library: {}",
            cmd.library_name
        );
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
        log_operation!("UNIFFI", "Fixing Swift modulemap file");
        fix_modulemap_file(&cmd.out_dir);
        log_operation!("UNIFFI", "Swift bindings generation completed");
    }
}

/// the generated module file starts with "module " but it should be "framework module "
fn fix_modulemap_file(out_dir: &Utf8Path) {
    let modulemap_file = out_dir
        .ls()
        .files()
        .find(|f| f.extension() == Some("modulemap"))
        .unwrap();

    log_trace!("UNIFFI", "Found modulemap file: {}", modulemap_file);

    let modulemap = modulemap_file.read_string().unwrap();

    if !modulemap.starts_with("module ") {
        panic!("modulemap file does not start with 'module '")
    }

    let mut new_modulemap = String::with_capacity(modulemap.len() + 10);
    new_modulemap.push_str("framework ");
    new_modulemap.push_str(&modulemap);

    modulemap_file.write(new_modulemap.as_bytes()).unwrap();
    log_trace!("UNIFFI", "Fixed modulemap file: added 'framework' prefix");
}
