use crate::{asset::Asset, encoding::Encoding, file_path::FilePathParts, negotiation};
use icu_locid::LanguageIdentifier;

/// AssetSet represents all variants of a single asset (different encodings and languages).
/// Previously known as Asset in the generated code.
#[derive(Debug)]
pub struct AssetSet {
    /// The absolute url path used to get this resource
    pub url_path: &'static str,
    pub file_path_parts: FilePathParts,
    /// All files (langs) are always encoded with all these encodings
    pub available_encodings: &'static [Encoding],
    pub available_languages: Option<&'static [LanguageIdentifier]>,
    pub mime: &'static str,
    pub provider: &'static fn(&str) -> Option<Vec<u8>>,
}

impl AssetSet {
    /// Creates a new AssetSet
    pub fn new(
        url_path: &'static str,
        file_path_parts: FilePathParts,
        available_encodings: &'static [Encoding],
        available_languages: Option<&'static [LanguageIdentifier]>,
        mime: &'static str,
        provider: &'static fn(&str) -> Option<Vec<u8>>,
    ) -> Self {
        Self {
            url_path,
            file_path_parts,
            available_encodings,
            available_languages,
            mime,
            provider,
        }
    }

    /// Performs content negotiation and returns the best matching Asset
    pub fn asset_for(&self, accept_encodings: &str, accept_languages: &str) -> Asset {
        // Negotiate encoding
        let encoding = negotiation::negotiate_encoding(accept_encodings, self.available_encodings);

        // Negotiate language (if languages are available)
        let lang = if let Some(available_langs) = self.available_languages {
            negotiation::negotiate_language(accept_languages, available_langs)
        } else {
            None
        };

        Asset::new(
            encoding,
            self.mime,
            lang,
            self.file_path_parts,
            self.provider,
        )
    }

    /// Gets a specific Asset variant without content negotiation
    pub fn asset_with(
        &self,
        encoding: Encoding,
        lang: Option<&LanguageIdentifier>,
    ) -> Option<Asset> {
        // Check if the requested encoding is available
        if !self.available_encodings.contains(&encoding) {
            return None;
        }

        // Check if the requested language is available (if specified)
        if let Some(requested_lang) = lang {
            if let Some(available_langs) = self.available_languages {
                if !available_langs.contains(requested_lang) {
                    return None;
                }
            } else {
                // Language requested but no languages available
                return None;
            }
        } else if self.available_languages.is_some() {
            // No language requested but languages are available - this might be valid
            // depending on use case, for now we'll allow it
        }

        Some(Asset::new(
            encoding,
            self.mime,
            lang.cloned(),
            self.file_path_parts,
            self.provider,
        ))
    }

    /// Returns all available language identifiers
    pub fn languages(&self) -> Option<&[LanguageIdentifier]> {
        self.available_languages
    }

    /// Returns all available encodings
    pub fn encodings(&self) -> &[Encoding] {
        self.available_encodings
    }

