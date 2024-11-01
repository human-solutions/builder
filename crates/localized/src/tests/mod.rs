use builder_command::Output;
use camino_fs::*;

use crate::{run, LocalizedCmd};

fn clean_out_dir(dir: &str) -> Utf8PathBuf {
    let output_dir = Utf8PathBuf::from(dir);
    output_dir.rm().unwrap();
    output_dir.mkdirs().unwrap();
    output_dir
}

#[test]
fn test_localized() {
    let output_dir = clean_out_dir("src/tests/output/localized");

    let cli = LocalizedCmd::new("src/tests/data/apple_store", "svg")
        .add_output(Output::new_compress_and_sum(output_dir));

    run(&cli);
}
