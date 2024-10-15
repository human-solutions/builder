use std::fmt::Display;

use camino::{Utf8Path, Utf8PathBuf};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Encoding {
    Brotli,
    Gzip,
    Identity,
}

impl Display for Encoding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Encoding {
    pub fn as_str(&self) -> &str {
        match self {
            Encoding::Brotli => "br",
            Encoding::Gzip => "gzip",
            Encoding::Identity => "",
        }
    }

    pub fn add_encoding(&self, path: &Utf8Path) -> Utf8PathBuf {
        if *self == Encoding::Identity {
            path.to_path_buf()
        } else {
            let ext = path.extension().unwrap_or_default();
            path.with_extension(format!("{ext}.{self}"))
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Output {
    /// Folder where the output files should be written
    pub dir: Utf8PathBuf,

    brotli: bool,

    gzip: bool,

    uncompressed: bool,

    /// Overrides the encoding settings and writes all possible encodings
    all_encodings: bool,

    pub checksum: bool,
}

impl Output {
    pub fn new<P: Into<Utf8PathBuf>>(dir: P) -> Self {
        Self {
            dir: dir.into(),
            brotli: false,
            gzip: false,
            uncompressed: false,
            all_encodings: false,
            checksum: false,
        }
    }

    pub fn new_compress_and_sum<P: Into<Utf8PathBuf>>(dir: P) -> Self {
        Self {
            dir: dir.into(),
            brotli: true,
            gzip: true,
            uncompressed: true,
            all_encodings: true,
            checksum: true,
        }
    }

    pub fn uncompressed(&self) -> bool {
        // if none are set, then default to uncompressed
        let default_uncompressed = !self.uncompressed && !self.brotli && !self.gzip;
        self.uncompressed || default_uncompressed || self.all_encodings
    }

    pub fn brotli(&self) -> bool {
        self.brotli || self.all_encodings
    }

    pub fn gzip(&self) -> bool {
        self.gzip || self.all_encodings
    }

    pub fn encodings(&self) -> Vec<Encoding> {
        let mut encodings = Vec::new();
        if self.gzip() {
            encodings.push(Encoding::Gzip);
        }
        if self.brotli() {
            encodings.push(Encoding::Brotli);
        }
        if self.uncompressed() {
            encodings.push(Encoding::Identity);
        }
        encodings
    }
}
