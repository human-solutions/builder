mod envargs;
mod ext;
pub mod out;
pub mod site_fs;

use builder_command::LogLevel;
use simplelog::{ColorChoice, ConfigBuilder, TermLogger, TerminalMode, format_description};
use std::sync::OnceLock;
use time::OffsetDateTime;

pub use envargs::CargoEnv;
pub use ext::RustNaming;

pub static RELEASE: OnceLock<bool> = OnceLock::new();
pub static LOG_LEVEL: OnceLock<LogLevel> = OnceLock::new();

pub fn is_release() -> bool {
    RELEASE.get().copied().unwrap_or(false)
}

pub fn log_level() -> LogLevel {
    LOG_LEVEL.get().copied().unwrap_or(LogLevel::Normal)
}

pub fn is_verbose() -> bool {
    matches!(log_level(), LogLevel::Verbose | LogLevel::Trace)
}

pub fn is_trace() -> bool {
    matches!(log_level(), LogLevel::Trace)
}

#[allow(unused_imports)]
#[cfg(not(test))]
use log::{debug, info, warn};

#[allow(unused_imports)]
#[cfg(test)]
use std::{println as info, println as warn, println as debug}; // Workaround to use prinltn! for logs.

pub struct Timer {
    start: OffsetDateTime,
    name: String,
}

impl Timer {
    pub fn new(name: &str) -> Self {
        Self {
            start: OffsetDateTime::now_utc(),
            name: name.to_string(),
        }
    }

    pub fn elapsed_ms(&self) -> i64 {
        (OffsetDateTime::now_utc() - self.start)
            .whole_milliseconds()
            .try_into()
            .unwrap_or(i64::MAX)
    }

    pub fn log_completion(&self) {
        let elapsed = self.elapsed_ms();
        log::info!(
            "Completed {} ({}.{}s)",
            self.name,
            elapsed / 1000,
            elapsed % 1000 / 100
        );
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        if is_verbose() {
            self.log_completion();
        }
    }
}

#[macro_export]
macro_rules! log_command {
    ($cmd:expr, $msg:expr) => {
        log::info!("[{}] {}", $cmd, $msg)
    };
    ($cmd:expr, $fmt:expr, $($arg:tt)*) => {
        log::info!("[{}] {}", $cmd, format!($fmt, $($arg)*))
    };
}

#[macro_export]
macro_rules! log_operation {
    ($cmd:expr, $msg:expr) => {
        log::debug!("[{}] {}", $cmd, $msg)
    };
    ($cmd:expr, $fmt:expr, $($arg:tt)*) => {
        log::debug!("[{}] {}", $cmd, format!($fmt, $($arg)*))
    };
}

#[macro_export]
macro_rules! log_trace {
    ($cmd:expr, $msg:expr) => {
        log::trace!("[{}] {}", $cmd, $msg)
    };
    ($cmd:expr, $fmt:expr, $($arg:tt)*) => {
        log::trace!("[{}] {}", $cmd, format!($fmt, $($arg)*))
    };
}

/// Log a warning that should use cargo::warning when running under cargo
#[macro_export]
macro_rules! warn_cargo {
    ($fmt:expr, $($arg:tt)*) => {
        if std::env::var("CARGO").is_ok() {
            println!("cargo::warning={}", format!($fmt, $($arg)*));
        } else {
            log::warn!($fmt, $($arg)*);
        }
    };
    ($msg:expr) => {
        if std::env::var("CARGO").is_ok() {
            println!("cargo::warning={}", $msg);
        } else {
            log::warn!("{}", $msg);
        }
    };
}

pub fn setup_logging(level: LogLevel) {
    let log_level = level.to_level_filter();

    let mut log_config_builder = ConfigBuilder::new();

    // Filter out walrus debug logs when in verbose mode
    if matches!(level, LogLevel::Verbose | LogLevel::Trace) {
        log_config_builder.add_filter_ignore_str("walrus");
        log_config_builder.add_filter_ignore_str("wasm_bindgen_cli_support");
    }

    let log_config = log_config_builder
        .set_time_format_custom(format_description!("[hour]:[minute]:[second]"))
        .build();

    let _ = TermLogger::init(
        log_level,
        log_config,
        TerminalMode::Mixed,
        ColorChoice::Never,
    );
}
