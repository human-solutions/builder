use crate::encoding::Encoding;
use icu_locid::LanguageIdentifier;

/// The file path parts allows constructing a full path given encoding and optionally a language
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FilePathParts {
    /// relative folder
    pub folder: Option<&'static str>,
    pub name: &'static str,
    pub hash: Option<&'static str>,
    pub ext: &'static str,
}

impl FilePathParts {
    /// Constructs a file path for the given encoding and optional language.
    ///
    /// Regular files: `folder/name[.hash].ext[.encoding_ext]`
    /// Translated files: `folder/name[.hash].ext/lang.ext[.encoding_ext]`
    pub fn construct_path(&self, encoding: Encoding, lang: Option<&LanguageIdentifier>) -> String {
        let mut path = String::new();

        // Add folder if present
        if let Some(folder) = self.folder {
            path.push_str(folder);
            path.push('/');
        }

        if let Some(lang) = lang {
            // Translated file: folder/name[.hash].ext/lang.ext[.encoding_ext]
            path.push_str(self.name);

            // Add hash if present
            if let Some(hash) = self.hash {
                path.push('.');
                path.push_str(hash);
            }

            path.push('.');
            path.push_str(self.ext);
            path.push('/');
            path.push_str(&lang.to_string());
            path.push('.');
            path.push_str(self.ext);
        } else {
            // Regular file: folder/name[.hash].ext[.encoding_ext]
            path.push_str(self.name);

            // Add hash if present
            if let Some(hash) = self.hash {
                path.push('.');
                path.push_str(hash);
            }

            path.push('.');
            path.push_str(self.ext);
        }

        // Add encoding extension if needed
        if let Some(encoding_ext) = encoding.file_ending() {
            path.push('.');
            path.push_str(encoding_ext);
        }

        path
    }

    /// Constructs the URL path (without encoding extensions)
    pub fn construct_url_path(&self) -> String {
        let mut path = String::from("/");

        // Add folder if present
        if let Some(folder) = self.folder {
            path.push_str(folder);
            path.push('/');
        }

        path.push_str(self.name);

        // Add hash if present
        if let Some(hash) = self.hash {
            path.push('.');
            path.push_str(hash);
        }

        path.push('.');
        path.push_str(self.ext);

        path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use icu_locid::langid;

    #[test]
    fn test_regular_file_no_hash_identity() {
        let parts = FilePathParts {
            folder: Some("assets"),
            name: "style",
            hash: None,
            ext: "css",
        };

        assert_eq!(
            parts.construct_path(Encoding::Identity, None),
            "assets/style.css"
        );
    }

    #[test]
    fn test_regular_file_with_hash_brotli() {
        let parts = FilePathParts {
            folder: Some("assets"),
            name: "style",
            hash: Some("jLsQ8S_Iyso="),
            ext: "css",
        };

        assert_eq!(
            parts.construct_path(Encoding::Brotli, None),
            "assets/style.jLsQ8S_Iyso=.css.br"
        );
    }

    #[test]
    fn test_regular_file_no_folder() {
        let parts = FilePathParts {
            folder: None,
            name: "favicon",
            hash: Some("abc123="),
            ext: "ico",
        };

        assert_eq!(
            parts.construct_path(Encoding::Gzip, None),
            "favicon.abc123=.ico.gzip"
        );
    }

    #[test]
    fn test_translated_file_with_hash() {
        let parts = FilePathParts {
            folder: Some("components"),
            name: "button",
            hash: Some("xyz789="),
            ext: "css",
        };

        let lang = langid!("fr");
        assert_eq!(
            parts.construct_path(Encoding::Brotli, Some(&lang)),
            "components/button.xyz789=.css/fr.css.br"
        );
    }

    #[test]
    fn test_translated_file_no_hash_no_folder() {
        let parts = FilePathParts {
            folder: None,
            name: "messages",
            hash: None,
            ext: "json",
        };

        let lang = langid!("en-US");
        assert_eq!(
            parts.construct_path(Encoding::Identity, Some(&lang)),
            "messages.json/en-US.json"
        );
    }

    #[test]
    fn test_url_path_construction() {
        let parts = FilePathParts {
            folder: Some("assets/fonts"),
            name: "roboto",
            hash: Some("hash123="),
            ext: "woff2",
        };

        assert_eq!(
            parts.construct_url_path(),
            "/assets/fonts/roboto.hash123=.woff2"
        );
    }

    #[test]
    fn test_url_path_no_folder_no_hash() {
        let parts = FilePathParts {
            folder: None,
            name: "favicon",
            hash: None,
            ext: "ico",
        };

        assert_eq!(parts.construct_url_path(), "/favicon.ico");
    }
}
