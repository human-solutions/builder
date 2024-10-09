use std::{
    hash::Hasher,
    io::{Cursor, Write},
};

use base64::{engine::general_purpose::URL_SAFE, Engine};
use brotli::{enc::BrotliEncoderParams, BrotliCompress};
use camino::Utf8Path;
use flate2::{Compression, GzBuilder};
use fs_err as fs;
use seahash::SeaHasher;

use crate::ext::Utf8PathExt;

pub trait OutputParams {
    fn gzip(&self) -> bool;
    fn brotli(&self) -> bool;
    fn uncompressed(&self) -> bool;
    fn checksum(&self) -> bool;
    fn output_dir(&self) -> &Utf8Path;
    fn file_extension(&self) -> &str;

    fn encodings(&self) -> Vec<String> {
        let mut encodings = vec![];
        if self.brotli() {
            encodings.push("br".to_string());
        }
        if self.gzip() {
            encodings.push("gzip".to_string());
        }
        if self.uncompressed() {
            encodings.push("".to_string());
        }
        encodings
    }
}

pub fn write_checksummed_variants<P: OutputParams>(
    opts: &P,
    variants: &[(String, Vec<u8>)],
) -> Option<String> {
    let hash = if opts.checksum() {
        let mut checksummer = SeaHasher::new();
        variants
            .iter()
            .for_each(|(_, content)| checksummer.write(content));
        URL_SAFE.encode(checksummer.finish().to_be_bytes())
    } else {
        String::new()
    };

    let ext = opts.file_extension();
    for (filename, content) in variants {
        let path = opts.output_dir().join(format!("{hash}{filename}.{ext}"));
        compress_and_write(opts, content, &path);
    }
    opts.checksum().then_some(hash)
}

pub fn compress_and_write<P: OutputParams>(opts: &P, contents: &[u8], path: &Utf8Path) {
    // if none are set, then default to uncompressed
    let default_uncompressed = !opts.uncompressed() && !opts.brotli() && !opts.gzip();

    if opts.uncompressed() || default_uncompressed {
        log::info!("Writing uncompressed file '{:?}'", path);
        fs::write(path, contents).unwrap();
    }
    if opts.brotli() {
        let path = path.push_ext("br");
        log::info!("Writing brotli file '{:?}'", path);

        let mut file = fs::File::create(path).unwrap();
        let mut cursor = Cursor::new(&contents);

        let params = BrotliEncoderParams {
            quality: 10,
            ..Default::default()
        };
        BrotliCompress(&mut cursor, &mut file, &params).unwrap();
    }

    if opts.gzip() {
        let path = path.push_ext("gz");
        log::info!("Writing gzip file '{:?}'", path);

        let f = fs::File::create(path).unwrap();
        let mut gz = GzBuilder::new().write(f, Compression::default());
        gz.write_all(contents).unwrap();
        gz.finish().unwrap();
    }
}
