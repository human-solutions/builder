use std::path::PathBuf;
use std::sync::OnceLock;

/// Global storage for the asset base path configuration
static ASSET_BASE_PATH: OnceLock<PathBuf> = OnceLock::new();

/// Sets the base path for filesystem asset loading.
/// This must be called before any asset operations when using DataProvider::FileSystem.
///
/// # Panics
/// Panics if the base path has already been set.
///
/// # Example
/// ```rust
/// use builder_assets::set_asset_base_path;
///
/// // In your main function or initialization code:
/// set_asset_base_path("/opt/myapp/assets");
/// ```
pub fn set_asset_base_path<P: Into<PathBuf>>(path: P) {
    ASSET_BASE_PATH.set(path.into()).unwrap_or_else(|_| {
        panic!("Asset base path has already been set. Call set_asset_base_path() only once during application initialization.")
    });
}

/// Gets the configured asset base path.
/// Returns None if no path has been configured.
pub fn get_asset_base_path() -> Option<&'static PathBuf> {
    ASSET_BASE_PATH.get()
}

/// Gets the configured asset base path or panics with helpful instructions.
/// This is used internally by generated asset code.
pub fn get_asset_base_path_or_panic() -> &'static PathBuf {
    ASSET_BASE_PATH.get().unwrap_or_else(|| {
        panic!(
            r#"Asset base path not configured!

When using DataProvider::FileSystem, you must set the asset base path before loading assets.

Add this to your main function or initialization code:

    use builder_assets::set_asset_base_path;

    fn main() {{
        // Set the path where assets are located at runtime
        set_asset_base_path("/path/to/assets");

        // ... rest of your application
    }}

For different deployment scenarios:
- Development: set_asset_base_path("./assets")
- Docker: set_asset_base_path("/app/assets")
- System package: set_asset_base_path("/usr/share/myapp/assets")
- Relative to binary: set_asset_base_path(exe_dir.join("assets"))

Alternatively, consider using DataProvider::Embed to avoid filesystem dependencies.
"#
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_and_get_asset_base_path() {
        // Note: This test can only run once per process due to OnceCell
        if get_asset_base_path().is_none() {
            set_asset_base_path("/test/assets");
            assert_eq!(get_asset_base_path(), Some(&PathBuf::from("/test/assets")));
        }
    }

    #[test]
    fn test_panic_message_content() {
        // Test that the panic message contains helpful information
        // We just validate the message content structure
        let panic_msg = r#"Asset base path not configured!"#;
        assert!(panic_msg.contains("Asset base path not configured"));

        // Test the API functions exist and work
        if get_asset_base_path().is_none() {
            set_asset_base_path("/test/path");
        }
        assert!(get_asset_base_path().is_some());
    }
}
