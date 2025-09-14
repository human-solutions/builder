#[cfg(test)]
mod tests {
    use crate::asset_code_generation::*;
    use builder_command::{AssetMetadata, DataProvider, Encoding, Output};
    use camino_fs::{Utf8PathBuf, Utf8PathBufExt, Utf8PathExt};
    use icu_locid::langid;
    use tempfile::TempDir;

    #[test]
    fn test_filesystem_provider_generation() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = Utf8PathBuf::from_path(temp_dir.path()).unwrap();

        let metadata = vec![
            AssetMetadata {
                url_path: "/style.css".to_string(),
                folder: None,
                name: "style".to_string(),
                hash: Some("abc123=".to_string()),
                ext: "css".to_string(),
                available_encodings: vec![Encoding::Identity, Encoding::Brotli],
                available_languages: None,
                mime: "text/css".to_string(),
            },
            AssetMetadata {
                url_path: "/messages.json".to_string(),
                folder: None,
                name: "messages".to_string(),
                hash: None,
                ext: "json".to_string(),
                available_encodings: vec![Encoding::Identity],
                available_languages: Some(vec![langid!("en"), langid!("fr")]),
                mime: "application/json".to_string(),
            },
        ];

        let generated_code = generate_asset_code_content_with_provider(
            &metadata,
            DataProvider::FileSystem,
            &temp_path,
        );

        // Verify FileSystem provider characteristics
        assert!(generated_code.contains("use builder_assets::*"));
        assert!(generated_code.contains("use icu_locid::langid"));
        assert!(!generated_code.contains("use rust_embed::Embed")); // Should not include rust-embed
        assert!(!generated_code.contains("#[derive(Embed)]")); // Should not include embed derive

        // Verify filesystem provider function
        assert!(
            generated_code.contains("/// Provider function for loading asset data from filesystem")
        );
        assert!(generated_code.contains("fn load_asset(path: &str) -> Option<Vec<u8>>"));
        assert!(generated_code.contains("std::fs::read(full_path).ok()"));
        assert!(generated_code.contains("builder_assets::get_asset_base_path_or_panic()"));
        assert!(generated_code.contains("base_path.join(path)"));

        // Verify AssetSets are generated
        assert!(generated_code.contains("pub static STYLE_CSS: AssetSet"));
        assert!(generated_code.contains("pub static MESSAGES_JSON: AssetSet"));
        assert!(generated_code.contains("provider: &load_asset"));

