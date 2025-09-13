#[cfg(test)]
mod tests {
    use crate::{AssetMetadata, Encoding, Output};
    use camino_fs::{Utf8PathBuf, Utf8PathBufExt};
    use icu_locid::langid;
    use insta::assert_snapshot;
    use tempfile::TempDir;

    #[test]
    fn test_generate_simple_asset_code() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = Utf8PathBuf::from_path(temp_dir.path()).unwrap();

        let mut output = Output::new(&temp_path);

        let metadata = AssetMetadata {
            url_path: "/style.css".to_string(),
            folder: None,
            name: "style".to_string(),
            hash: Some("abc123=".to_string()),
            ext: "css".to_string(),
            available_encodings: vec![Encoding::Identity, Encoding::Brotli],
            available_languages: None,
            mime: "text/css".to_string(),
        };

        output.asset_metadata.push(metadata);

        let generated_code = output.generate_asset_code_content();

        // Replace the temp path with a placeholder for consistent snapshots
        let normalized_code = generated_code.replace(&temp_path.to_string(), "/tmp/test");

        assert_snapshot!(normalized_code);
    }

    #[test]
    fn test_generate_multilingual_asset_code() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = Utf8PathBuf::from_path(temp_dir.path()).unwrap();

        let mut output = Output::new(&temp_path);

        let metadata = AssetMetadata {
            url_path: "/components/button.css".to_string(),
            folder: Some("components".to_string()),
            name: "button".to_string(),
            hash: Some("def456=".to_string()),
            ext: "css".to_string(),
            available_encodings: vec![Encoding::Identity, Encoding::Brotli, Encoding::Gzip],
            available_languages: Some(vec![langid!("en"), langid!("fr"), langid!("de")]),
            mime: "text/css".to_string(),
        };

        output.asset_metadata.push(metadata);

        let generated_code = output.generate_asset_code_content();
        let normalized_code = generated_code.replace(&temp_path.to_string(), "/tmp/test");

        assert_snapshot!(normalized_code);
    }

    #[test]
    fn test_generate_multiple_assets_code() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = Utf8PathBuf::from_path(temp_dir.path()).unwrap();

        let mut output = Output::new(&temp_path);

        // Add multiple diverse assets
        let assets = vec![
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

        output.asset_metadata.extend(assets);

        let generated_code = output.generate_asset_code_content();
        let normalized_code = generated_code.replace(&temp_path.to_string(), "/tmp/test");

        assert_snapshot!(normalized_code);
    }

    #[test]
    fn test_generate_edge_case_names() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = Utf8PathBuf::from_path(temp_dir.path()).unwrap();

        let mut output = Output::new(&temp_path);

        let metadata = AssetMetadata {
            url_path: "/assets/roboto-bold@2x.woff2".to_string(),
            folder: Some("assets".to_string()),
            name: "roboto-bold@2x".to_string(),
            hash: Some("special_hash=".to_string()),
            ext: "woff2".to_string(),
            available_encodings: vec![Encoding::Identity],
            available_languages: None,
            mime: "font/woff2".to_string(),
        };

        output.asset_metadata.push(metadata);

        let generated_code = output.generate_asset_code_content();
        let normalized_code = generated_code.replace(&temp_path.to_string(), "/tmp/test");

        assert_snapshot!(normalized_code);
    }

    #[test]
    fn test_generate_empty_assets() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = Utf8PathBuf::from_path(temp_dir.path()).unwrap();

        let output = Output::new(&temp_path);
        // No metadata added

        let generated_code = output.generate_asset_code_content();
        let normalized_code = generated_code.replace(&temp_path.to_string(), "/tmp/test");

        assert_snapshot!(normalized_code);
    }
}
