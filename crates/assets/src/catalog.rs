use crate::asset_set::AssetSet;
use std::collections::BTreeMap;

/// AssetCatalog provides efficient URL-based lookups for assets
#[derive(Debug)]
pub struct AssetCatalog {
    assets: BTreeMap<&'static str, &'static AssetSet>,
}

impl AssetCatalog {
    /// Creates a new empty AssetCatalog
    pub fn new() -> Self {
        Self {
            assets: BTreeMap::new(),
        }
    }

    /// Creates an AssetCatalog from a slice of AssetSets
    pub fn from_assets(assets: &'static [&'static AssetSet]) -> Self {
        let mut catalog = Self::new();
        for asset_set in assets {
            catalog.add_asset(asset_set);
        }
        catalog
    }

    /// Adds an AssetSet to the catalog
    pub fn add_asset(&mut self, asset_set: &'static AssetSet) {
        self.assets.insert(asset_set.url_path, asset_set);
    }

    /// Looks up an AssetSet by URL path
    pub fn get_asset_set(&self, url_path: &str) -> Option<&'static AssetSet> {
        self.assets.get(url_path).copied()
    }

    /// Returns an iterator over all URL paths in the catalog
    pub fn urls(&self) -> impl Iterator<Item = &'static str> + '_ {
        self.assets.keys().copied()
    }

    /// Returns an iterator over all AssetSets in the catalog
    pub fn asset_sets(&self) -> impl Iterator<Item = &'static AssetSet> + '_ {
        self.assets.values().copied()
    }

    /// Returns the number of assets in the catalog
    pub fn len(&self) -> usize {
        self.assets.len()
    }

    /// Returns true if the catalog is empty
    pub fn is_empty(&self) -> bool {
        self.assets.is_empty()
    }

    /// Checks if a URL path exists in the catalog
    pub fn contains_url(&self, url_path: &str) -> bool {
        self.assets.contains_key(url_path)
    }

    /// Returns a list of all available MIME types in the catalog
    pub fn mime_types(&self) -> Vec<&'static str> {
        let mut mime_types: Vec<_> = self
            .assets
            .values()
            .map(|asset_set| asset_set.mime_type())
            .collect();
        mime_types.sort_unstable();
        mime_types.dedup();
        mime_types
    }

    /// Filters assets by MIME type
    pub fn assets_by_mime_type<'a>(
        &'a self,
        mime_type: &'a str,
    ) -> impl Iterator<Item = &'static AssetSet> + 'a {
        self.assets
            .values()
            .filter(move |asset_set| asset_set.mime_type() == mime_type)
            .copied()
    }

    /// Joins another AssetCatalog into this one, combining all assets
    ///
    /// If both catalogs contain assets with the same URL path, the other catalog's asset
    /// will overwrite the existing one in this catalog.
    ///
    /// # Arguments
    /// * `other` - The other catalog to merge into this one
    ///
    /// # Example
    /// ```
    /// use builder_assets::AssetCatalog;
    /// let mut catalog1 = AssetCatalog::new();
    /// let catalog2 = AssetCatalog::new();
    /// catalog1.join(catalog2);
    /// ```
    pub fn join(mut self, other: AssetCatalog) -> AssetCatalog {
        for (url_path, asset_set) in other.assets {
            self.assets.insert(url_path, asset_set);
        }
        self
    }
}

impl Default for AssetCatalog {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{encoding::Encoding, file_path::FilePathParts};

    // Mock provider for testing
    static MOCK_PROVIDER: fn(&str) -> Option<Vec<u8>> = mock_provider;
    fn mock_provider(_path: &str) -> Option<Vec<u8>> {
        Some(b"mock data".to_vec())
    }

    #[test]
    fn test_catalog_creation() {
        let catalog = AssetCatalog::new();
        assert!(catalog.is_empty());
        assert_eq!(catalog.len(), 0);
    }

    #[test]
    fn test_add_and_get_asset() {
        let mut catalog = AssetCatalog::new();

        // Create a static AssetSet (this would normally be done by generated code)
        static STYLE_PARTS: FilePathParts = FilePathParts {
            folder: Some("css"),
            name: "style",
            hash: None,
            ext: "css",
        };
        static STYLE_ENCODINGS: [Encoding; 2] = [Encoding::Identity, Encoding::Brotli];
        static STYLE_ASSET: AssetSet = AssetSet {
            url_path: "/css/style.css",
            file_path_parts: STYLE_PARTS,
            available_encodings: &STYLE_ENCODINGS,
            available_languages: None,
            mime: "text/css",
            provider: &MOCK_PROVIDER,
        };

        catalog.add_asset(&STYLE_ASSET);

        assert!(!catalog.is_empty());
        assert_eq!(catalog.len(), 1);
        assert!(catalog.contains_url("/css/style.css"));

        let asset_set = catalog.get_asset_set("/css/style.css");
        assert!(asset_set.is_some());
        assert_eq!(asset_set.unwrap().url_path, "/css/style.css");
        assert_eq!(asset_set.unwrap().mime, "text/css");
    }

