use std::{fs::File, io::Read, process::ExitCode};

use base64::{engine::general_purpose::URL_SAFE, Engine};
use camino::Utf8PathBuf;
use clap::Parser;
use seahash::hash;

fn main() -> ExitCode {
    let args = Args::parse();

    let Ok(mut file) = File::open(&args.path) else {
        eprintln!("Failed to open file: {}", args.path);
        return ExitCode::FAILURE;
    };

    let mut buffer = Vec::new();

    let Ok(_) = file.read_to_end(&mut buffer) else {
        eprintln!("Failed to read file: {}", args.path);
        return ExitCode::FAILURE;
    };

    let h = hash(&buffer);

    println!("{}", URL_SAFE.encode(h.to_be_bytes()));

    ExitCode::SUCCESS
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    path: Utf8PathBuf,
}
