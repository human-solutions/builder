use common::out::OutputParams;

use crate::Cli;

#[derive(Debug)]
pub struct Asset {
    /// the url used to access the asset
    pub url: String,
    /// the name of the asset
    pub name: String,
    pub encodings: Vec<String>,
    pub localizations: Vec<String>,
}

impl Asset {
    pub fn from_localized(cli: &Cli, checksum: Option<String>, localizations: Vec<String>) -> Self {
        Self {
            url: cli.url(checksum),
            name: cli.input_dir_name(),
            encodings: cli.encodings(),
            localizations,
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
