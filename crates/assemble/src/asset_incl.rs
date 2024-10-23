#[derive(Debug)]
pub struct Asset {
    pub langs: Option<&'static [LanguageIdentifier]>,
    pub encodings: &'static [&'static str],
    pub mime: &'static str,
}

impl Asset {
    pub fn encoding(&self, accept_encodings: &str) -> Option<&'static str> {
        if self.encodings.contains(&"br") && accept_encodings.contains("br") {
            Some("br")
        } else if self.encodings.contains(&"gzip") && accept_encodings.contains("gzip") {
            Some("gzip")
        } else {
            None
        }
    }
}
