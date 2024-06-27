use builder::anyhow::Result;
use builder::{PostbuildArgs, PostbuildManifest, PrebuildArgs, PrebuildManifest, RawPrebuildArgs};
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
        Commands::Prebuild(info) => {
            let info: PrebuildArgs = info.try_into()?;
            let manifest = PrebuildManifest::try_parse(&info)?;
            manifest.process(&info)
        }
        Commands::Postbuild(info) => {
            let manifest = PostbuildManifest::try_parse(&info)?;
            manifest.process(&info)
        }
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
    Prebuild(RawPrebuildArgs),
    Postbuild(PostbuildArgs),
}
