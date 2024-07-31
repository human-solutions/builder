use builder::Commands;
use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    match Cli::parse().command.run() {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("builder error: {e:?}");
            ExitCode::FAILURE
        }
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}
