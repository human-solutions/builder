mod config;
mod ext;
mod generate;
mod install;
mod postbuild;
mod prebuild;
mod task;
mod util;

pub use config::{CmdArgs, Commands, Config};
pub use ext::anyhow;
use fs_err::File;
use log::LevelFilter;
use simplelog::{
    ColorChoice, CombinedLogger, Config as LogConfig, TermLogger, TerminalMode, WriteLogger,
};

pub fn setup_logging(log_file: &str, log_level: LevelFilter) {
    const LOG_TO_TERM_AND_FILE: bool = false;

    if LOG_TO_TERM_AND_FILE {
        let _ = CombinedLogger::init(vec![
            TermLogger::new(
                log_level,
                LogConfig::default(),
                TerminalMode::Mixed,
                ColorChoice::Auto,
            ),
            WriteLogger::new(
                LevelFilter::Info,
                LogConfig::default(),
                File::create(log_file).unwrap(),
            ),
        ]);
    } else {
        let _ = WriteLogger::init(
            log_level,
            LogConfig::default(),
            File::create(log_file).unwrap(),
        );
    }
}
