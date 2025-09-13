#[cfg(test)]
mod tests {
    use super::super::{
        hash_output::HashCollector,
        site_fs::{SiteFile, finalize_hash_outputs, write_file_to_site},
    };
    use builder_command::Output;
    use camino_fs::{Utf8PathBuf, Utf8PathExt};

    #[test]
    fn test_hash_output_integration() {
        // Create a temporary directory for the test
        let temp_dir = Utf8PathBuf::from("target/test_hash_integration");
        let hash_file = Utf8PathBuf::from("target/test_hashes.rs");

        // Clean up any existing files
        if temp_dir.exists() {
            temp_dir.rm().unwrap();
        }
        if hash_file.exists() {
            hash_file.rm().unwrap();
        }

        temp_dir.mkdirs().unwrap();

        // Create output configuration with checksum and hash output enabled
        let output = Output::new_compress_and_sum(&temp_dir).hash_output_path(&hash_file);
        let mut output_config = [output];

        // Write test files
        let css_content = b"body { color: blue; }";
        let css_file = SiteFile::new("style", "css");
        write_file_to_site(&css_file, css_content, &mut output_config);

        let js_content = b"console.log('Hello, world!');";
        let js_file = SiteFile::new("script", "js");
        write_file_to_site(&js_file, js_content, &mut output_config);

        // Finalize hash outputs
        finalize_hash_outputs().unwrap();

        // Verify the hash file was created and contains expected content
        assert!(hash_file.exists(), "Hash output file should be created");

        let hash_file_content = hash_file.read_string().unwrap();
        assert!(hash_file_content.contains("pub const SCRIPT_JS: &str ="));
        assert!(hash_file_content.contains("pub const STYLE_CSS: &str ="));
        assert!(!hash_file_content.contains("FILE_HASHES")); // Should not contain the old format

        // Clean up
        temp_dir.rm().unwrap();
        hash_file.rm().unwrap();
    }

    #[test]
    fn test_hash_collector_direct() {
        let mut collector = HashCollector::new();
        collector.add_entry("test.css", "abc123");
        collector.add_entry("test.js", "def456");

        let temp_file = Utf8PathBuf::from("target/test_direct_hash.rs");
        if temp_file.exists() {
            temp_file.rm().unwrap();
        }

        collector.write_to_rust_file(&temp_file).unwrap();

        assert!(temp_file.exists());
        let content = temp_file.read_string().unwrap();
        assert!(content.contains("pub const TEST_CSS: &str = \"abc123\";"));
        assert!(content.contains("pub const TEST_JS: &str = \"def456\";"));

        // Clean up
        temp_file.rm().unwrap();
    }
}
