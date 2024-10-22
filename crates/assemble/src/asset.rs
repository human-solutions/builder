use common::RustNaming;

#[derive(Debug)]
pub struct Asset {
    /// the url used to access the asset
    pub url: String,
    /// the name of the asset
    pub name: String,
    pub encodings: Vec<String>,
    pub mime: String,
    pub localizations: Vec<String>,
}

impl Asset {
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

    pub fn url_const(&self, url_prefix: &str) -> String {
        format!(
            r#"pub const {name}_URL: &str = "{url_prefix}{url}";"#,
            url = self.url,
            name = self.name.to_rust_const()
        )
    }
}
