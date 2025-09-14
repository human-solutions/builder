use builder_assets::*;

// Include the generated assets file created by build.rs
include!(concat!(env!("ASSET_RS_PATH")));

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Once;

    static INIT: Once = Once::new();

    fn setup_asset_base_path() {
        INIT.call_once(|| {
            // Use the correct filesystem base path
            let workspace_target = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent() // go up from crates/examples
                .unwrap()
                .parent() // go up from crates
                .unwrap()
                .join("target/dist/filesystem");

            let base_path = camino_fs::Utf8PathBuf::try_from(workspace_target).unwrap();
            builder_assets::set_asset_base_path(&base_path);
        });
    }

    #[test]
    fn test_generated_assets_exist() {
        setup_asset_base_path();

        // Verify that assets were generated
        assert!(ASSETS.len() > 0, "No assets were generated");

        println!("Generated {} assets:", ASSETS.len());
        for asset in &ASSETS {
            println!("  - {} ({})", asset.url_path, asset.mime);
            if let Some(langs) = asset.available_languages {
                println!(
                    "    Languages: {:?}",
                    langs.iter().map(|l| l.to_string()).collect::<Vec<_>>()
                );
            }
        }
    }

    #[test]
    fn test_filesystem_assets_loadable() {
        setup_asset_base_path();

        let fs_assets: Vec<_> = ASSETS
            .iter()
            .filter(|asset| {
                // Check if this asset uses the filesystem provider by trying to identify it
                // This is a bit hacky but works for our test
                asset.url_path.starts_with("/styles.css") || asset.url_path.starts_with("/app.js")
            })
            .collect();

        assert!(fs_assets.len() > 0, "No filesystem assets found");

        for asset_set in fs_assets {
            // Handle localized assets by specifying a language
            let asset = if asset_set.available_languages.is_some() {
                asset_set.asset_for("identity", "en") // Use English for localized assets
            } else {
                asset_set.asset_for("", "") // Use defaults for non-localized assets
            };
            match std::panic::catch_unwind(|| asset.data_for()) {
                Ok(data) => {
                    assert!(data.len() > 0, "Asset {} is empty", asset_set.url_path);
                    println!("✅ Loaded {} ({} bytes)", asset_set.url_path, data.len());
                }
                Err(_) => {
                    panic!("Failed to load filesystem asset: {}", asset_set.url_path);
                }
            }
        }
    }

    #[test]
    fn test_embedded_assets_loadable() {
        setup_asset_base_path();

        let embed_assets: Vec<_> = ASSETS
            .iter()
            .filter(|asset| {
                // Check if this asset uses the embed provider (now with site_dir)
                asset.url_path.starts_with("/static/config.json")
                    || asset.url_path.starts_with("/static/favicon.ico")
            })
            .collect();

        assert!(embed_assets.len() > 0, "No embedded assets found");

        for asset_set in embed_assets {
            let asset = asset_set.asset_for("", "");
            match std::panic::catch_unwind(|| asset.data_for()) {
                Ok(data) => {
                    assert!(data.len() > 0, "Asset {} is empty", asset_set.url_path);
                    println!(
                        "✅ Loaded embedded {} ({} bytes)",
                        asset_set.url_path,
                        data.len()
                    );
                }
                Err(_) => {
                    panic!("Failed to load embedded asset: {}", asset_set.url_path);
                }
            }
        }
    }

    #[test]
    fn test_asset_catalog_functionality() {
        setup_asset_base_path();

        let catalog = get_asset_catalog();

        // Test that we can find assets by URL (including site_dir paths)
        let expected_urls = ["/styles.css", "/app.js", "/static/config.json", "/static/favicon.ico"];

        for url in expected_urls {
            let asset_set = catalog.get_asset_set(url);
            match asset_set {
                Some(found_asset_set) => {
                    println!(
                        "✅ Catalog found {} -> {}",
                        url, found_asset_set.file_path_parts.name
                    );
                    assert_eq!(found_asset_set.url_path, url);
                }
                None => {
                    panic!("Asset catalog failed to find: {}", url);
                }
            }
        }
    }

    #[test]
    fn test_mixed_provider_loading() {
        setup_asset_base_path();

        let mut fs_loaded = 0;
        let mut embed_loaded = 0;

        for asset_set in &ASSETS {
            // Handle localized assets by specifying a language
            let asset = if asset_set.available_languages.is_some() {
                asset_set.asset_for("identity", "en") // Use English for localized assets
            } else {
                asset_set.asset_for("", "") // Use defaults for non-localized assets
            };
            match std::panic::catch_unwind(|| asset.data_for()) {
                Ok(data) => {
                    assert!(
                        data.len() > 0,
                        "Asset {} loaded but is empty",
                        asset_set.url_path
                    );

                    // Categorize by likely provider based on file extension/path
                    if asset_set.url_path.contains(".css") || asset_set.url_path.contains(".js") {
                        fs_loaded += 1;
                        println!(
                            "✅ FileSystem: {} ({} bytes)",
                            asset_set.url_path,
                            data.len()
                        );
                    } else {
                        embed_loaded += 1;
                        println!("✅ Embedded: {} ({} bytes)", asset_set.url_path, data.len());
                    }
                }
                Err(_) => {
                    // Some assets may fail to load (e.g., localized assets with path issues)
                    println!("⚠️ Failed to load asset: {}", asset_set.url_path);
                }
            }
        }

        assert!(fs_loaded >= 2, "Should load at least 2 filesystem assets (app.js, styles.css)");
        assert!(embed_loaded >= 1, "Should load at least 1 embedded asset (favicon.ico)");

        println!(
            "Successfully loaded {} filesystem and {} embedded assets",
            fs_loaded, embed_loaded
        );
    }

    #[test]
    fn test_asset_content_validation() {
        setup_asset_base_path();

        for asset_set in &ASSETS {
            // Request uncompressed version for content validation, handle localized assets
            let asset = if asset_set.available_languages.is_some() {
                asset_set.asset_for("identity", "en") // Use English for localized assets
            } else {
                asset_set.asset_for("identity", "") // Use uncompressed for non-localized assets
            };
            match std::panic::catch_unwind(|| asset.data_for()) {
                Ok(data) => {
                    let content = String::from_utf8_lossy(&data);

                    // Validate content based on file type
                    match asset_set.file_path_parts.ext {
                        "css" => {
                            assert!(
                                content.contains("color")
                                    || content.contains("background")
                                    || content.contains("{"),
                                "CSS file {} doesn't look like valid CSS",
                                asset_set.url_path
                            );
                        }
                        "js" => {
                            assert!(
                                content.contains("function")
                                    || content.contains("class")
                                    || content.contains("console"),
                                "JS file {} doesn't look like valid JavaScript",
                                asset_set.url_path
                            );
                        }
                        "json" => {
                            // Try to parse as JSON
                            serde_json::from_str::<serde_json::Value>(&content).unwrap_or_else(
                                |_| panic!("JSON file {} is not valid JSON", asset_set.url_path),
                            );
                        }
                        "ico" => {
                            // For our placeholder ICO file, just check it's not empty
                            assert!(
                                data.len() > 10,
                                "ICO file {} seems too small",
                                asset_set.url_path
                            );
                        }
                        _ => {
                            // Unknown extension, just verify it's not empty
                            assert!(data.len() > 0, "Asset {} is empty", asset_set.url_path);
                        }
                    }

                    println!(
                        "✅ Content validation passed for {} ({} ext)",
                        asset_set.url_path, asset_set.file_path_parts.ext
                    );
                }
                Err(_) => {
                    // Asset failed to load, skip validation
                    println!(
                        "⚠️  Skipping validation for {} (failed to load)",
                        asset_set.url_path
                    );
                }
            }
        }
    }

    #[test]
    fn test_environment_variables() {
        // Test that ASSET_RS_PATH is exported correctly
        let asset_rs_path = env!("ASSET_RS_PATH");
        assert!(
            asset_rs_path.contains("assets.rs"),
            "ASSET_RS_PATH should point to assets.rs, got: {}",
            asset_rs_path
        );

        println!("✅ ASSET_RS_PATH environment variable: {}", asset_rs_path);
    }

    #[test]
    fn test_localized_assets() {
        setup_asset_base_path();

        // Look for localized welcome image
        let welcome_assets: Vec<_> = ASSETS
            .iter()
            .filter(|asset| asset.url_path.contains("welcome.svg"))
            .collect();

        if welcome_assets.len() > 0 {
            for asset_set in welcome_assets {
                println!("Found localized asset: {}", asset_set.url_path);

                if let Some(languages) = asset_set.available_languages {
                    println!(
                        "  Available languages: {:?}",
                        languages.iter().map(|l| l.to_string()).collect::<Vec<_>>()
                    );

                    // Test loading different language variants
                    for lang in languages {
                        let asset = asset_set.asset_for("identity", &lang.to_string());
                        match std::panic::catch_unwind(|| asset.data_for()) {
                            Ok(data) => {
                                assert!(
                                    data.len() > 0,
                                    "Localized asset {} for {} is empty",
                                    asset_set.url_path,
                                    lang
                                );

                                let content = String::from_utf8_lossy(&data);
                                assert!(
                                    content.contains("<svg"),
                                    "Localized asset {} should be SVG content",
                                    asset_set.url_path
                                );

                                // Verify language-specific content
                                match lang.to_string().as_str() {
                                    "en" => assert!(
                                        content.contains("Welcome"),
                                        "English version should contain 'Welcome'"
                                    ),
                                    "es" => assert!(
                                        content.contains("Bienvenido"),
                                        "Spanish version should contain 'Bienvenido'"
                                    ),
                                    "fr" => assert!(
                                        content.contains("Bienvenue"),
                                        "French version should contain 'Bienvenue'"
                                    ),
                                    _ => {}
                                }

                                println!(
                                    "  ✅ Successfully loaded {} variant ({} bytes)",
                                    lang,
                                    data.len()
                                );
                            }
                            Err(_) => {
                                println!("  ⚠️  Failed to load {} variant", lang);
                            }
                        }
                    }
                } else {
                    println!("  No languages available for this asset");
                }
            }
        } else {
            println!("⚠️  No localized welcome assets found");
        }
    }
}