    /// Returns the MIME type for this asset
    pub fn mime_type(&self) -> &'static str {
        self.mime
    }

    /// Returns the URL path for this asset
    pub fn url(&self) -> &'static str {
        self.url_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use icu_locid::langid;

    // Mock provider for testing
    static MOCK_PROVIDER: fn(&str) -> Option<Vec<u8>> = mock_provider;
    fn mock_provider(path: &str) -> Option<Vec<u8>> {
        match path {
            "css/style.css" => Some(b"body { color: blue; }".to_vec()),
            "css/style.css.br" => Some(b"compressed css".to_vec()),
            "css/style.css.gzip" => Some(b"gzipped css".to_vec()),
            "css/style.hash123=.css/en.css" => Some(b"body { color: blue; }".to_vec()),
            "css/style.hash123=.css/fr.css" => Some(b"corps { couleur: bleu; }".to_vec()),
            "css/style.hash123=.css/en.css.br" => Some(b"compressed english css".to_vec()),
            _ => None,
        }
    }

    static TEST_PARTS: FilePathParts = FilePathParts {
        folder: Some("css"),
        name: "style",
        hash: None,
        ext: "css",
    };
    static TEST_ENCODINGS: [Encoding; 3] = [Encoding::Identity, Encoding::Brotli, Encoding::Gzip];
    static TEST_ENCODINGS_2: [Encoding; 2] = [Encoding::Identity, Encoding::Brotli];
    static TEST_LANGUAGES: [LanguageIdentifier; 3] = [langid!("en"), langid!("fr"), langid!("de")];
    static TEST_PARTS_WITH_HASH: FilePathParts = FilePathParts {
        folder: Some("css"),
        name: "style",
        hash: Some("hash123="),
        ext: "css",
    };

    #[test]
    fn test_asset_set_creation() {
        let asset_set = AssetSet::new(
            "/css/style.css",
            TEST_PARTS,
            &TEST_ENCODINGS,
            None,
            "text/css",
            &MOCK_PROVIDER,
        );

        assert_eq!(asset_set.url_path, "/css/style.css");
        assert_eq!(asset_set.mime, "text/css");
        assert_eq!(asset_set.available_encodings.len(), 3);
        assert!(asset_set.available_languages.is_none());
    }

    #[test]
    fn test_content_negotiation_encoding_only() {
        let asset_set = AssetSet::new(
            "/css/style.css",
            TEST_PARTS,
            &TEST_ENCODINGS,
            None,
            "text/css",
            &MOCK_PROVIDER,
        );

        // Test Brotli preference
        let asset = asset_set.asset_for("br, gzip", "");
        assert_eq!(asset.encoding, Encoding::Brotli);
        assert_eq!(asset.file_path(), "css/style.css.br");
        assert!(asset.lang.is_none());

        // Test Gzip fallback
        let asset = asset_set.asset_for("gzip", "");
        assert_eq!(asset.encoding, Encoding::Gzip);
        assert_eq!(asset.file_path(), "css/style.css.gzip");
    }

    #[test]
    fn test_content_negotiation_with_languages() {
        let asset_set = AssetSet::new(
            "/css/style.hash123=.css",
            TEST_PARTS_WITH_HASH,
            &TEST_ENCODINGS_2,
            Some(&TEST_LANGUAGES),
            "text/css",
            &MOCK_PROVIDER,
        );

        // Test language negotiation
        let asset = asset_set.asset_for("br", "fr, en");
        assert_eq!(asset.encoding, Encoding::Brotli);
        assert_eq!(asset.lang, Some(langid!("fr")));
        assert_eq!(asset.file_path(), "css/style.hash123=.css/fr.css.br");

        // Test fallback to first available language when requested isn't available
        let asset = asset_set.asset_for("identity", "es, de");
        assert_eq!(asset.encoding, Encoding::Identity);
        assert_eq!(asset.lang, Some(langid!("de")));
        assert_eq!(asset.file_path(), "css/style.hash123=.css/de.css");
    }

    static TEST_LANGUAGES_2: [LanguageIdentifier; 2] = [langid!("en"), langid!("fr")];

    #[test]
    fn test_asset_with_specific_variant() {
        let asset_set = AssetSet::new(
            "/css/style.css",
            TEST_PARTS,
            &TEST_ENCODINGS_2,
            Some(&TEST_LANGUAGES_2),
            "text/css",
            &MOCK_PROVIDER,
        );

        // Valid combination
        let asset = asset_set.asset_with(Encoding::Brotli, Some(&langid!("en")));
        assert!(asset.is_some());
        let asset = asset.unwrap();
        assert_eq!(asset.encoding, Encoding::Brotli);
        assert_eq!(asset.lang, Some(langid!("en")));

        // Invalid encoding
        let asset = asset_set.asset_with(Encoding::Gzip, Some(&langid!("en")));
        assert!(asset.is_none());

        // Invalid language
        let asset = asset_set.asset_with(Encoding::Identity, Some(&langid!("de")));
        assert!(asset.is_none());
    }

    #[test]
    fn test_asset_set_accessors() {
        let asset_set = AssetSet::new(
            "/css/style.css",
            TEST_PARTS,
            &TEST_ENCODINGS_2,
            Some(&TEST_LANGUAGES_2),
            "text/css",
            &MOCK_PROVIDER,
        );

        assert_eq!(asset_set.url(), "/css/style.css");
        assert_eq!(asset_set.mime_type(), "text/css");
        assert_eq!(asset_set.encodings().len(), 2);
        assert_eq!(asset_set.languages().unwrap().len(), 2);
        assert!(asset_set.languages().unwrap().contains(&langid!("en")));
        assert!(asset_set.languages().unwrap().contains(&langid!("fr")));
    }
}
