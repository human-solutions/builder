use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AssembleCmd {
    /// Input sfd file path
    pub localized: Vec<Utf8PathBuf>,

    /// Prefix added to the URL. It should end with a slash.
    pub url_prefix: String,

    /// Path to one of the asset files. If there are other
    /// versions of it (e.g. compressed), then they'll be
    /// automatically detected.
    pub files: Vec<Utf8PathBuf>,

    pub wasm_dir: Option<Utf8PathBuf>,

    /// Where to write the generated code.
    pub code_file: Option<Utf8PathBuf>,

    /// Where to write a rust file with the environment variables
    pub url_env_file: Option<Utf8PathBuf>,
}

impl AssembleCmd {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn write_generated_code_to<P: Into<Utf8PathBuf>>(mut self, out_file: P) -> Self {
        self.code_file = Some(out_file.into());
        self
    }

    pub fn write_url_envs_to<P: Into<Utf8PathBuf>>(mut self, rs_file: P) -> Self {
        self.url_env_file = Some(rs_file.into());
        self
    }

    pub fn url_prefix<S: AsRef<str>>(mut self, url_prefix: S) -> Self {
        self.url_prefix = url_prefix.as_ref().into();
        self
    }

    pub fn wasm_dir<P: Into<Utf8PathBuf>>(mut self, wasm_dir: P) -> Self {
        self.wasm_dir = Some(wasm_dir.into());
        self
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
