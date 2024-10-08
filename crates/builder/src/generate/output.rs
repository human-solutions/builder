use crate::anyhow::Result;
use crate::ext::{ByteVecExt, Utf8PathExt};
use base64::{engine::general_purpose::URL_SAFE, Engine};
use brotli::{enc::BrotliEncoderParams, BrotliCompress};
use camino::{Utf8Path, Utf8PathBuf};
use flate2::{Compression, GzBuilder};
use fs_err as fs;
use seahash::SeaHasher;
use serde::{Deserialize, Serialize};
use std::hash::Hasher;
use std::io::{Cursor, Write};
use unic_langid::LanguageIdentifier;

#[derive(Default, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Output {
    pub brotli: bool,
    pub gzip: bool,
    pub uncompressed: bool,
    pub checksum: bool,
    /// sub-folder in generated site
    pub folder: Option<Utf8PathBuf>,
}

impl Output {
    /// Encodings according to https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Accept-Encoding
    pub fn encodings(&self) -> Vec<String> {
        let mut encodings = vec![];
        if self.brotli {
            encodings.push("br".to_string());
        }
        if self.gzip {
            encodings.push("gzip".to_string());
        }
        if self.uncompressed {
            encodings.push("identity".to_string());
        }
        encodings
    }

    pub fn url(&self, filename: &str, checksum: Option<String>) -> String {
        let folder = if let Some(folder) = self.folder.as_ref() {
            format!("/{folder}")
        } else {
            "".to_string()
        };
        format!("{folder}/{}{filename}", checksum.unwrap_or_default(),)
    }

    pub fn path(
        &self,
        hash: &Option<String>,
        dir: &Utf8Path,
        filename: &str,
    ) -> Result<Utf8PathBuf> {
        let prefix = hash.as_deref().unwrap_or_default();
        let dir = self.full_created_dir(dir)?;
        let filename = format!("{prefix}{filename}");
        let path = dir.join(filename);
        Ok(path)
    }

    pub fn write_file(
        &self,
        contents: &[u8],
        dir: &Utf8Path,
        filename: &str,
    ) -> Result<Option<String>> {
        let hash = self.checksum.then(|| contents.base64_checksum());
        let path = self.path(&hash, dir, filename)?;
        self.compress_and_write(contents, &path)?;

        Ok(hash)
    }

    fn full_created_dir(&self, dir: &Utf8Path) -> Result<Utf8PathBuf> {
        let dir = if let Some(folder) = &self.folder {
            dir.join(folder)
        } else {
            dir.to_path_buf()
        };
        if !dir.exists() {
            fs::create_dir_all(&dir)?;
        }
        Ok(dir)
    }

    fn compress_and_write(&self, contents: &[u8], path: &Utf8Path) -> Result<()> {
        // if none are set, then default to uncompressed
        let default_uncompressed = !self.uncompressed && !self.brotli && !self.gzip;

        if self.uncompressed || default_uncompressed {
            log::info!("Writing uncompressed file '{:?}'", path);
            fs::write(path, contents)?;
        }
        if self.brotli {
            let path = path.push_ext("br");
            log::info!("Writing brotli file '{:?}'", path);

            let mut file = fs::File::create(path)?;
            let mut cursor = Cursor::new(&contents);

            let params = BrotliEncoderParams {
                quality: 10,
                ..Default::default()
            };
            BrotliCompress(&mut cursor, &mut file, &params)?;
        }

        if self.gzip {
            let path = path.push_ext("gz");
            log::info!("Writing gzip file '{:?}'", path);

            let f = fs::File::create(path)?;
            let mut gz = GzBuilder::new().write(f, Compression::default());
            gz.write_all(contents)?;
            gz.finish()?;
        }
        Ok(())
    }

    pub fn write_localized(
        &self,
        dir: &Utf8Path,
        filename: &str,
        ext: &str,
        variants: Vec<(LanguageIdentifier, Vec<u8>)>,
    ) -> Result<Option<String>> {
        let hash = self.checksum.then(|| {
            let mut checksummer = SeaHasher::new();
            variants
                .iter()
                .for_each(|(_, content)| checksummer.write(content));
            URL_SAFE.encode(checksummer.finish().to_be_bytes())
        });

        for (langid, content) in variants {
            let filename = format!("{filename}.{ext}.{langid}");
            let path = self.path(&hash, dir, &filename)?;
            log::info!("Creating localized file '{:?}'", path);
            self.compress_and_write(&content, &path)?;
        }
        Ok(hash)
    }
}
