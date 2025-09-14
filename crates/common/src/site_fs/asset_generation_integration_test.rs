#[cfg(test)]
mod tests {
    use crate::site_fs::{SiteFile, write_file_to_site, write_translations};
    use builder_command::{Encoding, Output};
    use camino_fs::{Utf8PathBuf, Utf8PathBufExt, Utf8PathExt};
    use icu_locid::langid;
    use tempfile::TempDir;

    #[test]
    fn test_end_to_end_asset_generation_workflow() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = Utf8PathBuf::from_path(temp_dir.path()).unwrap();
        let site_dir = temp_path.join("site");

        site_dir.mkdirs().unwrap();

        // Create output configuration with asset generation
        let output =
            Output::new_compress_and_sum(&site_dir).hash_output_path(&temp_path.join("hashes.rs"));
        let mut output_configs = [output];

        // Test 1: Regular file writing
        let css_content = b"body { color: blue; margin: 0; }";
        let css_file = SiteFile::new("style", "css");
        write_file_to_site(&css_file, css_content, &mut output_configs);

        // Test 2: File with subdirectory
        let js_content = b"console.log('Hello, world!');";
        let js_file = SiteFile::new("app", "js").with_dir("js");
        write_file_to_site(&js_file, js_content, &mut output_configs);

        // Test 3: Translations
        let translations = vec![
            (langid!("en"), b"Hello".to_vec()),
            (langid!("fr"), b"Bonjour".to_vec()),
            (langid!("de"), b"Hallo".to_vec()),
        ];
        write_translations("messages.json", &translations, &mut output_configs);

        // Verify metadata was collected
        let collected_metadata = &output_configs[0].asset_metadata;
        assert_eq!(collected_metadata.len(), 3);

        // Check metadata for regular CSS file
        let css_metadata = collected_metadata
            .iter()
            .find(|m| m.name == "style" && m.ext == "css")
            .expect("CSS metadata should be collected");

        assert_eq!(css_metadata.url_path, "/style.css");
        assert!(css_metadata.folder.is_none());
        assert!(css_metadata.hash.is_some()); // Should have checksum
        assert_eq!(css_metadata.mime, "text/css");
        assert!(css_metadata.available_languages.is_none());
        assert!(css_metadata.available_encodings.contains(&Encoding::Brotli));
        assert!(css_metadata.available_encodings.contains(&Encoding::Gzip));
        assert!(
            css_metadata
                .available_encodings
                .contains(&Encoding::Identity)
        );

        // Check metadata for JS file with subdirectory
        let js_metadata = collected_metadata
            .iter()
            .find(|m| m.name == "app" && m.ext == "js")
            .expect("JS metadata should be collected");

        assert_eq!(js_metadata.url_path, "/js/app.js");
        assert_eq!(js_metadata.folder, Some("js".to_string()));
        assert_eq!(js_metadata.mime, "application/javascript");

        // Check metadata for translations
        let translations_metadata = collected_metadata
            .iter()
            .find(|m| m.name == "messages" && m.ext == "json")
            .expect("Translation metadata should be collected");

        assert_eq!(translations_metadata.url_path, "/messages.json");
        assert!(translations_metadata.available_languages.is_some());
        let langs = translations_metadata.available_languages.as_ref().unwrap();
        assert_eq!(langs.len(), 3);
        assert!(langs.contains(&langid!("en")));
        assert!(langs.contains(&langid!("fr")));
        assert!(langs.contains(&langid!("de")));

        // Test that asset metadata was collected correctly
        assert_eq!(collected_metadata.len(), 3);

        // Test direct asset code generation from collected metadata
        let generated_content = crate::asset_code_generation::generate_asset_code_content(
            collected_metadata,
            "/style.css",
        );

        // Verify it contains all expected elements
        assert!(generated_content.contains("use builder_assets::*"));
        assert!(generated_content.contains("fn load_asset"));
        assert!(generated_content.contains("pub static STYLE_CSS"));
        assert!(generated_content.contains("pub static APP_JS"));
        assert!(generated_content.contains("pub static MESSAGES_JSON"));
        assert!(generated_content.contains("pub static ASSETS"));
        assert!(generated_content.contains("pub fn get_asset_catalog"));

        // Verify translation support in generated code
        assert!(generated_content.contains(r#"langid!("en")"#));
        assert!(generated_content.contains(r#"langid!("fr")"#));
        assert!(generated_content.contains(r#"langid!("de")"#));
    }

    // Note: Code generation format details are covered by snapshot tests in out_snapshot_test.rs
    // This test focuses on the end-to-end workflow and metadata collection accuracy
}
