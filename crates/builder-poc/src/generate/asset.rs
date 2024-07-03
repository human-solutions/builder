use unic_langid::LanguageIdentifier;

use crate::prebuild::{File, Localized, Sass};

#[derive(Debug)]
pub struct Asset {
    /// the url used to access the asset
    pub url: String,
    /// the name of the asset
    pub name: String,
    pub encodings: Vec<String>,
    pub localizations: Vec<LanguageIdentifier>,
}

impl Asset {
    pub fn from_sass(sass: &Sass, checksum: Option<String>) -> Self {
        Self {
            url: sass.url(checksum),
            name: sass.file.file_name().unwrap().to_string(),
            encodings: sass.out.encodings(),
            localizations: Vec::new(),
        }
    }

    pub fn from_localized(
        localized: &Localized,
        checksum: Option<String>,
        localizations: Vec<LanguageIdentifier>,
    ) -> Self {
        Self {
            url: localized.url(checksum),
            name: localized.path.iter().last().unwrap().to_string(),
            encodings: localized.out.encodings(),
            localizations,
        }
    }

    pub fn from_file(file: &File, checksum: Option<String>) -> Self {
        Self {
            url: file.url(checksum),
            name: file.path.iter().last().unwrap().to_string(),
            encodings: file.out.encodings(),
            localizations: Vec::new(),
        }
    }

    pub fn quoted_encoding_list(&self) -> (usize, String) {
        let count = self.encodings.len();
        let encodings = self
            .encodings
            .iter()
            .map(|e| format!(r#""{}""#, e))
            .collect::<Vec<_>>()
            .join(", ");
        (count, encodings)
    }

    pub fn quoted_lang_list(&self) -> (usize, String) {
        let count = self.localizations.len();
        let langs = self
            .localizations
            .iter()
            .map(|l| format!(r#""{l}""#))
            .collect::<Vec<_>>()
            .join(", ");
        (count, langs)
    }
}
