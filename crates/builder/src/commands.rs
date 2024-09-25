use fs_err as fs;
use log::LevelFilter;
use serde::{Deserialize, Serialize};

use crate::tasks::{BuildStep, Setup};
use crate::{ext::anyhow::Result, setup_logging};
use camino::Utf8PathBuf;
use clap::{Args, Subcommand};

#[derive(Args, Debug, Clone, Serialize, Deserialize)]
pub struct CmdArgs {
    #[clap(long, env = "CARGO_MANIFEST_DIR")]
    pub dir: Utf8PathBuf,
    #[clap(long, env = "PROFILE")]
    pub profile: String,
    #[clap(long, env = "CARGO_PKG_NAME")]
    pub package: String,
    #[clap(long, env = "TARGET")]
    pub target: String,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Prebuild(CmdArgs),
    Postbuild(CmdArgs),
}

impl Commands {
    pub fn run(&self) -> Result<()> {
        let (args, step) = match self {
            Self::Prebuild(args) => (args, BuildStep::Prebuild),
            Self::Postbuild(args) => (args, BuildStep::Postbuild),
        };

        let setup = Setup::new(args)?;

        let log_path = setup
            .config
            .package_target_dir(&setup.config.package_name, &step);
        fs::create_dir_all(&log_path)?;

        let log_file = log_path.join(format!("{}-{}.log", step.as_str(), args.profile));
        setup_logging(log_file.as_str(), LevelFilter::Debug);
        log::info!("Args: {args:?}");

        setup.run(step)
    }
}
