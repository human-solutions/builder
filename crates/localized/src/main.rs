mod asset;
mod generator;
#[cfg(test)]
mod tests;

use asset::Asset;
use camino::{Utf8Path, Utf8PathBuf};
use clap::Parser;
use common::{
    dir::remove_content_of_dir,
    out::{write_checksummed_variants, OutputParams},
    setup_logging,
};
use fs_err as fs;
use generator::Generator;
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

    #[arg(short, long, value_name = "FILE-PATH")]
    /// Where the generated code should be written
    generate_code: Utf8PathBuf,

    #[arg(long)]
    brotli: bool,

    #[arg(long)]
    gzip: bool,

    #[arg(long)]
    uncompressed: bool,

    #[arg(long)]
    checksum: bool,

    #[arg(short, long)]
    verbose: bool,
}

impl OutputParams for Cli {
    fn brotli(&self) -> bool {
        self.brotli
    }
    fn gzip(&self) -> bool {
        self.gzip
    }
    fn uncompressed(&self) -> bool {
        self.uncompressed
    }
    fn checksum(&self) -> bool {
        self.checksum
    }
    fn output_dir(&self) -> &Utf8Path {
        &self.output_dir
    }
    fn file_extension(&self) -> &str {
        &self.file_extension
    }
}

impl Cli {
    pub fn input_dir_name(&self) -> String {
        self.input_dir.iter().last().unwrap().to_string()
    }

    pub fn url(&self, checksum: Option<String>) -> String {
        let ext = &self.file_extension;
        let name = self.input_dir_name();
        let sum = checksum.as_deref().unwrap_or_default();
        format!("{sum}{name}.{ext}")
    }
}

fn run(cli: Cli) {
    setup_logging(cli.verbose);

    remove_content_of_dir(&cli.output_dir);
    if !cli.output_dir.exists() {
        fs::create_dir_all(&cli.output_dir).unwrap();
    }

    let variants = get_variants(&cli);
    let localizations = variants.iter().map(|(lang, _)| lang.clone()).collect();

    let hash = write_checksummed_variants(&cli, &variants);

    let generator = &mut Generator::default();
    generator.add_asset(
        Asset::from_localized(&cli, hash, localizations),
        cli.generate_code.clone(),
    );
    generator.write(&cli);
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
