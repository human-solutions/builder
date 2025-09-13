#[cfg(test)]
mod tests {
    use crate::{AssetMetadata, Encoding, Output};
    use camino_fs::{Utf8PathBuf, Utf8PathBufExt, Utf8PathExt};
    use tempfile::TempDir;

    // Asset metadata collection and code generation are now covered by snapshot tests

    #[test]
    fn test_generate_asset_code_to_file() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = Utf8PathBuf::from_path(temp_dir.path()).unwrap();
        let output_file = temp_path.join("assets.rs");

        let mut output = Output::new(&temp_path);

        // Add some test metadata
        let metadata = AssetMetadata {
            url_path: "/favicon.ico".to_string(),
            folder: None,
            name: "favicon".to_string(),
            hash: None,
            ext: "ico".to_string(),
            available_encodings: vec![Encoding::Identity],
            available_languages: None,
            mime: "image/x-icon".to_string(),
        };
        output.asset_metadata.push(metadata);

        // Generate asset code to file
        output.generate_asset_code(output_file.as_str()).unwrap();

        // Verify file was created and contains expected content
        assert!(output_file.exists());
        let content = output_file.read_string().unwrap();

        assert!(content.contains("use builder_assets::*"));
        assert!(content.contains("use icu_locid::langid"));
        assert!(content.contains("fn load_asset"));
        assert!(content.contains("pub static FAVICON: AssetSet"));
        assert!(content.contains(r#"mime: "image/x-icon""#));
    }

    #[test]
    fn test_empty_metadata() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = Utf8PathBuf::from_path(temp_dir.path()).unwrap();
        let output_file = temp_path.join("empty_assets.rs");

        let output = Output::new(&temp_path);

        // Should handle empty metadata gracefully
        let result = output.generate_asset_code(output_file.as_str());
        assert!(result.is_ok());
        assert!(!output_file.exists()); // No file should be created when no assets
    }

    #[test]
    fn test_const_name_generation() {
        let output = Output::new("test");

        assert_eq!(output.generate_const_name("style"), "STYLE");
        assert_eq!(output.generate_const_name("app-bundle"), "APP_BUNDLE");
        assert_eq!(output.generate_const_name("my.file.name"), "MY_FILE_NAME");
        assert_eq!(output.generate_const_name("file@2x"), "FILE_2X");
    }

    #[test]
    fn test_deduplication() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = Utf8PathBuf::from_path(temp_dir.path()).unwrap();

        let mut output = Output::new(&temp_path);

        // Add duplicate metadata (same URL path)
        let metadata1 = AssetMetadata {
            url_path: "/style.css".to_string(),
            folder: None,
            name: "style".to_string(),
            hash: None,
            ext: "css".to_string(),
            available_encodings: vec![Encoding::Identity],
            available_languages: None,
            mime: "text/css".to_string(),
        };

        let metadata2 = metadata1.clone(); // Duplicate

        output.asset_metadata.push(metadata1);
        output.asset_metadata.push(metadata2);

        let generated_code = output.generate_asset_code_content();

        // Should only have one STYLE constant despite duplicate metadata
        let style_count = generated_code.matches("pub static STYLE: AssetSet").count();
        assert_eq!(style_count, 1);

        // Catalog should only reference it once
        let catalog_ref_count = generated_code.matches("&STYLE").count();
        assert_eq!(catalog_ref_count, 1);
    }

    // Complex file paths and multilingual assets are now covered by snapshot tests
}
