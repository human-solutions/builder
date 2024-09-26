mod commands;
mod ext;
mod generate;
mod install;
mod tasks;
mod util;

use std::env;

pub use commands::{CmdArgs, Commands};
pub use ext::anyhow;
use fs_err::File;
use log::LevelFilter;
use simplelog::{
    format_description, ColorChoice, CombinedLogger, Config as LogConfig, ConfigBuilder,
    TermLogger, TerminalMode, WriteLogger,
};

pub fn setup_logging(log_file: &str, log_level: LevelFilter) {
    const LOG_TO_TERM_AND_FILE: bool = true;

    if LOG_TO_TERM_AND_FILE {
        let log_config = if env::var("BUILDER_LOG_AS_CARGO_WARNING").is_ok() {
            ConfigBuilder::new()
                .set_time_format_custom(format_description!(
                    "cargo::warning=[hour]:[minute]:[second]"
                ))
                .build()
        } else {
            LogConfig::default()
        };

        let _ = CombinedLogger::init(vec![
            TermLogger::new(
                log_level,
                log_config,
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