    #[test]
    fn test_get_asset_with_negotiation() {
        let mut catalog = AssetCatalog::new();

        static SCRIPT_PARTS: FilePathParts = FilePathParts {
            folder: Some("js"),
            name: "app",
            hash: Some("hash123="),
            ext: "js",
        };
        static SCRIPT_ENCODINGS: [Encoding; 3] =
            [Encoding::Identity, Encoding::Gzip, Encoding::Brotli];
        static SCRIPT_ASSET: AssetSet = AssetSet {
            url_path: "/js/app.hash123=.js",
            file_path_parts: SCRIPT_PARTS,
            available_encodings: &SCRIPT_ENCODINGS,
            available_languages: None,
            mime: "application/javascript",
            provider: &MOCK_PROVIDER,
        };

        catalog.add_asset(&SCRIPT_ASSET);

        let asset = catalog
            .get_asset_set("/js/app.hash123=.js")
            .and_then(|set| set.asset_for(Some("br, gzip"), None));

        assert!(asset.is_some());

        let asset = asset.unwrap();
        assert_eq!(asset.encoding, Encoding::Brotli);
        assert_eq!(asset.mime, "application/javascript");
        assert_eq!(asset.file_path(), "js/app.hash123=.js.br");
    }

    #[test]
    fn test_catalog_iteration() {
        let mut catalog = AssetCatalog::new();

        static CSS_PARTS: FilePathParts = FilePathParts {
            folder: Some("css"),
            name: "style",
            hash: None,
            ext: "css",
        };
        static CSS_ENCODINGS: [Encoding; 1] = [Encoding::Identity];
        static CSS_ASSET: AssetSet = AssetSet {
            url_path: "/css/style.css",
            file_path_parts: CSS_PARTS,
            available_encodings: &CSS_ENCODINGS,
            available_languages: None,
            mime: "text/css",
            provider: &MOCK_PROVIDER,
        };

        static JS_PARTS: FilePathParts = FilePathParts {
            folder: Some("js"),
            name: "app",
            hash: None,
            ext: "js",
        };
        static JS_ENCODINGS: [Encoding; 1] = [Encoding::Identity];
        static JS_ASSET: AssetSet = AssetSet {
            url_path: "/js/app.js",
            file_path_parts: JS_PARTS,
            available_encodings: &JS_ENCODINGS,
            available_languages: None,
            mime: "application/javascript",
            provider: &MOCK_PROVIDER,
        };

        catalog.add_asset(&CSS_ASSET);
        catalog.add_asset(&JS_ASSET);

        let urls: Vec<_> = catalog.urls().collect();
        assert_eq!(urls.len(), 2);
        assert!(urls.contains(&"/css/style.css"));
        assert!(urls.contains(&"/js/app.js"));

        let asset_sets: Vec<_> = catalog.asset_sets().collect();
        assert_eq!(asset_sets.len(), 2);
    }

    #[test]
    fn test_mime_type_operations() {
        let mut catalog = AssetCatalog::new();

        static CSS_PARTS: FilePathParts = FilePathParts {
            folder: None,
            name: "style",
            hash: None,
            ext: "css",
        };
        static CSS_ENCODINGS: [Encoding; 1] = [Encoding::Identity];
        static CSS_ASSET: AssetSet = AssetSet {
            url_path: "/style.css",
            file_path_parts: CSS_PARTS,
            available_encodings: &CSS_ENCODINGS,
            available_languages: None,
            mime: "text/css",
            provider: &MOCK_PROVIDER,
        };

        static IMG_PARTS: FilePathParts = FilePathParts {
            folder: None,
            name: "logo",
            hash: None,
            ext: "png",
        };
        static IMG_ENCODINGS: [Encoding; 1] = [Encoding::Identity];
        static IMG_ASSET: AssetSet = AssetSet {
            url_path: "/logo.png",
            file_path_parts: IMG_PARTS,
            available_encodings: &IMG_ENCODINGS,
            available_languages: None,
            mime: "image/png",
            provider: &MOCK_PROVIDER,
        };

        catalog.add_asset(&CSS_ASSET);
        catalog.add_asset(&IMG_ASSET);

        let mime_types = catalog.mime_types();
        assert_eq!(mime_types.len(), 2);
        assert!(mime_types.contains(&"text/css"));
        assert!(mime_types.contains(&"image/png"));

        let css_assets: Vec<_> = catalog.assets_by_mime_type("text/css").collect();
        assert_eq!(css_assets.len(), 1);
        assert_eq!(css_assets[0].url_path, "/style.css");
    }

