pub mod dir;
mod envargs;
mod ext;
mod log;
pub mod out;

use std::sync::OnceLock;

pub use envargs::CargoEnv;
pub use ext::{RustNaming, Utf8PathExt};
pub use log::setup_logging;

pub static RELEASE: OnceLock<bool> = OnceLock::new();

pub fn is_release() -> bool {
    RELEASE.get().map(|b| *b).unwrap_or(false)
}
