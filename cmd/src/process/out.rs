use std::io::{Cursor, Write};

use crate::config::Out;
use anyhow::Result;
use base64::engine::general_purpose::URL_SAFE;
use base64::prelude::*;
use brotli::enc::BrotliEncoderParams;
use brotli::BrotliCompress;
use camino::Utf8Path;
use flate2::{Compression, GzBuilder};
use fs_err as fs;

impl Out {
    pub fn write_file(&self, contents: Vec<u8>, dir: &Utf8Path, filename: &str) -> Result<()> {
        let prefix = if self.checksum {
            let hash = seahash::hash(&contents);
            let hash_str = URL_SAFE.encode(hash.to_be_bytes());
            format!("{hash_str}.")
        } else {
            "".to_string()
        };

        let dir = if let Some(folder) = &self.site_folder {
            dir.join(folder)
        } else {
            dir.to_path_buf()
        };

        if self.uncompressed {
            let path = dir.join(format!("{prefix}{filename}"));
            fs::write(path, &contents)?;
        }
        if self.brotli {
            let path = dir.join(format!("{prefix}{filename}.br"));
            let mut file = fs::File::create(path)?;
            let mut cursor = Cursor::new(&contents);

            let params = BrotliEncoderParams {
                quality: 8,
                ..Default::default()
            };
            BrotliCompress(&mut cursor, &mut file, &params)?;
        }

        if self.gzip {
            let filename = format!("{prefix}{filename}.gz");
            let f = fs::File::create(dir.join(&filename))?;
            let mut gz = GzBuilder::new()
                .filename(filename)
                .write(f, Compression::default());
            gz.write_all(&contents)?;
            gz.finish()?;
        }

        Ok(())
    }
}
