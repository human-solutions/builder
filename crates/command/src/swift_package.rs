use builder_mtimes::{InputFiles, OutputFiles};
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

impl InputFiles for SwiftPackageCmd {
    fn input_files(&self) -> Vec<Utf8PathBuf> {
        vec![self.manifest_dir.join("Cargo.toml")]
    }
}

impl OutputFiles for SwiftPackageCmd {
    fn output_files(&self) -> Vec<Utf8PathBuf> {
        let profile = if self.release { "release" } else { "debug" };
        vec![self.manifest_dir.join("target").join(profile)]
    }
}

impl crate::CommandMetadata for SwiftPackageCmd {
    fn output_dir(&self) -> &camino_fs::Utf8Path {
        &self.manifest_dir
    }

    fn name(&self) -> &'static str {
        "swift-package"
    }
}
