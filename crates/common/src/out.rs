use std::{
    collections::HashMap,
    hash::Hasher,
    io::{Cursor, Write},
};

use base64::{engine::general_purpose::URL_SAFE, Engine};
use brotli::{enc::BrotliEncoderParams, BrotliCompress};
use builder_command::{Encoding, Output};
use camino::Utf8Path;
use flate2::{Compression, GzBuilder};
use fs_err as fs;
use seahash::SeaHasher;

use crate::{dir, ext::Utf8PathExt};

pub fn write_checksummed_variants(
    opts: &Output,
    file_extension: &str,
    variants: &[(String, Vec<u8>)],
) {
    let hash = if opts.checksum {
        let mut checksummer = SeaHasher::new();
        variants
            .iter()
            .for_each(|(_, content)| checksummer.write(content));
        URL_SAFE.encode(checksummer.finish().to_be_bytes())
    } else {
        String::new()
    };

    let ext = file_extension;
    for (filename, content) in variants {
        let path = opts.dir.join(format!("{hash}{filename}.{ext}"));
        compress_and_write(opts, content, &path);
    }
}

pub fn write<'a, It>(opts: It, content: &[u8], filename: &str)
where
    It: IntoIterator<Item = &'a Output>,
{
    let mut outputs: HashMap<Encoding, Vec<u8>> = Default::default();

    for out in opts {
        if !out.dir.exists() {
            fs::create_dir_all(&out.dir).unwrap();
        } else {
            dir::remove_files_containing(&out.dir, filename);
        }
        let filename = if out.checksum {
            let mut checksummer = SeaHasher::new();
            checksummer.write(content);
            let hash = URL_SAFE.encode(checksummer.finish().to_be_bytes());
            format!("{hash}{filename}")
        } else {
            filename.to_string()
        };
        let path = out.dir.join(&filename);

        log::info!("Writing file '{path}' for encodings: {:?}", out.encodings());
        for enc in out.encodings() {
            let contents = outputs.entry(enc).or_insert_with(|| enc.encode(content));
            enc.write(&path, contents);
        }
    }
}

fn compress_and_write(opts: &Output, contents: &[u8], path: &Utf8Path) {
    // if none are set, then default to uncompressed

    if opts.uncompressed() {
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

pub trait EncodingOutput {
    fn encode(&self, contents: &[u8]) -> Vec<u8>;
    fn write(&self, path: &Utf8Path, contents: &[u8]);
}

impl EncodingOutput for Encoding {
    fn encode(&self, contents: &[u8]) -> Vec<u8> {
        match self {
            Encoding::Brotli => brotli(contents),
            Encoding::Gzip => gzip(contents),
            Encoding::Identity => contents.to_vec(),
        }
    }

    fn write(&self, path: &Utf8Path, contents: &[u8]) {
        let path = self.add_encoding(path);
        fs::write(&path, contents).unwrap();
    }
}

fn brotli(contents: &[u8]) -> Vec<u8> {
    let mut cursor = Cursor::new(&contents);

    let params = BrotliEncoderParams {
        quality: 10,
        ..Default::default()
    };
    let mut bytes = vec![];
    BrotliCompress(&mut cursor, &mut bytes, &params).unwrap();
    bytes
}

fn gzip(contents: &[u8]) -> Vec<u8> {
    let mut bytes = vec![];
    let mut gz = GzBuilder::new().write(&mut bytes, Compression::default());
    gz.write_all(contents).unwrap();
    gz.finish().unwrap();
    bytes
}
