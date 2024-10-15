use log::LevelFilter;
use simplelog::{
    format_description, ColorChoice, Config as LogConfig, ConfigBuilder, TermLogger, TerminalMode,
};
use std::env;

pub fn setup_logging(verbose: bool) {
    let log_level = if verbose {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };

    let log_config = if env::var("CARGO").is_ok() && verbose {
        ConfigBuilder::new()
            .set_time_format_custom(format_description!(
                "cargo::warning=[hour]:[minute]:[second]"
            ))
            .build()
    } else {
        LogConfig::default()
    };

    let _ = TermLogger::init(
        log_level,
        log_config,
        TerminalMode::Mixed,
        ColorChoice::Auto,
    );
}
