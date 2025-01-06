use std::{convert::Infallible, fmt::Display, str::FromStr};

use camino_fs::Utf8PathBuf;

#[derive(Debug, Default, PartialEq, Eq)]
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

impl Display for SwiftPackageCmd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "manifest_dir={}", self.manifest_dir)?;
        writeln!(f, "release={}", self.release)
    }
}

impl FromStr for SwiftPackageCmd {
    type Err = Infallible;

    fn from_str(_s: &str) -> Result<Self, Self::Err> {
        let mut cmd = SwiftPackageCmd::default();

        for line in _s.lines() {
            let (key, value) = line.split_once('=').unwrap();
            match key {
                "manifest_dir" => cmd.manifest_dir = value.into(),
                "release" => cmd.release = value.parse().unwrap(),
                _ => panic!("unexpected key: {}", key),
            }
        }
        Ok(cmd)
    }
}
