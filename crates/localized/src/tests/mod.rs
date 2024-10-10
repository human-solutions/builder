use camino::Utf8PathBuf;
use fs_err as fs;

use crate::{run, Cli};

fn clean_out_dir(dir: &str) -> Utf8PathBuf {
    let output_dir = Utf8PathBuf::from(dir);
    if output_dir.exists() {
        fs::remove_dir_all(&output_dir).unwrap();
    }
    fs::create_dir_all(&output_dir).unwrap();
    output_dir
}

#[test]
fn test_localized() {
    let output_dir = clean_out_dir("src/tests/output/localized");

    let cli = Cli {
        input_dir: Utf8PathBuf::from("src/tests/data/apple_store"),
        output_dir,
        file_extension: "svg".to_string(),
        no_brotli: false,
        no_gzip: false,
        no_uncompressed: false,
        no_checksum: false,
        verbose: true,
    };

    run(&cli);
}
