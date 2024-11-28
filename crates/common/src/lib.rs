mod envargs;
mod ext;
pub mod out;
pub mod site_fs;

use log::LevelFilter;
use simplelog::{
    format_description, ColorChoice, Config as LogConfig, ConfigBuilder, TermLogger, TerminalMode,
};
use std::env;
use std::sync::OnceLock;

pub use envargs::CargoEnv;
pub use ext::RustNaming;

pub static RELEASE: OnceLock<bool> = OnceLock::new();
pub static VERBOSE: OnceLock<bool> = OnceLock::new();

pub fn is_release() -> bool {
    RELEASE.get().map(|b| *b).unwrap_or(false)
}
pub fn is_verbose() -> bool {
    VERBOSE.get().map(|b| *b).unwrap_or(false)
}

#[allow(unused_imports)]
#[cfg(not(test))]
use log::{debug, info, warn};

#[allow(unused_imports)]
#[cfg(test)]
use std::{println as info, println as warn, println as debug}; // Workaround to use prinltn! for logs.

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
