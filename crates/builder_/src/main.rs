mod builder;
mod types;

use std::process::ExitCode;

use anyhow::Result;
use cargo_metadata::camino::Utf8PathBuf;
use clap::Parser;

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
    let args = BuilderArgs::parse();

    builder::run(args)
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct BuilderArgs {
    #[arg(long, env = "CARGO_MANIFEST_DIR")]
    pub dir: Utf8PathBuf,
    #[arg(long, env = "PROFILE")]
    pub profile: String,
    #[arg(long, env = "CARGO_PKG_NAME")]
    pub package: String,
}
