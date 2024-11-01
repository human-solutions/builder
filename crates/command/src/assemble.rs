use camino_fs::Utf8PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AssembleCmd {
    pub site_root: Utf8PathBuf,
    pub include_names: Vec<String>,

    /// Where to write the generated code.
    pub code_file: Option<Utf8PathBuf>,

    /// Where to write a rust file with the environment variables
    pub url_env_file: Option<Utf8PathBuf>,
}

impl AssembleCmd {
    pub fn new<P: Into<Utf8PathBuf>>(site_root: P) -> Self {
        Self {
            site_root: site_root.into(),
            ..Default::default()
        }
    }

    pub fn write_generated_code_to<P: Into<Utf8PathBuf>>(mut self, out_file: P) -> Self {
        self.code_file = Some(out_file.into());
        self
    }

    pub fn write_url_envs_to<P: Into<Utf8PathBuf>>(mut self, rs_file: P) -> Self {
        self.url_env_file = Some(rs_file.into());
        self
    }

    pub fn add_include_name<S: AsRef<str>>(mut self, name: S) -> Self {
        self.include_names.push(name.as_ref().into());
        self
    }
    pub fn include_names<I: IntoIterator<Item = S>, S: AsRef<str>>(mut self, names: I) -> Self {
        self.include_names
            .extend(names.into_iter().map(|s| s.as_ref().into()));
        self
    }
}
