use std::ffi::OsStr;

use clap::ValueEnum;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, StackerError>;

#[derive(Debug, Clone, ValueEnum)]
pub enum Format {
    Minified,
    Pretty,
}

pub enum StyleExtension {
    Css,
    Scss,
    Sass,
}

impl StyleExtension {
    pub fn from_os_str(os_str: Option<&OsStr>) -> Option<Self> {
        os_str.and_then(|os_str| {
            os_str
                .to_str()
                .and_then(|s| match s.to_lowercase().as_str() {
                    "css" => Some(Self::Css),
                    "scss" => Some(Self::Scss),
                    "sass" => Some(Self::Sass),
                    _ => None,
                })
        })
    }
}

#[derive(Debug, Error)]
pub enum StackerError {
    #[error("Failed to collect style files : {0}")]
    Collect(String),
    #[error("Failed to process SASS : {0}")]
    Sass(String),
    #[error("Failed to process CSS : {0}")]
    Stylesheet(String),
    #[error("Failed to save styles : {0}")]
    Save(String),
}
