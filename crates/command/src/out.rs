use camino_fs::*;
use std::{fmt::Display, str::FromStr};

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
    pub fn name(&self) -> &'static str {
        match self {
            Encoding::Brotli => "Brotli",
            Encoding::Gzip => "Gzip",
            Encoding::Identity => "Identity",
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Encoding::Brotli => "br",
            Encoding::Gzip => "gzip",
            Encoding::Identity => "",
        }
    }

    pub fn add_encoding(&self, path: &Utf8Path) -> Utf8PathBuf {
        if let Some(enc) = self.file_ending() {
            let ext = path.extension().unwrap_or_default();
            if !ext.ends_with(enc) {
                return path.with_extension(format!("{ext}.{enc}"));
            }
        }
        path.to_path_buf()
    }

    pub fn file_ending(&self) -> Option<&str> {
        match self {
            Encoding::Brotli => Some("br"),
            Encoding::Gzip => Some("gzip"),
            Encoding::Identity => None,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct Output {
    /// Folder where the output files should be written
    pub dir: Utf8PathBuf,

    pub site_dir: Option<Utf8PathBuf>,

    brotli: bool,

    gzip: bool,

    uncompressed: bool,

    /// Overrides the encoding settings and writes all possible encodings
    all_encodings: bool,

    pub checksum: bool,

    /// Optional path to write file hashes as a Rust file
    pub hash_output_path: Option<Utf8PathBuf>,
}

impl Output {
    pub fn new<P: Into<Utf8PathBuf>>(dir: P) -> Self {
        Self {
            dir: dir.into(),
            site_dir: None,
            brotli: false,
            gzip: false,
            uncompressed: false,
            all_encodings: false,
            checksum: false,
            hash_output_path: None,
        }
    }

    pub fn new_compress_and_sum<P: Into<Utf8PathBuf>>(dir: P) -> Self {
        Self {
            dir: dir.into(),
            site_dir: None,
            brotli: true,
            gzip: true,
            uncompressed: true,
            all_encodings: true,
            checksum: true,
            hash_output_path: None,
        }
    }

    pub fn new_compress<P: Into<Utf8PathBuf>>(dir: P) -> Self {
        Self {
            dir: dir.into(),
            site_dir: None,
            brotli: true,
            gzip: true,
            uncompressed: true,
            all_encodings: true,
            checksum: false,
            hash_output_path: None,
        }
    }

    pub fn site_dir<P: Into<Utf8PathBuf>>(mut self, dir: P) -> Self {
        self.site_dir = Some(dir.into());
        self
    }

    pub fn hash_output_path<P: Into<Utf8PathBuf>>(mut self, path: P) -> Self {
        self.hash_output_path = Some(path.into());
        self
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

impl Display for Output {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "dir={}\t", self.dir)?;
        if let Some(site_dir) = &self.site_dir {
            write!(f, "site_dir={}\t", site_dir)?;
        }
        write!(f, "brotli={}\t", self.brotli)?;
        write!(f, "gzip={}\t", self.gzip)?;
        write!(f, "uncompressed={}\t", self.uncompressed)?;
        write!(f, "all_encodings={}\t", self.all_encodings)?;
        write!(f, "checksum={}\t", self.checksum)?;
        if let Some(hash_output_path) = &self.hash_output_path {
            write!(f, "hash_output_path={}\t", hash_output_path)?;
        }
        Ok(())
    }
}

impl FromStr for Output {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut cmd = Output::default();
        for item in s.split('\t').filter(|s| !s.is_empty()) {
            let (key, value) = item.split_once('=').unwrap();

            match key {
                "dir" => cmd.dir = value.into(),
                "site_dir" => cmd.site_dir = Some(value.into()),
                "brotli" => cmd.brotli = value.parse().unwrap(),
                "gzip" => cmd.gzip = value.parse().unwrap(),
                "uncompressed" => cmd.uncompressed = value.parse().unwrap(),
                "all_encodings" => cmd.all_encodings = value.parse().unwrap(),
                "checksum" => cmd.checksum = value.parse().unwrap(),
                "hash_output_path" => cmd.hash_output_path = Some(value.into()),
                _ => panic!("unknown key: {}", key),
            }
        }
        Ok(cmd)
    }
}
