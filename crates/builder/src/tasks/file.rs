use crate::generate::{Asset, Generator, Output};
use fs_err as fs;
use std::collections::HashSet;

use anyhow::Result;
use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};

use super::{BuildStep, Config};

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct FilesParams {
    pub path: Utf8PathBuf,
    pub out: Output,
}

impl FilesParams {
    pub fn url(&self, checksum: Option<String>) -> String {
        let filename = self.path.file_name().unwrap();
        self.out.url(filename, checksum)
    }

    pub fn process(
        &self,
        config: &Config,
        phase: &BuildStep,
        generator: &mut Generator,
        watched: &mut HashSet<String>,
    ) -> Result<()> {
        let path = config.args.dir.join(&self.path);
        let contents = fs::read(&path)?;
        let filename = self.path.file_name().unwrap();
        let hash = self
            .out
            .write_file(&contents, &config.site_dir("files", phase), filename)?;

        generator.add_asset(Asset::from_file(self, hash), None);
        watched.insert(self.path.to_string());

        Ok(())
    }
}
