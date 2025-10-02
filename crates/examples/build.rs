use anyhow::Result;
use builder_command::{BuilderCmd, CopyCmd, DataProvider, LocalizedCmd, Output};
use camino_fs::Utf8PathBuf;
use std::env;

// static CARGO_PREFIX: &str = "cargo:warning=";
static CARGO_PREFIX: &str = "";

/// Find the target dir which is the CARGO_MANIFEST_DIR if it contains
/// the Cargo.lock file, or the first parent directory that contains it.
fn target_dir() -> Utf8PathBuf {
    let mut root_dir = Utf8PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    while !root_dir.join("Cargo.lock").exists() {
        root_dir = root_dir.parent().unwrap().to_path_buf();
    }
    root_dir.join("target")
}

fn main() -> Result<()> {
    println!("cargo:rerun-if-changed=assets/");
    println!("cargo:rerun-if-changed=embedded/");

    // Tell cargo to rerun if the builder binary changes
    println!("cargo:rerun-if-changed=../../target/debug/builder");
    println!("cargo:rerun-if-changed=../../target/release/builder");

    // Get paths relative to the crate root
    let dist_out = target_dir().join("dist");
    let asset_rs_path = dist_out.join("assets.rs");

    println!("{CARGO_PREFIX}Setting up multi-provider asset generation");
    println!("{CARGO_PREFIX}Workspace target dir: {}", dist_out);
    println!("{CARGO_PREFIX}    Asset code output: {}", asset_rs_path);

    // Export the asset code path as environment variable
    println!("cargo:rustc-env=ASSET_RS_PATH={}", asset_rs_path);

    // FileSystem provider: Copy non-localized assets/ to workspace target
    let filesystem_copy = CopyCmd::new("assets")
        .recursive(true)
        .file_extensions(["css", "js", "png", "jpg", "ico", "woff", "woff2"]) // Removed "svg" since welcome.svg is localized
        .add_output(
            Output::new_compress_and_sum(dist_out.join("filesystem"))
                .asset_code_gen(&asset_rs_path, DataProvider::FileSystem),
        );

    // Localized FileSystem provider: Handle welcome.svg with multiple languages
    let localized_images = LocalizedCmd::new("assets/images/welcome", "svg").add_output(
        Output::new_compress_and_sum(dist_out.join("filesystem"))
            .asset_code_gen(&asset_rs_path, DataProvider::FileSystem),
    );

    // Embed provider: Copy embedded/ to workspace target with asset code generation
    let embed_copy = CopyCmd::new("embedded")
        .recursive(true)
        .file_extensions(["json", "ico", "txt", "html", "xml"])
        .add_output(
            Output::new(dist_out.join("embedded"))
                .site_dir("static") // Add site_dir to test the functionality
                .asset_code_gen(&asset_rs_path, DataProvider::Embed),
        );

    // Look for builder binary - try debug first, then release
    let binary_path = if std::path::Path::new("../../target/debug/builder").exists() {
        "../../target/debug/builder"
    } else if std::path::Path::new("../../target/release/builder").exists() {
        "../../target/release/builder"
    } else {
        // Builder binary doesn't exist - fail the build with clear instructions
        eprintln!("\n❌ ERROR: Builder binary not found!");
        eprintln!("\nThis crate requires the builder binary to generate assets.");
        eprintln!("\nPlease build the builder binary first:");
        eprintln!("    cargo build -p builder");
        eprintln!("\nThen rebuild this crate:");
        eprintln!("    cargo build");
        eprintln!("\nOr use this one-liner:");
        eprintln!("    cargo build -p builder && cargo build\n");

        return Err(anyhow::anyhow!(
            "Builder binary not found. Build it first with: cargo build -p builder"
        ));
    };

    // Execute using the existing binary
    BuilderCmd::new()
        .add_copy(filesystem_copy)
        .add_localized(localized_images)
        .add_copy(embed_copy)
        .exec(binary_path);

    println!("{CARGO_PREFIX}Multi-provider asset generation completed successfully");

    Ok(())
}
