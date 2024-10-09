use std::process::{Command, ExitCode};

use camino::{Utf8Path, Utf8PathBuf};
use clap::Parser;
use common::{
    dir::{self, remove_content_of_dir},
    out::{self, OutputParams},
    setup_logging,
};
use fs_err as fs;

fn main() -> ExitCode {
    let args = Cli::parse();
    fontforge(args);
    ExitCode::SUCCESS
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long, value_name = "FILE")]
    /// Input sfd file path
    font_file: Utf8PathBuf,

    #[arg(short, long, value_name = "DIR")]
    /// Where to write the woff2 files
    output_dir: Utf8PathBuf,

    #[arg(long)]
    no_brotli: bool,

    #[arg(long)]
    no_gzip: bool,

    #[arg(long)]
    no_uncompressed: bool,

    #[arg(long)]
    no_checksum: bool,

    #[arg(short, long)]
    verbose: bool,
}

impl OutputParams for Cli {
    fn brotli(&self) -> bool {
        !self.no_brotli
    }
    fn gzip(&self) -> bool {
        !self.no_gzip
    }
    fn uncompressed(&self) -> bool {
        !self.no_uncompressed
    }
    fn checksum(&self) -> bool {
        !self.no_checksum
    }
}
fn fontforge(cli: Cli) {
    setup_logging(cli.verbose);

    log::info!("Running builder-fontforge");
    let sfd_file = Utf8Path::new(&cli.font_file);
    let sum_file = sfd_file.with_extension("hash");
    let name = sfd_file.file_stem().unwrap();

    dir::remove_files_containing(&cli.output_dir, &format!("{name}.woff2"));

    if !sfd_file.exists() {
        panic!("File not found: {:?}", sfd_file);
    }
    let sfd_bytes = fs::read(sfd_file).unwrap();
    let hash = format!("{:x}", seahash::hash(&sfd_bytes));

    let sfd_dir = sfd_file.parent().unwrap();

    if sum_file.exists() {
        let current_hash = fs::read_to_string(&sum_file).unwrap();

        if hash == current_hash {
            log::info!("No change detected, skipping {sfd_file}");
        } else {
            generate_woff2_otf(sfd_dir, name);
            let otf_file = sfd_dir.join(name).with_extension("otf");

            // copy otf file to font directory (only macos)
            if cfg!(target_os = "macos") {
                macos_install_font(&otf_file, name);
            }
            fs::remove_file(&otf_file).unwrap();
            log::info!("Removed {otf_file}");
        }
    }

    remove_content_of_dir(&cli.output_dir);
    let contents = fs::read(&sfd_dir.join(name).with_extension("woff2")).unwrap();
    out::write(
        &cli,
        &contents,
        &cli.output_dir.join(name).with_extension("woff2"),
    );
    // fs::write(&sum_file, hash).unwrap();
}

fn generate_woff2_otf(sfd_dir: &Utf8Path, name: &str) {
    let ff = format!("Open('{name}.sfd'); Generate('{name}.woff2'); Generate('{name}.otf')");

    log::info!("Generating {name}.woff2 and {name}.otf from {name}.sfd");
    let cmd = Command::new("fontforge")
        .args(["-lang=ff", "-c", &ff])
        .current_dir(sfd_dir)
        .status()
        .unwrap();

    if !cmd.success() {
        panic!("installed binary fontforge failed")
    }
}

fn macos_install_font(otf_file: &Utf8Path, name: &str) {
    log::info!("Copying polygot.otf to ~/Library/Fonts");
    let home = std::env::var("HOME").unwrap();
    let dest = Utf8Path::new(&home)
        .join("Library/Fonts")
        .join(name)
        .with_extension("otf");
    fs::copy(&otf_file, dest).unwrap();
}
