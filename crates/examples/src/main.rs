use builder_assets::*;

// Include the generated assets file created by build.rs
include!(concat!(env!("ASSET_RS_PATH")));

fn main() {
    println!("🚀 Multi-Provider Asset Example");
    println!("================================");

    // Set the filesystem base path for runtime asset loading
    let asset_rs_path = env!("ASSET_RS_PATH");
    println!("Generated asset code path: {}", asset_rs_path);

    // Use the correct filesystem base path
    let workspace_target = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent() // go up from crates/examples
        .unwrap()
        .parent() // go up from crates
        .unwrap()
        .join("target/dist/filesystem");

    let base_path = camino_fs::Utf8PathBuf::try_from(workspace_target).unwrap();
    builder_assets::set_asset_base_path(&base_path);

    println!("\n📂 Available Assets:");
    println!("Total assets: {}", ASSETS.len());

    for (i, asset_set) in ASSETS.iter().enumerate() {
        println!("\n{}. {}", i + 1, asset_set.url_path);
        println!("   Name: {}", asset_set.file_path_parts.name);
        println!("   Extension: {}", asset_set.file_path_parts.ext);
        println!("   MIME: {}", asset_set.mime);

        if let Some(folder) = &asset_set.file_path_parts.folder {
            println!("   Folder: {}", folder);
        }

        if let Some(hash) = &asset_set.file_path_parts.hash {
            println!("   Hash: {}", hash);
        }

        println!("   Encodings: {:?}", asset_set.available_encodings);

        // Try to load the asset using proper content negotiation
        // For localized assets, specify a language; for others, use defaults
        let (asset_opt, is_localized) = if asset_set.available_languages.is_some() {
            (asset_set.asset_for(Some("identity"), Some("en")), true) // Use English for localized assets
        } else {
            (asset_set.asset_for(None, None), false) // Use defaults for non-localized assets
        };

        let Some(asset) = asset_opt else {
            println!("   ⚠️  Failed to negotiate asset (no matching encoding/language)");
            continue;
        };
        match std::panic::catch_unwind(|| asset.data_for()) {
            Ok(data) => {
                println!("   ✅ Loaded successfully ({} bytes)", data.len());

                // Show localized asset info
                if is_localized {
                    println!("   🌐 Localized asset - showing English variant");
                    if let Some(languages) = asset_set.available_languages {
                        println!(
                            "   Available languages: {:?}",
                            languages.iter().map(|l| l.to_string()).collect::<Vec<_>>()
                        );
                    }
                }

                // For compressed data, compare with original file
                if asset.encoding != builder_assets::Encoding::Identity {
                    // Load the identity version for comparison
                    if let Some(identity_asset) = asset_set.asset_for(Some("identity"), None) {
                        match std::panic::catch_unwind(|| identity_asset.data_for()) {
                        Ok(original_data) => {
                            println!("   📄 Original file: {} bytes", original_data.len());
                            println!(
                                "   🗜️  Compressed to: {} bytes ({:.1}% reduction)",
                                data.len(),
                                (1.0 - data.len() as f64 / original_data.len() as f64) * 100.0
                            );

                            // Show preview of original text files only
                            if asset_set.mime.starts_with("text/")
                                || asset_set.mime == "application/javascript"
                                || asset_set.mime == "application/json"
                            {
                                let preview = String::from_utf8_lossy(&original_data);
                                let preview_lines: Vec<&str> = preview.lines().take(2).collect();
                                if !preview_lines.is_empty() {
                                    println!("   Preview (original):");
                                    for line in preview_lines {
                                        println!("     {}", line.trim());
                                    }
                                    if preview.lines().count() > 2 {
                                        println!(
                                            "     ... ({} more lines)",
                                            preview.lines().count() - 2
                                        );
                                    }
                                }
                            }
                        }
                        Err(_) => {
                            println!("   ⚠️  Could not load original file for comparison");
                        }
                        }
                    } else {
                        println!("   ⚠️  Could not negotiate identity version for comparison");
                    }
                } else {
                    // Show preview for uncompressed text files
                    if asset_set.mime.starts_with("text/")
                        || asset_set.mime == "application/javascript"
                        || asset_set.mime == "application/json"
                    {
                        let preview = String::from_utf8_lossy(&data);
                        let preview_lines: Vec<&str> = preview.lines().take(2).collect();
                        if !preview_lines.is_empty() {
                            println!("   Preview:");
                            for line in preview_lines {
                                println!("     {}", line.trim());
                            }
                            if preview.lines().count() > 2 {
                                println!("     ... ({} more lines)", preview.lines().count() - 2);
                            }
                        }
                    }
                }
            }
            Err(_) => {
                println!("   ⚠️  Failed to load (asset path or language resolution issue)");

                // For localized assets, show available languages
                if let Some(languages) = asset_set.available_languages {
                    println!(
                        "   Available languages: {:?}",
                        languages.iter().map(|l| l.to_string()).collect::<Vec<_>>()
                    );
                }
            }
        }
    }

    println!("\n🔍 Asset Catalog Usage:");
    let catalog = get_asset_catalog();

    // Test URL-based lookups with content negotiation
    let test_urls = ["/styles.css", "/app.js", "/static/config.json", "/static/favicon.ico"];
    for url in &test_urls {
        if let Some(asset_set) = catalog.get_asset_set(url) {
            // Use the asset set to create an Asset with content negotiation
            if let Some(_asset) = asset_set.asset_for(None, None) {
                println!(
                    "   ✅ Found asset for URL: {} -> {}",
                    url, asset_set.file_path_parts.name
                );
            } else {
                println!(
                    "   ⚠️  Found asset set for URL: {} but failed content negotiation",
                    url
                );
            }
        } else {
            println!("   ❌ No asset found for URL: {}", url);
        }
    }

    println!("\n🎯 Multi-Provider Example Complete!");
    println!("This example demonstrates:");
    println!("  • FileSystem provider loading assets from dist/");
    println!("  • Embed provider loading assets from binary");
    println!("  • Unified asset catalog with both providers");
    println!("  • Runtime asset loading and verification");
}