        // Verify catalog
        assert!(generated_code.contains("pub static ASSETS: [&AssetSet; 2]"));
        assert!(generated_code.contains("pub fn get_asset_catalog()"));
    }

    #[test]
    fn test_embed_provider_generation() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = Utf8PathBuf::from_path(temp_dir.path()).unwrap();

        let metadata = vec![AssetMetadata {
            url_path: "/app.js".to_string(),
            folder: Some("js".to_string()),
            name: "app".to_string(),
            hash: Some("xyz789=".to_string()),
            ext: "js".to_string(),
            available_encodings: vec![Encoding::Identity, Encoding::Brotli, Encoding::Gzip],
            available_languages: None,
            mime: "application/javascript".to_string(),
        }];

        let generated_code =
            generate_asset_code_content_with_provider(&metadata, DataProvider::Embed, &temp_path);

        // Verify Embed provider characteristics
        assert!(generated_code.contains("use builder_assets::*"));
        assert!(generated_code.contains("use icu_locid::langid"));
        assert!(generated_code.contains("use rust_embed::Embed")); // Should include rust-embed

        // Verify rust-embed setup
        assert!(generated_code.contains("#[derive(Embed)]"));
        assert!(generated_code.contains(&format!("#[folder = \"{}\"]", temp_path)));
        assert!(generated_code.contains("pub struct AssetFiles;"));

        // Verify embedded provider function
        assert!(generated_code.contains("/// Provider function for loading embedded asset data"));
        assert!(generated_code.contains("fn load_asset(path: &str) -> Option<Vec<u8>>"));
        assert!(generated_code.contains("AssetFiles::get(path)"));
        assert!(generated_code.contains("f.data.into_owned()"));

        // Verify AssetSets are generated
        assert!(generated_code.contains("pub static APP_JS: AssetSet"));
        assert!(generated_code.contains("provider: &load_asset"));

        // Verify catalog
        assert!(generated_code.contains("pub static ASSETS: [&AssetSet; 1]"));
    }

    #[test]
    fn test_end_to_end_embed_workflow() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = Utf8PathBuf::from_path(temp_dir.path()).unwrap();
        let site_dir = temp_path.join("site");
        let assets_dir = temp_path.join("assets");
        let assets_file = temp_path.join("embedded_assets.rs");

        // Create directories
        site_dir.mkdirs().unwrap();
        assets_dir.mkdirs().unwrap();

        // Create some test files that would be embedded
        let css_file = assets_dir.join("style.css");
        css_file.write("body { color: blue; }").unwrap();

        let js_file = assets_dir.join("app.js");
        js_file
            .write("console.log('Hello, embedded world!');")
            .unwrap();

        // Register metadata with Embed provider
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
                url_path: "/app.js".to_string(),
                folder: None,
                name: "app".to_string(),
                hash: None,
                ext: "js".to_string(),
                available_encodings: vec![Encoding::Identity, Encoding::Brotli],
                available_languages: None,
                mime: "application/javascript".to_string(),
            },
        ];

        register_asset_metadata_for_output(
            &assets_file,
            metadata,
            DataProvider::Embed,
            &assets_dir,
        );

        // Test finalization (directory creation handled automatically)
        finalize_asset_code_outputs().unwrap();

        // Verify the file was created
        assert!(assets_file.exists());
        let content = assets_file.read_string().unwrap();

        // Verify embed-specific content
        assert!(content.contains("use rust_embed::Embed"));
        assert!(content.contains("#[derive(Embed)]"));
        assert!(content.contains(&format!("#[folder = \"{}\"]", assets_dir)));
        assert!(content.contains("pub struct AssetFiles;"));
        assert!(content.contains("AssetFiles::get(path)"));

        // Verify standard content
        assert!(content.contains("pub static STYLE_CSS: AssetSet"));
        assert!(content.contains("pub static APP_JS: AssetSet"));
        assert!(content.contains("pub static ASSETS: [&AssetSet; 2]"));
    }

    #[test]
    fn test_end_to_end_filesystem_workflow() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = Utf8PathBuf::from_path(temp_dir.path()).unwrap();
        let site_dir = temp_path.join("site");
        let assets_file = temp_path.join("filesystem_assets.rs");

        site_dir.mkdirs().unwrap();

        let metadata = vec![AssetMetadata {
            url_path: "/favicon.ico".to_string(),
            folder: None,
            name: "favicon".to_string(),
            hash: Some("hash123=".to_string()),
            ext: "ico".to_string(),
            available_encodings: vec![Encoding::Identity],
            available_languages: None,
            mime: "image/x-icon".to_string(),
        }];

        register_asset_metadata_for_output(
            &assets_file,
            metadata,
            DataProvider::FileSystem,
            &site_dir,
        );

        // Test finalization (directory creation handled automatically)
        finalize_asset_code_outputs().unwrap();

        // Verify the file was created
        assert!(assets_file.exists());
        let content = assets_file.read_string().unwrap();

        // Verify filesystem-specific content
        assert!(content.contains("use builder_assets::*"));
        assert!(content.contains("use icu_locid::langid"));
        assert!(!content.contains("use rust_embed::Embed")); // Should not include rust-embed
        assert!(!content.contains("#[derive(Embed)]")); // Should not include embed derive
        assert!(content.contains("std::fs::read(full_path).ok()"));
        assert!(content.contains("builder_assets::get_asset_base_path_or_panic()"));
        assert!(content.contains("base_path.join(path)"));

        // Verify standard content
        assert!(content.contains("pub static FAVICON_ICO: AssetSet"));
        assert!(content.contains("pub static ASSETS: [&AssetSet; 1]"));
    }

    #[test]
    fn test_output_builder_pattern() {
        // Test the builder pattern for configuring asset code generation
        let output = Output::new("dist").asset_code_gen("src/assets.rs", DataProvider::Embed);

        assert_eq!(
            output.asset_code_generation,
            Some((Utf8PathBuf::from("src/assets.rs"), DataProvider::Embed))
        );

        let output2 = Output::new_compress_and_sum("dist")
            .asset_code_gen("generated/assets.rs", DataProvider::FileSystem);

        assert_eq!(
            output2.asset_code_generation,
            Some((
                Utf8PathBuf::from("generated/assets.rs"),
                DataProvider::FileSystem
            ))
        );
    }

    #[test]
    fn test_multiple_outputs_same_file() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = Utf8PathBuf::from_path(temp_dir.path()).unwrap();
        let assets_file = temp_path.join("combined_assets.rs");

        // Register metadata from multiple sources to the same output file
        let metadata1 = vec![AssetMetadata {
            url_path: "/style.css".to_string(),
            folder: None,
            name: "style".to_string(),
            hash: None,
            ext: "css".to_string(),
            available_encodings: vec![Encoding::Identity],
            available_languages: None,
            mime: "text/css".to_string(),
        }];

        let metadata2 = vec![AssetMetadata {
            url_path: "/app.js".to_string(),
            folder: None,
            name: "app".to_string(),
            hash: None,
            ext: "js".to_string(),
            available_encodings: vec![Encoding::Brotli],
            available_languages: None,
            mime: "application/javascript".to_string(),
        }];

        register_asset_metadata_for_output(
            &assets_file,
            metadata1,
            DataProvider::Embed,
            &temp_path,
        );
        register_asset_metadata_for_output(
            &assets_file,
            metadata2,
            DataProvider::Embed,
            &temp_path,
        );

        // Test finalization (directory creation handled automatically)
        finalize_asset_code_outputs().unwrap();

        assert!(assets_file.exists());
        let content = assets_file.read_string().unwrap();

        // Should contain both assets
        assert!(content.contains("pub static STYLE_CSS: AssetSet"));
        assert!(content.contains("pub static APP_JS: AssetSet"));
        assert!(content.contains("pub static ASSETS: [&AssetSet; 2]"));
    }
}
