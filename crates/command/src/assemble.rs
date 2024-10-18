use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AssembleCmd {
    /// Input sfd file path
    pub localized: Vec<Utf8PathBuf>,

    /// Prefix added to the URL. It should end with a slash.
    pub url_prefix: String,

    /// Path to one of the asset files. If there are other
    /// versions of it (e.g. compressed), then they'll be
    /// automatically detected.
    pub files: Vec<Utf8PathBuf>,

    /// Where to write the generated code.
    pub out_file: Utf8PathBuf,
}

impl AssembleCmd {
    pub fn new<P: Into<Utf8PathBuf>, S: AsRef<str>>(out_file: P, url_prefix: S) -> Self {
        Self {
            localized: Vec::new(),
            url_prefix: url_prefix.as_ref().into(),
            files: Vec::new(),
            out_file: out_file.into(),
        }
    }

    pub fn add_localized<P: Into<Utf8PathBuf>>(mut self, localized: P) -> Self {
        self.localized.push(localized.into());
        self
    }

    pub fn add_file<P: Into<Utf8PathBuf>>(mut self, file: P) -> Self {
        self.files.push(file.into());
        self
    }
}
