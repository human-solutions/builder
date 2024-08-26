use crate::generate::Output;

use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct FileParams {
    pub path: Utf8PathBuf,
    pub out: Output,
}

impl FileParams {
    pub fn url(&self, checksum: Option<String>) -> String {
        let filename = self.path.file_name().unwrap();
        self.out.url(filename, checksum)
    }
}
