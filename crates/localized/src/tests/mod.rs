use builder_command::Output;
use camino::Utf8PathBuf;
use fs_err as fs;

use crate::{run, LocalizedCmd};

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

    let cli = LocalizedCmd::new("src/tests/data/apple_store", "svg")
        .add_output(Output::new_compress_and_sum(output_dir));

    run(&cli);
}