    #[test]
    fn test_from_assets() {
        static PARTS1: FilePathParts = FilePathParts {
            folder: None,
            name: "file1",
            hash: None,
            ext: "css",
        };
        static ENCODINGS1: [Encoding; 1] = [Encoding::Identity];
        static ASSET1: AssetSet = AssetSet {
            url_path: "/file1.css",
            file_path_parts: PARTS1,
            available_encodings: &ENCODINGS1,
            available_languages: None,
            mime: "text/css",
            provider: &MOCK_PROVIDER,
        };

        static PARTS2: FilePathParts = FilePathParts {
            folder: None,
            name: "file2",
            hash: None,
            ext: "js",
        };
        static ENCODINGS2: [Encoding; 1] = [Encoding::Identity];
        static ASSET2: AssetSet = AssetSet {
            url_path: "/file2.js",
            file_path_parts: PARTS2,
            available_encodings: &ENCODINGS2,
            available_languages: None,
            mime: "application/javascript",
            provider: &MOCK_PROVIDER,
        };

        static ASSETS: [&AssetSet; 2] = [&ASSET1, &ASSET2];

        let catalog = AssetCatalog::from_assets(&ASSETS);

        assert_eq!(catalog.len(), 2);
        assert!(catalog.contains_url("/file1.css"));
        assert!(catalog.contains_url("/file2.js"));
    }

    #[test]
    fn test_catalog_join() {
        // Create first catalog with CSS asset
        let mut catalog1 = AssetCatalog::new();

        static CSS_PARTS: FilePathParts = FilePathParts {
            folder: None,
            name: "style",
            hash: None,
            ext: "css",
        };
        static CSS_ENCODINGS: [Encoding; 1] = [Encoding::Identity];
        static CSS_ASSET: AssetSet = AssetSet {
            url_path: "/style.css",
            file_path_parts: CSS_PARTS,
            available_encodings: &CSS_ENCODINGS,
            available_languages: None,
            mime: "text/css",
            provider: &MOCK_PROVIDER,
        };

        catalog1.add_asset(&CSS_ASSET);
        assert_eq!(catalog1.len(), 1);

        // Create second catalog with JS asset
        let mut catalog2 = AssetCatalog::new();

        static JS_PARTS: FilePathParts = FilePathParts {
            folder: None,
            name: "app",
            hash: None,
            ext: "js",
        };
        static JS_ENCODINGS: [Encoding; 1] = [Encoding::Identity];
        static JS_ASSET: AssetSet = AssetSet {
            url_path: "/app.js",
            file_path_parts: JS_PARTS,
            available_encodings: &JS_ENCODINGS,
            available_languages: None,
            mime: "application/javascript",
            provider: &MOCK_PROVIDER,
        };

        catalog2.add_asset(&JS_ASSET);
        assert_eq!(catalog2.len(), 1);

        // Join catalog2 into catalog1
        let catalog1 = catalog1.join(catalog2);

        // Verify the joined catalog contains both assets
        assert_eq!(catalog1.len(), 2);
        assert!(catalog1.contains_url("/style.css"));
        assert!(catalog1.contains_url("/app.js"));

        // Verify we can retrieve both assets
        assert!(catalog1.get_asset_set("/style.css").is_some());
        assert!(catalog1.get_asset_set("/app.js").is_some());
    }

    #[test]
    fn test_catalog_join_overwrites_duplicates() {
        // Create first catalog with an asset
        let mut catalog1 = AssetCatalog::new();

        static ORIGINAL_PARTS: FilePathParts = FilePathParts {
            folder: None,
            name: "original",
            hash: None,
            ext: "css",
        };
        static ORIGINAL_ENCODINGS: [Encoding; 1] = [Encoding::Identity];
        static ORIGINAL_ASSET: AssetSet = AssetSet {
            url_path: "/style.css",
            file_path_parts: ORIGINAL_PARTS,
            available_encodings: &ORIGINAL_ENCODINGS,
            available_languages: None,
            mime: "text/css",
            provider: &MOCK_PROVIDER,
        };

        catalog1.add_asset(&ORIGINAL_ASSET);

        // Create second catalog with asset at same URL
        let mut catalog2 = AssetCatalog::new();

        static UPDATED_PARTS: FilePathParts = FilePathParts {
            folder: None,
            name: "updated",
            hash: None,
            ext: "css",
        };
        static UPDATED_ENCODINGS: [Encoding; 1] = [Encoding::Identity];
        static UPDATED_ASSET: AssetSet = AssetSet {
            url_path: "/style.css", // Same URL as original
            file_path_parts: UPDATED_PARTS,
            available_encodings: &UPDATED_ENCODINGS,
            available_languages: None,
            mime: "text/css",
            provider: &MOCK_PROVIDER,
        };

        catalog2.add_asset(&UPDATED_ASSET);

        // Join catalog2 into catalog1 - should overwrite
        let catalog1 = catalog1.join(catalog2);

        // Verify the catalog still has 1 asset but with updated content
        assert_eq!(catalog1.len(), 1);
        let asset_set = catalog1.get_asset_set("/style.css").unwrap();
        assert_eq!(asset_set.file_path_parts.name, "updated"); // Should be the new one
    }
}
