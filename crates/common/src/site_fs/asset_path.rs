use std::fmt::Display;

use camino_fs::{Utf8Path, Utf8PathBuf};

use crate::ext::OptStringExt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SiteFile {
    pub name: String,
    pub ext: String,
    pub site_dir: Option<String>,
}

impl SiteFile {
    pub fn new<N: AsRef<str>, E: AsRef<str>>(name: N, ext: E) -> Self {
        Self {
            name: name.as_ref().to_string(),
            ext: ext.as_ref().to_string(),
            site_dir: None,
        }
    }
    pub fn with_dir<S: AsRef<str>>(mut self, dir: S) -> Self {
        self.site_dir = Some(dir.as_ref().to_string());
        self
    }

    pub fn from_file(file: &Utf8Path) -> Self {
        let (name, ext) = file.file_name().unwrap().split_once('.').unwrap();
        Self::new(name, ext)
    }
    pub fn from_relative_path(file: &Utf8Path) -> Self {
        let (name, ext) = file.file_name().unwrap().split_once('.').unwrap();

        let site_dir = file.parent().map(|p| p.to_string());
        Self {
            name: name.to_string(),
            ext: ext.to_string(),
            site_dir,
        }
    }
}

impl Display for SiteFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{dir}{name}.{ext}",
            dir = self.site_dir.postfixed_or_default('/'),
            name = self.name,
            ext = self.ext
        )
    }
}

pub struct AssetPath {
    pub subdir: Utf8PathBuf,
    pub name_ext: SiteFile,
    pub checksum: Option<String>,
}

impl AssetPath {
    pub fn absolute_path(&self, site_root: &Utf8Path) -> Utf8PathBuf {
        let filename = format!(
            "{name}{hash}.{ext}",
            name = self.name_ext.name,
            hash = self.checksum.prefixed_or_default('.'),
            ext = self.name_ext.ext
        );
        site_root.join(&self.subdir).join(filename)
    }
}

pub struct TranslatedAssetPath {
    pub site_file: SiteFile,
    pub lang: String,
    pub checksum: Option<String>,
}

impl TranslatedAssetPath {
    pub fn absolute_path(&self, site_root: &Utf8Path) -> Utf8PathBuf {
        let ext = &self.site_file.ext;
        let file_dir = format!(
            "{name}{hash}.{ext}",
            name = self.site_file.name,
            hash = self.checksum.prefixed_or_default('.'),
        );
        let file_name = format!("{lang}.{ext}", lang = self.lang);

        let mut site_root = site_root.to_path_buf();
        if let Some(dir) = &self.site_file.site_dir {
            site_root.push(dir);
        }
        site_root.join(file_dir).join(file_name)
    }
}
