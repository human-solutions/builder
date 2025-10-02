use crate::{encoding::Encoding, file_path::FilePathParts};
use icu_locid::LanguageIdentifier;

/// An asset represents a specific variant of a file with a particular encoding and language.
/// It contains all the information needed to load the actual file data.
#[derive(Debug)]
pub struct Asset {
    pub encoding: Encoding,
    pub mime: &'static str,
    pub lang: Option<LanguageIdentifier>,
    pub file_part_paths: FilePathParts,
    provider: &'static fn(&str) -> Option<Vec<u8>>,
}

impl Asset {
    /// Creates a new Asset instance
    pub fn new(
        encoding: Encoding,
        mime: &'static str,
        lang: Option<LanguageIdentifier>,
        file_part_paths: FilePathParts,
        provider: &'static fn(&str) -> Option<Vec<u8>>,
    ) -> Self {
        Self {
            encoding,
            mime,
            lang,
            file_part_paths,
            provider,
        }
    }

    /// Loads and returns the data for this asset
    pub fn data_for(&self) -> Vec<u8> {
        let path = self.file_path();
        (self.provider)(&path).expect("Asset should exist and be loadable")
    }

    /// Constructs the file system path for this specific asset variant
    pub fn file_path(&self) -> String {
        self.file_part_paths
            .construct_path(self.encoding, self.lang.as_ref())
    }

    /// Returns the URL path for this asset (same for all variants)
    pub fn url_path(&self) -> String {
        self.file_part_paths.construct_url_path()
    }
}

// Note: Asset cannot implement Clone because function pointers cannot be cloned in a meaningful way
// However, we can provide a manual clone method if needed

impl Asset {
    /// Manual clone method since Asset contains a function pointer
    pub fn clone_asset(&self) -> Self {
        Self {
            encoding: self.encoding,
            mime: self.mime,
            lang: self.lang.clone(),
            file_part_paths: self.file_part_paths,
            provider: self.provider,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use icu_locid::langid;

    // Mock provider function for testing
    static MOCK_PROVIDER: fn(&str) -> Option<Vec<u8>> = mock_provider;
    fn mock_provider(path: &str) -> Option<Vec<u8>> {
        match path {
            "assets/style.css" => Some(b"body { color: blue; }".to_vec()),
            "assets/style.css.br" => Some(b"compressed css".to_vec()),
            "assets/button.hash123=.css/fr.css" => Some(b"bouton { couleur: bleu; }".to_vec()),
            "favicon.ico" => Some(b"favicon data".to_vec()),
            _ => None,
        }
    }

    #[test]
    fn test_asset_creation() {
        let parts = FilePathParts {
            folder: Some("assets"),
            name: "style",
            hash: None,
            ext: "css",
        };

        let asset = Asset::new(Encoding::Identity, "text/css", None, parts, &MOCK_PROVIDER);

        assert_eq!(asset.encoding, Encoding::Identity);
        assert_eq!(asset.mime, "text/css");
        assert!(asset.lang.is_none());
        assert_eq!(asset.file_path(), "assets/style.css");
        assert_eq!(asset.url_path(), "/assets/style.css");
    }

    #[test]
    fn test_asset_with_encoding() {
        let parts = FilePathParts {
            folder: Some("assets"),
            name: "style",
            hash: None,
            ext: "css",
        };

        let asset = Asset::new(Encoding::Brotli, "text/css", None, parts, &MOCK_PROVIDER);

        assert_eq!(asset.file_path(), "assets/style.css.br");
        assert_eq!(asset.url_path(), "/assets/style.css");
    }

    #[test]
    fn test_asset_with_language() {
        let parts = FilePathParts {
            folder: Some("assets"),
            name: "button",
            hash: Some("hash123="),
            ext: "css",
        };

        let lang = langid!("fr");
        let asset = Asset::new(
            Encoding::Identity,
            "text/css",
            Some(lang),
            parts,
            &MOCK_PROVIDER,
        );

        assert_eq!(asset.file_path(), "assets/button.hash123=.css/fr.css");
        assert_eq!(asset.url_path(), "/assets/button.hash123=.css");
    }

    #[test]
    fn test_asset_data_loading() {
        let parts = FilePathParts {
            folder: Some("assets"),
            name: "style",
            hash: None,
            ext: "css",
        };

        let asset = Asset::new(Encoding::Identity, "text/css", None, parts, &MOCK_PROVIDER);

        let data = asset.data_for();
        assert_eq!(data, b"body { color: blue; }");
    }

    #[test]
    fn test_asset_clone() {
        let parts = FilePathParts {
            folder: None,
            name: "favicon",
            hash: None,
            ext: "ico",
        };

        let asset = Asset::new(
            Encoding::Identity,
            "image/x-icon",
            None,
            parts,
            &MOCK_PROVIDER,
        );

        let cloned = asset.clone_asset();
        assert_eq!(cloned.encoding, asset.encoding);
        assert_eq!(cloned.mime, asset.mime);
        assert_eq!(cloned.lang, asset.lang);
        assert_eq!(cloned.file_path(), asset.file_path());
    }
}
