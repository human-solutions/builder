use std::{fmt::Display, str::Split};

use super::AssetEncodings;
use crate::debug;
use camino_fs::{Utf8Path, Utf8PathBuf};
use icu_locid::LanguageIdentifier;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Asset {
    pub sub_dir: Option<Utf8PathBuf>,
    pub name: String,
    pub hash: Option<String>,
    pub ext: String,
    pub encodings: AssetEncodings,
    pub translations: Vec<LanguageIdentifier>,
}

impl Asset {
    pub fn from_site_path(path: &Utf8Path) -> Option<Self> {
        if !path.is_relative() {
            debug!("Not a relative path {path}");
            return None;
        }
        parse_translated_asset(path).or_else(|| parse_asset(path))
    }

    pub fn to_url(&self) -> String {
        let mut url = "/".to_string();
        if let Some(sub_dir) = &self.sub_dir {
            url.push_str(sub_dir.as_ref());
            url.push('/');
        }
        url.push_str(&self.filename());
        url
    }

    fn filename(&self) -> String {
        let mut filename = String::new();
        filename.push_str(&self.name);
        if let Some(hash) = &self.hash {
            filename.push('.');
            filename.push_str(hash);
        }
        filename.push('.');
        filename.push_str(&self.ext);
        filename
    }

    pub fn join(&mut self, other: Self) {
        if self.sub_dir != other.sub_dir {
            crate::warn_cargo!("Can't join assets with different subdirs {self} {other}");
            return;
        }
        if self.name != other.name {
            crate::warn_cargo!("Can't join assets with different names {self} {other}");
            return;
        }
        if self.ext != other.ext {
            crate::warn_cargo!("Can't join assets with different extensions {self} {other}");
            return;
        }
        if self.hash != other.hash {
            crate::warn_cargo!("Can't join assets with different hashes {self} {other}");
            return;
        }
        self.encodings.join(&other.encodings);
        for lang in other.translations {
            if !self.translations.contains(&lang) {
                self.translations.push(lang);
            }
        }
        self.translations.sort_by_key(|l| l.to_string());
    }
}

impl Display for Asset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} [{}, {}]",
            self.to_url(),
            self.encodings
                .into_iter()
                .map(|enc| enc.name())
                .collect::<Vec<_>>()
                .join(", "),
            self.translations
                .iter()
                .map(|l| l.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

fn parse_asset(path: &Utf8Path) -> Option<Asset> {
    let mut parts = path.file_name()?.split('.');
    let (name, hash, ext) = parse_name_hash_ext(&mut parts)?;
    let encodings = parse_encoding(&mut parts)?;

    Some(Asset {
        sub_dir: non_empty_parent_path(path),
        name: name.to_string(),
        hash: hash.map(|s| s.to_string()),
        ext: ext.to_string(),
        encodings,
        translations: Vec::new(),
    })
}

fn parse_translated_asset(path: &Utf8Path) -> Option<Asset> {
    let dir = path.parent()?;
    let mut dir_parts = dir.file_name()?.split('.');

    let (name, hash, ext) = parse_name_hash_ext(&mut dir_parts)?;

    let mut file_parts = path.file_name()?.split('.');
    let lang = parse_language(&mut file_parts)?;
    if file_parts.next() != Some(ext) {
        crate::warn_cargo!("Translated asset dir extension doesn't match file extension {path}");
        return None;
    }
    let encodings = parse_encoding(&mut file_parts)?;

    Some(Asset {
        sub_dir: non_empty_parent_path(dir),
        name: name.to_string(),
        hash: hash.map(|s| s.to_string()),
        ext: ext.to_string(),
        encodings,
        translations: vec![lang],
    })
}

fn parse_name_hash_ext<'a>(
    parts: &mut Split<'a, char>,
) -> Option<(&'a str, Option<&'a str>, &'a str)> {
    let name = parts.next()?;
    let hash_or_ext = parts.next()?;
    Some(if hash_or_ext.ends_with('=') {
        let Some(ext) = parts.next() else {
            crate::warn_cargo!("No extension found after {hash_or_ext}");
            return None;
        };
        (name, Some(hash_or_ext), ext)
    } else {
        (name, None, hash_or_ext)
    })
}

fn parse_language(parts: &mut Split<char>) -> Option<LanguageIdentifier> {
    let lang_str = parts.next()?;
    match lang_str.parse() {
        Ok(lang) => Some(lang),
        Err(_e) => {
            crate::warn_cargo!("Failed to parse language identifier '{lang_str}': {_e}");
            None
        }
    }
}

fn parse_encoding(parts: &mut Split<char>) -> Option<AssetEncodings> {
    let Some(enc_str) = parts.next() else {
        return Some(AssetEncodings::uncompressed());
    };
    match enc_str.parse() {
        Ok(enc) => Some(enc),
        Err(_e) => {
            crate::warn_cargo!("Failed to parse encoding '{enc_str}': {_e}");
            None
        }
    }
}

fn non_empty_parent_path(path: &Utf8Path) -> Option<Utf8PathBuf> {
    let parent = path.parent()?;
    if parent.components().count() == 0 {
        None
    } else {
        Some(parent.to_path_buf())
    }
}

#[test]
fn asset_path() {
    let path = Utf8Path::new("assets/font.woff2");
    let asset = parse_asset(path).unwrap();
    assert_eq!(
        asset,
        Asset {
            sub_dir: Some(Utf8Path::new("assets").to_path_buf()),
            name: "font".to_string(),
            hash: None,
            ext: "woff2".to_string(),
            encodings: AssetEncodings::uncompressed(),
            translations: Vec::new(),
        }
    );
}
