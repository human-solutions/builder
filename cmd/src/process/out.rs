use std::io::{Cursor, Write};

use crate::config::Out;
use crate::ext::ByteVecExt;
use anyhow::Result;
use brotli::enc::BrotliEncoderParams;
use brotli::BrotliCompress;
use camino::Utf8Path;
use flate2::{Compression, GzBuilder};
use fs_err as fs;

impl Out {
    pub fn write_file(
        &self,
        contents: Vec<u8>,
        dir: &Utf8Path,
        filename: &str,
    ) -> Result<Option<String>> {
        let hash = self.checksum.then(|| contents.base64_checksum());
        let prefix = hash.as_deref().unwrap_or_default();

        let dir = if let Some(folder) = &self.folder {
            dir.join(folder)
        } else {
            dir.to_path_buf()
        };

        if !dir.exists() {
            fs::create_dir_all(&dir)?;
        }

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

        Ok(hash)
    }
}
