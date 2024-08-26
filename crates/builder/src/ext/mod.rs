pub mod anyhow;
pub mod metadata;
pub mod value;

use base64::engine::general_purpose::URL_SAFE;
use base64::prelude::*;
use camino::{Utf8Path, Utf8PathBuf};

pub trait ByteVecExt {
    fn base64_checksum(&self) -> String;
}

impl ByteVecExt for [u8] {
    fn base64_checksum(&self) -> String {
        let hash = seahash::hash(self);
        URL_SAFE.encode(hash.to_be_bytes())
    }
}

pub trait RustNaming {
    fn to_rust_module(&self) -> String;
    fn to_rust_const(&self) -> String;
}

impl RustNaming for str {
    fn to_rust_module(&self) -> String {
        self.replace('-', "_")
    }

    fn to_rust_const(&self) -> String {
        let mut s = String::with_capacity(self.len());
        for (i, char) in self.chars().enumerate() {
            if char == '.' {
                s.push('_');
                continue;
            } else if char == '_' {
                // allowed
            } else if !char.is_ascii_alphanumeric() {
                panic!("Only ascii chars and '.' allowed in rust constant names, not {char}")
            }
            if char.is_ascii_uppercase() && i != 0 {
                s.push('_');
                s.push(char);
            } else {
                s.push(char.to_ascii_uppercase());
            }
        }
        s
    }
}

pub trait Utf8PathExt {
    fn push_ext(&self, ext: &str) -> Utf8PathBuf;
}

impl Utf8PathExt for Utf8Path {
    fn push_ext(&self, ext: &str) -> Utf8PathBuf {
        let mut s = self.to_string();
        s.push('.');
        s.push_str(ext);
        Utf8PathBuf::from(s)
    }
}
