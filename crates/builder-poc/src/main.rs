use anyhow::Result;
use builder_poc::{CmdArgs, Config};
use clap::{Parser, Subcommand};
use std::process::ExitCode;

fn main() -> ExitCode {
    match try_main() {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("builder error: {e:?}");
            ExitCode::FAILURE
        }
    }
}

fn try_main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Prebuild(info) => Config::from_path(info)?.run_prebuild(),
        Commands::Postbuild(info) => Config::from_path(info)?.run_postbuild(),
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Prebuild(CmdArgs),
    Postbuild(CmdArgs),
}
