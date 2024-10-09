#[cfg(test)]
mod tests;

use camino::{Utf8Path, Utf8PathBuf};
use clap::Parser;
use common::{
    dir::remove_content_of_dir,
    out::{write_checksummed_variants, OutputParams, VariantOutputParams},
    setup_logging,
};
use fs_err as fs;
use std::process::ExitCode;
use unic_langid::LanguageIdentifier;

fn main() -> ExitCode {
    let args = Cli::parse();
    run(args);
    ExitCode::SUCCESS
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long, value_name = "DIR-PATH")]
    input_dir: Utf8PathBuf,

    #[arg(short = 'x', long, value_name = "FILE-EXT")]
    /// File extensions that should be processed when searching for files in the input directory
    file_extension: String,

    #[arg(short, long, value_name = "DIR-PATH")]
    /// Folder where the output files should be written
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

impl VariantOutputParams for Cli {
    fn output_dir(&self) -> &Utf8Path {
        &self.output_dir
    }
    fn file_extension(&self) -> &str {
        &self.file_extension
    }
}

fn run(cli: Cli) {
    setup_logging(cli.verbose);

    log::info!("Running builder-localized");

    remove_content_of_dir(&cli.output_dir);

    if !cli.output_dir.exists() {
        fs::create_dir_all(&cli.output_dir).unwrap();
    }

    let variants = get_variants(&cli);

    write_checksummed_variants(&cli, &variants);
}

fn get_variants(cli: &Cli) -> Vec<(String, Vec<u8>)> {
    let mut variants: Vec<(String, Vec<u8>)> = Vec::new();

    // list all file names in folder
    for file in cli.input_dir.read_dir_utf8().unwrap() {
        let file = file.unwrap();
        let file_type = file.file_type().unwrap();

        let file_extension_match = file
            .path()
            .extension()
            .map(|ext| ext == cli.file_extension)
            .unwrap_or_default();

        if file_type.is_file() && file_extension_match {
            let loc = file.path().file_stem().unwrap();
            let langid: LanguageIdentifier = loc.parse().unwrap();
            let content = fs::read(file.path()).unwrap();
            variants.push((langid.to_string(), content));
        }
    }

    variants.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));

    variants
}
