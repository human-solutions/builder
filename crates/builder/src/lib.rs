mod config;
mod ext;
mod generate;
mod install;
mod postbuild;
mod prebuild;
mod util;

pub use config::{CmdArgs, Commands, Config};
pub use ext::anyhow;
use fs_err::File;
use log::LevelFilter;
use simplelog::{
    ColorChoice, CombinedLogger, Config as LogConfig, TermLogger, TerminalMode, WriteLogger,
};

pub fn setup_logging(log_path: &str) {
    let _ = CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Info,
            LogConfig::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(
            LevelFilter::Info,
            LogConfig::default(),
            File::create(log_path).unwrap(),
        ),
    ]);
}
