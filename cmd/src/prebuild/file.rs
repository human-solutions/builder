use crate::generate::Output;

use camino::Utf8PathBuf;
use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
pub struct File {
    pub path: Utf8PathBuf,
    pub out: Output,
}

impl File {
    pub fn url(&self, checksum: Option<String>) -> String {
        let filename = self.path.file_name().unwrap();
        self.out.url(filename, checksum)
    }
}
