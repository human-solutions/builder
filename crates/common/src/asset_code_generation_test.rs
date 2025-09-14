#[cfg(test)]
mod tests {
    use crate::asset_code_generation::*;
    use builder_command::{AssetMetadata, Encoding};
    use icu_locid::langid;
    use insta::assert_snapshot;

    #[test]
    fn test_generate_simple_asset_code() {
        let metadata = vec![AssetMetadata {
            url_path: "/style.css".to_string(),
            folder: None,
            name: "style".to_string(),
            hash: Some("abc123=".to_string()),
            ext: "css".to_string(),
            available_encodings: vec![Encoding::Identity, Encoding::Brotli],
            available_languages: None,
            mime: "text/css".to_string(),
        }];

        let generated_code = generate_asset_code_content(&metadata, "/style.css");
        assert_snapshot!(generated_code);
    }

    #[test]
    fn test_generate_multilingual_asset_code() {
        let metadata = vec![AssetMetadata {
            url_path: "/components/button.css".to_string(),
            folder: Some("components".to_string()),
            name: "button".to_string(),
            hash: Some("def456=".to_string()),
            ext: "css".to_string(),
            available_encodings: vec![Encoding::Identity, Encoding::Brotli, Encoding::Gzip],
            available_languages: Some(vec![langid!("en"), langid!("fr"), langid!("de")]),
            mime: "text/css".to_string(),
        }];

        let generated_code = generate_asset_code_content(&metadata, "/components/button.css");
        assert_snapshot!(generated_code);
    }

    #[test]
    fn test_generate_multiple_assets_code() {
        let metadata = vec![
            AssetMetadata {
                url_path: "/style.css".to_string(),
                folder: None,
                name: "style".to_string(),
                hash: None,
                ext: "css".to_string(),
                available_encodings: vec![Encoding::Identity],
                available_languages: None,
                mime: "text/css".to_string(),
            },
            AssetMetadata {
                url_path: "/js/app.js".to_string(),
                folder: Some("js".to_string()),
                name: "app".to_string(),
                hash: Some("hash123=".to_string()),
                ext: "js".to_string(),
                available_encodings: vec![Encoding::Brotli, Encoding::Gzip],
                available_languages: None,
                mime: "application/javascript".to_string(),
            },
            AssetMetadata {
                url_path: "/favicon.ico".to_string(),
                folder: None,
                name: "favicon".to_string(),
                hash: None,
                ext: "ico".to_string(),
                available_encodings: vec![Encoding::Identity],
                available_languages: None,
                mime: "image/x-icon".to_string(),
            },
            AssetMetadata {
                url_path: "/messages.json".to_string(),
                folder: None,
                name: "messages".to_string(),
                hash: Some("xyz789=".to_string()),
                ext: "json".to_string(),
                available_encodings: vec![Encoding::Identity, Encoding::Gzip],
                available_languages: Some(vec![langid!("en"), langid!("fr"), langid!("es-MX")]),
                mime: "application/json".to_string(),
            },
        ];

        let generated_code = generate_asset_code_content(&metadata, "/style.css");
        assert_snapshot!(generated_code);
    }

    #[test]
    fn test_generate_edge_case_names() {
        let metadata = vec![AssetMetadata {
            url_path: "/assets/roboto-bold@2x.woff2".to_string(),
            folder: Some("assets".to_string()),
            name: "roboto-bold@2x".to_string(),
            hash: Some("special_hash=".to_string()),
            ext: "woff2".to_string(),
            available_encodings: vec![Encoding::Identity],
            available_languages: None,
            mime: "font/woff2".to_string(),
        }];

        let generated_code = generate_asset_code_content(&metadata, "/assets/roboto-bold@2x.woff2");
        assert_snapshot!(generated_code);
    }

    #[test]
    fn test_generate_empty_assets() {
        let metadata: Vec<AssetMetadata> = vec![];
        let generated_code = generate_asset_code_content(&metadata, "");
        assert_snapshot!(generated_code);
    }

    #[test]
    fn test_const_name_generation() {
        assert_eq!(generate_const_name("style", "css"), "STYLE_CSS");
        assert_eq!(generate_const_name("app-bundle", "js"), "APP_BUNDLE_JS");
        assert_eq!(
            generate_const_name("my.file.name", "woff2"),
            "MY_FILE_NAME_WOFF2"
        );
        assert_eq!(generate_const_name("file@2x", "png"), "FILE_2X_PNG");
        assert_eq!(generate_const_name("apple_store", "svg"), "APPLE_STORE_SVG");
    }
}
