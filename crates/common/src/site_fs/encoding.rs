use crate::{debug, is_release, warn};
use std::{
    io::{Cursor, Write},
    str::FromStr,
};

use anyhow::Result;
use brotli::{enc::BrotliEncoderParams, BrotliCompress};
use builder_command::{Encoding, Output};
use camino_fs::*;
use flate2::{Compression, GzBuilder};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AssetEncodings {
    pub brotli: bool,
    pub gzip: bool,
    pub uncompressed: bool,
}

impl FromStr for AssetEncodings {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        let mut encodings = Self::default();
        for enc in s.split(',') {
            encodings.add_encoding(enc);
        }
        Ok(encodings)
    }
}

impl AssetEncodings {
    pub fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Encoding>,
    {
        let mut encodings = Self::default();
        for enc in iter {
            match enc {
                Encoding::Brotli => encodings.brotli = true,
                Encoding::Gzip => encodings.gzip = true,
                Encoding::Identity => encodings.uncompressed = true,
            }
        }
        encodings
    }
    pub fn from_output(output: &Output) -> Self {
        Self {
            brotli: output.brotli(),
            gzip: output.gzip(),
            uncompressed: output.uncompressed(),
        }
    }

    pub fn all() -> Self {
        Self {
            brotli: true,
            gzip: true,
            uncompressed: true,
        }
    }

    pub fn uncompressed() -> Self {
        Self {
            brotli: false,
            gzip: false,
            uncompressed: true,
        }
    }

    pub fn add_uncompressed(&mut self) {
        self.uncompressed = true;
    }

    pub fn add_encoding(&mut self, enc: &str) {
        match enc {
            "br" => self.brotli = true,
            "gzip" => self.gzip = true,
            _ => warn!("invalid encoding: {enc}"),
        }
    }

    pub fn write(&self, path: &Utf8Path, bytes: &[u8]) -> Result<()> {
        if let Some(dir) = path.parent() {
            dir.mkdirs()?;
        }
        for enc in *self {
            enc.write(path, bytes, is_release())?;
        }
        Ok(())
    }
    pub fn join(&mut self, other: &Self) {
        self.brotli |= other.brotli;
        self.gzip |= other.gzip;
        self.uncompressed |= other.uncompressed;
    }

    pub fn is_empty(&self) -> bool {
        !self.brotli && !self.gzip && !self.uncompressed
    }

    pub fn len(&self) -> usize {
        let mut len = 0;
        if self.brotli {
            len += 1;
        }
        if self.gzip {
            len += 1;
        }
        if self.uncompressed {
            len += 1;
        }
        len
    }
}

impl IntoIterator for AssetEncodings {
    type Item = Encoding;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        let mut vec = Vec::new();
        if self.brotli {
            vec.push(Encoding::Brotli);
        }
        if self.gzip {
            vec.push(Encoding::Gzip);
        }
        if self.uncompressed {
            vec.push(Encoding::Identity);
        }
        vec.into_iter()
    }
}

pub trait EncodedWrite {
    fn write(&self, path: &Utf8Path, contents: &[u8], release: bool) -> Result<()>;
}

impl EncodedWrite for Encoding {
    fn write(&self, path: &Utf8Path, contents: &[u8], relase: bool) -> Result<()> {
        let path = self.add_encoding(path);
        debug!("Writing file '{:?}'", path);

        let encoded = match self {
            Encoding::Brotli => brotli(contents, relase),
            Encoding::Gzip => gzip(contents, relase),
            Encoding::Identity => {
                path.write(contents)?;
                return Ok(());
            }
        };

        path.write(encoded)?;
        Ok(())
    }
}

fn brotli(contents: &[u8], release: bool) -> Vec<u8> {
    let mut cursor = Cursor::new(&contents);

    let quality = if release { 10 } else { 1 };

    let params = BrotliEncoderParams {
        quality,
        ..Default::default()
    };
    let mut bytes = vec![];
    BrotliCompress(&mut cursor, &mut bytes, &params).unwrap();
    bytes
}

fn gzip(contents: &[u8], release: bool) -> Vec<u8> {
    let mut bytes = vec![];
    let compression = if release {
        Compression::best()
    } else {
        Compression::fast()
    };
    let mut gz = GzBuilder::new().write(&mut bytes, compression);
    gz.write_all(contents).unwrap();
    gz.finish().unwrap();
    bytes
}
