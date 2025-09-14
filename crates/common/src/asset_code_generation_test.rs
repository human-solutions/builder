#[cfg(test)]
mod tests {
    use crate::asset_code_generation::*;
    use builder_command::{AssetMetadata, Encoding};
    use camino_fs::Utf8PathBuf;
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

        let config = AssetCodeConfig {
            embed_config: None,
            filesystem_config: Some(ProviderConfig {
                metadata,
                base_path: Utf8PathBuf::from(""),
            }),
        };
        let generated_code = generate_multi_provider_asset_code(&config);
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

        let config = AssetCodeConfig {
            embed_config: None,
            filesystem_config: Some(ProviderConfig {
                metadata,
                base_path: Utf8PathBuf::from(""),
            }),
        };
        let generated_code = generate_multi_provider_asset_code(&config);
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

        let config = AssetCodeConfig {
            embed_config: None,
            filesystem_config: Some(ProviderConfig {
                metadata,
                base_path: Utf8PathBuf::from(""),
            }),
        };
        let generated_code = generate_multi_provider_asset_code(&config);
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

        let config = AssetCodeConfig {
            embed_config: None,
            filesystem_config: Some(ProviderConfig {
                metadata,
                base_path: Utf8PathBuf::from(""),
            }),
        };
        let generated_code = generate_multi_provider_asset_code(&config);
        assert_snapshot!(generated_code);
    }

    #[test]
    fn test_generate_empty_assets() {
        let metadata: Vec<AssetMetadata> = vec![];
        let config = AssetCodeConfig {
            embed_config: None,
            filesystem_config: Some(ProviderConfig {
                metadata,
                base_path: Utf8PathBuf::from(""),
            }),
        };
        let generated_code = generate_multi_provider_asset_code(&config);
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

    #[test]
    fn test_generate_filesystem_provider() {
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

        let config = AssetCodeConfig {
            embed_config: None,
            filesystem_config: Some(ProviderConfig {
                metadata,
                base_path: Utf8PathBuf::from("/tmp/test"),
            }),
        };
        let generated_code = generate_multi_provider_asset_code(&config);

        assert_snapshot!(generated_code);
    }

    #[test]
    fn test_generate_embed_provider() {
        let metadata = vec![AssetMetadata {
            url_path: "/app.js".to_string(),
            folder: Some("js".to_string()),
            name: "app".to_string(),
            hash: Some("xyz789=".to_string()),
            ext: "js".to_string(),
            available_encodings: vec![Encoding::Identity, Encoding::Brotli],
            available_languages: None,
            mime: "application/javascript".to_string(),
        }];

        let config = AssetCodeConfig {
            embed_config: Some(ProviderConfig {
                metadata,
                base_path: Utf8PathBuf::from("/tmp/test"),
            }),
            filesystem_config: None,
        };
        let generated_code = generate_multi_provider_asset_code(&config);

        assert_snapshot!(generated_code);
    }

    #[test]
    fn test_generate_embed_multilingual() {
        let metadata = vec![AssetMetadata {
            url_path: "/messages.json".to_string(),
            folder: None,
            name: "messages".to_string(),
            hash: None,
            ext: "json".to_string(),
            available_encodings: vec![Encoding::Identity, Encoding::Gzip],
            available_languages: Some(vec![langid!("en"), langid!("fr"), langid!("de")]),
            mime: "application/json".to_string(),
        }];

        let config = AssetCodeConfig {
            embed_config: Some(ProviderConfig {
                metadata,
                base_path: Utf8PathBuf::from("/assets"),
            }),
            filesystem_config: None,
        };
        let generated_code = generate_multi_provider_asset_code(&config);

        assert_snapshot!(generated_code);
    }

    #[test]
    fn test_multi_provider_mixed_same_file() {
        // Create a mock config with both providers
        let config = AssetCodeConfig {
            embed_config: Some(ProviderConfig {
                metadata: vec![AssetMetadata {
                    url_path: "/config.json".to_string(),
                    folder: None,
                    name: "config".to_string(),
                    hash: None,
                    ext: "json".to_string(),
                    available_encodings: vec![Encoding::Identity],
                    available_languages: None,
                    mime: "application/json".to_string(),
                }],
                base_path: Utf8PathBuf::from("/assets"),
            }),
            filesystem_config: Some(ProviderConfig {
                metadata: vec![AssetMetadata {
                    url_path: "/style.css".to_string(),
                    folder: None,
                    name: "style".to_string(),
                    hash: Some("hash123=".to_string()),
                    ext: "css".to_string(),
                    available_encodings: vec![Encoding::Identity, Encoding::Brotli],
                    available_languages: None,
                    mime: "text/css".to_string(),
                }],
                base_path: Utf8PathBuf::from("/dist"),
            }),
        };

        let generated_code = generate_multi_provider_asset_code(&config);

        // Check that both providers are present
        assert!(generated_code.contains("EmbedAssetFiles"));
        assert!(generated_code.contains("load_embed_asset"));
        assert!(generated_code.contains("LOAD_EMBED_ASSET"));
        assert!(generated_code.contains("load_filesystem_asset"));
        assert!(generated_code.contains("LOAD_FILESYSTEM_ASSET"));

        // Check that both asset constants are present
        assert!(generated_code.contains("pub static CONFIG_JSON"));
        assert!(generated_code.contains("pub static STYLE_CSS"));

        // Check provider assignments
        assert!(generated_code.contains("provider: &LOAD_EMBED_ASSET"));
        assert!(generated_code.contains("provider: &LOAD_FILESYSTEM_ASSET"));

        // Check unified catalog
        assert!(generated_code.contains("pub static ASSETS: [&AssetSet; 2]"));
    }

    #[test]
    fn test_multi_provider_embed_only() {
        let config = AssetCodeConfig {
            embed_config: Some(ProviderConfig {
                metadata: vec![AssetMetadata {
                    url_path: "/font.woff2".to_string(),
                    folder: Some("fonts".to_string()),
                    name: "font".to_string(),
                    hash: None,
                    ext: "woff2".to_string(),
                    available_encodings: vec![Encoding::Identity],
                    available_languages: None,
                    mime: "font/woff2".to_string(),
                }],
                base_path: Utf8PathBuf::from("/fonts"),
            }),
            filesystem_config: None,
        };

        let generated_code = generate_multi_provider_asset_code(&config);

        // Should only have embed provider
        assert!(generated_code.contains("EmbedAssetFiles"));
        assert!(generated_code.contains("load_embed_asset"));
        assert!(!generated_code.contains("load_filesystem_asset"));

        assert!(generated_code.contains("pub static FONT_WOFF2"));
        assert!(generated_code.contains("provider: &LOAD_EMBED_ASSET"));
    }

    #[test]
    fn test_multi_provider_filesystem_only() {
        let config = AssetCodeConfig {
            embed_config: None,
            filesystem_config: Some(ProviderConfig {
                metadata: vec![AssetMetadata {
                    url_path: "/image.png".to_string(),
                    folder: Some("images".to_string()),
                    name: "image".to_string(),
                    hash: Some("img123=".to_string()),
                    ext: "png".to_string(),
                    available_encodings: vec![Encoding::Identity],
                    available_languages: None,
                    mime: "image/png".to_string(),
                }],
                base_path: Utf8PathBuf::from("/static"),
            }),
        };

        let generated_code = generate_multi_provider_asset_code(&config);

        // Should only have filesystem provider
        assert!(generated_code.contains("load_filesystem_asset"));
        assert!(!generated_code.contains("EmbedAssetFiles"));
        assert!(!generated_code.contains("load_embed_asset"));

        assert!(generated_code.contains("pub static IMAGE_PNG"));
        assert!(generated_code.contains("provider: &LOAD_FILESYSTEM_ASSET"));
    }

    #[test]
    #[should_panic(expected = "Asset constant name conflict across providers")]
    fn test_cross_provider_naming_conflict() {
        let metadata1 = AssetMetadata {
            url_path: "/style.css".to_string(),
            folder: None,
            name: "style".to_string(),
            hash: Some("first=".to_string()),
            ext: "css".to_string(),
            available_encodings: vec![Encoding::Identity],
            available_languages: None,
            mime: "text/css".to_string(),
        };

        let metadata2 = AssetMetadata {
            url_path: "/themes/style.css".to_string(), // Different path
            folder: Some("themes".to_string()),
            name: "style".to_string(), // Same name
            hash: Some("second=".to_string()),
            ext: "css".to_string(), // Same ext -> conflict
            available_encodings: vec![Encoding::Identity],
            available_languages: None,
            mime: "text/css".to_string(),
        };

        let metadata_refs = vec![&metadata1, &metadata2];

        // This should panic due to naming conflict
        check_global_naming_conflicts(&metadata_refs);
    }
}
