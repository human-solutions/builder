use camino_fs::*;
use std::process::Command;

/// Test that the CLI binary works correctly
#[test]
fn test_cli_copy_command() {
    // Create test fixture - use .css which has a known mime type
    let fixture_dir = Utf8PathBuf::from("tests/fixtures/copy");
    fixture_dir.mkdirs().unwrap();

    let src_dir = fixture_dir.join("src");
    src_dir.mkdirs().unwrap();
    src_dir
        .join("test.css")
        .write("body { color: red; }")
        .unwrap();

    let out_dir = fixture_dir.join("out");
    if out_dir.exists() {
        out_dir.rm().unwrap();
    }

    // Create a builder.yaml config
    let config_path = fixture_dir.join("builder.yaml");
    let config = format!(
        r#"
log_level: Normal
log_destination: Terminal
release: false
builder_toml: {}
in_cargo: false
cmds:
  - !Copy
    src_dir: {}
    recursive: false
    file_extensions:
      - css
    output:
      - dir: {}
        site_dir: null
        brotli: false
        gzip: false
        uncompressed: true
        all_encodings: false
        checksum: false
        hash_output_path: null
        asset_code_generation: null
        asset_metadata: []
"#,
        config_path.as_str(),
        src_dir.as_str(),
        out_dir.as_str()
    );

    config_path.write(&config).unwrap();

    // Find the builder binary
    let binary = env!("CARGO_BIN_EXE_builder");

    // Run the CLI
    let output = Command::new(binary)
        .arg(config_path.as_str())
        .output()
        .expect("Failed to execute builder binary");

    if !output.status.success() {
        eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        panic!("Builder command failed");
    }

    // Verify output
    assert!(out_dir.exists(), "Output directory should exist");
    assert!(
        out_dir.join("test.css").exists(),
        "Copied file should exist"
    );

    let content = out_dir.join("test.css").read_string().unwrap();
    assert_eq!(content, "body { color: red; }");
}

/// Test the library execution path (non-CLI)
#[test]
fn test_library_execution() {
    use builder::builder_command::{BuilderCmd, CopyCmd, Output};

    let fixture_dir = Utf8PathBuf::from("tests/fixtures/library");
    fixture_dir.mkdirs().unwrap();

    let src_dir = fixture_dir.join("src");
    src_dir.mkdirs().unwrap();
    src_dir
        .join("lib_test.css")
        .write("div { margin: 0; }")
        .unwrap();

    let out_dir = fixture_dir.join("out");
    if out_dir.exists() {
        out_dir.rm().unwrap();
    }

    // Execute directly via library
    let cmd = BuilderCmd::new().add_copy(
        CopyCmd::new(&src_dir)
            .recursive(false)
            .file_extensions(["css"])
            .add_output(Output::new(&out_dir)),
    );

    builder::execute(cmd);

    // Verify output
    assert!(out_dir.exists(), "Output directory should exist");
    assert!(
        out_dir.join("lib_test.css").exists(),
        "Copied file should exist"
    );

    let content = out_dir.join("lib_test.css").read_string().unwrap();
    assert_eq!(content, "div { margin: 0; }");
}
