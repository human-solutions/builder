use camino_fs::Utf8PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SwiftPackageCmd {
    pub manifest_dir: Utf8PathBuf,
    pub release: bool,
}

impl SwiftPackageCmd {
    pub fn new<P1: Into<Utf8PathBuf>>(manifest_dir: P1) -> Self {
        Self {
            manifest_dir: manifest_dir.into(),
            release: false,
        }
    }

    pub fn release(mut self, release: bool) -> Self {
        self.release = release;
        self
    }
}
