use common::{site_fs::Asset, RustNaming};

pub trait AssetExt {
    fn quoted_encoding_list(&self) -> (usize, String);
    fn langid_list(&self) -> (usize, String);
    fn url_const(&self) -> String;
}

impl AssetExt for Asset {
    fn quoted_encoding_list(&self) -> (usize, String) {
        let count = self.encodings.into_iter().count();
        let encodings = self
            .encodings
            .into_iter()
            .map(|e| format!(r#""{}""#, e))
            .collect::<Vec<_>>()
            .join(", ");
        (count, encodings)
    }

    fn langid_list(&self) -> (usize, String) {
        let count = self.translations.len();
        let mut langs = self
            .translations
            .iter()
            .map(|l| format!(r#"langid!("{l}")"#))
            .collect::<Vec<_>>();
        langs.sort();
        (count, langs.join(", "))
    }

    fn url_const(&self) -> String {
        format!(
            r#"pub const {name}_URL: &str = "{url}";"#,
            url = self.to_url(),
            name = self.name.to_rust_const()
        )
    }
}
