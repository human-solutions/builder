use camino::Utf8PathBuf;
use clap::Parser;
use common::setup_logging;
use fs_err as fs;
use serde::{Deserialize, Serialize};
use uniffi_bindgen::{
    bindings::{KotlinBindingGenerator, SwiftBindingGenerator},
    generate_external_bindings,
};

#[derive(Parser, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[arg(short, long, value_name = "FILE")]
    udl_file: Utf8PathBuf,

    #[arg(short, long, value_name = "DIR")]
    /// Where to generate the bindings
    out_dir: Utf8PathBuf,

    #[clap(long)]
    /// the .dylib or .so file to generate bindings for
    /// normally in target/debug or target/release
    pub built_lib_file: Utf8PathBuf,

    #[clap(long)]
    pub library_name: String,

    #[arg(long)]
    swift: bool,

    #[arg(long)]
    kotlin: bool,

    #[arg(short, long)]
    verbose: bool,
}

pub fn run(cli: &Cli) {
    setup_logging(cli.verbose);

    log::info!("Running builder-uniffi");

    let udl_copy = cli.out_dir.join(cli.udl_file.file_name().unwrap());
    let cli_copy = cli.out_dir.join("cli.json");

    if udl_copy.exists() && cli_copy.exists() {
        let udl_ref_bytes = fs::read(&udl_copy).unwrap();
        let udl_src_bytes = fs::read(&cli.udl_file).unwrap();
        let is_udl_same = udl_ref_bytes == udl_src_bytes;

        let prev_cli: Cli =
            serde_json::from_slice(fs::read(&cli_copy).unwrap().as_slice()).unwrap();
        let is_cli_same = prev_cli == *cli;

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
    if !cli.out_dir.exists() {
        fs::create_dir_all(&cli.out_dir).unwrap();
    }
    fs::copy(&cli.udl_file, &udl_copy).unwrap();
    fs::write(cli_copy, serde_json::to_vec_pretty(&cli).unwrap()).unwrap();

    if cli.kotlin {
        log::info!("Generating Kotlin bindings for {}", cli.library_name);
        generate_external_bindings(
            &KotlinBindingGenerator,
            &cli.udl_file,
            None::<&Utf8PathBuf>,
            Some(&cli.out_dir),
            // None::<&Utf8PathBuf>,
            Some(cli.built_lib_file.clone()),
            Some(&cli.library_name),
            true,
        )
        .unwrap();
    }
    if cli.swift {
        log::info!("Generating Swift bindings for {}", cli.library_name);
        generate_external_bindings(
            &SwiftBindingGenerator,
            &cli.udl_file,
            None::<&Utf8PathBuf>,
            Some(&cli.out_dir),
            Some(cli.built_lib_file.clone()),
            Some(&cli.library_name),
            true,
        )
        .unwrap();
    }
}
