use base64::engine::general_purpose::URL_SAFE;
use base64::prelude::*;
use std::{
    hash::Hasher,
    io::{Cursor, Write},
};

use crate::config::OutputOptions;
use crate::ext::ByteVecExt;
use anyhow::Result;
use brotli::enc::BrotliEncoderParams;
use brotli::BrotliCompress;
use camino::{Utf8Path, Utf8PathBuf};
use flate2::{Compression, GzBuilder};
use fs_err as fs;
use seahash::SeaHasher;
use unic_langid::LanguageIdentifier;

impl OutputOptions {
    pub fn write_file(
        &self,
        contents: Vec<u8>,
        dir: &Utf8Path,
        filename: &str,
    ) -> Result<Option<String>> {
        let hash = self.checksum.then(|| contents.base64_checksum());
        let prefix = hash.as_deref().unwrap_or_default();

        let dir = self.full_created_dir(dir)?;
        let filename = format!("{prefix}{filename}");
        self.compress_and_write(contents, &filename, &dir)?;

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

    fn compress_and_write(&self, contents: Vec<u8>, filename: &str, dir: &Utf8Path) -> Result<()> {
        // if none are set, then default to uncompressed
        let default_uncompressed = !self.uncompressed && !self.brotli && !self.gzip;

        if self.uncompressed || default_uncompressed {
            let path = dir.join(filename);
            fs::write(path, &contents)?;
        }
        if self.brotli {
            let path = dir.join(format!("{filename}.br"));
            let mut file = fs::File::create(path)?;
            let mut cursor = Cursor::new(&contents);

            let params = BrotliEncoderParams {
                quality: 8,
                ..Default::default()
            };
            BrotliCompress(&mut cursor, &mut file, &params)?;
        }

        if self.gzip {
            let filename = format!("{filename}.gz");
            let f = fs::File::create(dir.join(&filename))?;
            let mut gz = GzBuilder::new()
                .filename(filename)
                .write(f, Compression::default());
            gz.write_all(&contents)?;
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
        let dir = self.full_created_dir(dir)?;

        let hash = self.checksum.then(|| {
            let mut checksummer = SeaHasher::new();
            variants
                .iter()
                .for_each(|(_, content)| checksummer.write(content));
            URL_SAFE.encode(checksummer.finish().to_be_bytes())
        });

        let prefix = hash.as_deref().unwrap_or_default();
        let filename = format!("{prefix}{filename}");

        for (langid, content) in variants {
            let lang = langid.to_string();
            let filename = format!("{filename}.{ext}.{lang}");
            self.compress_and_write(content, &filename, &dir)?;
        }
        Ok(hash)
    }
}
