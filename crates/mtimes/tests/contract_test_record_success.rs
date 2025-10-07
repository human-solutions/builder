use builder_mtimes::{InputFiles, OutputFiles, record_success};
use camino_fs::{Utf8Path, Utf8PathBuf};
use tempfile::TempDir;

struct MockCmd {
    inputs: Vec<Utf8PathBuf>,
    outputs: Vec<Utf8PathBuf>,
}

impl InputFiles for MockCmd {
    fn input_files(&self) -> Vec<Utf8PathBuf> {
        self.inputs.clone()
    }
}

impl OutputFiles for MockCmd {
    fn output_files(&self) -> Vec<Utf8PathBuf> {
        self.outputs.clone()
    }
}

#[test]
fn test_record_success_creates_tsv() -> anyhow::Result<()> {
    let temp = TempDir::new()?;
    let output_dir = Utf8Path::from_path(temp.path()).unwrap();

    // Create test input file
    let input_path = output_dir.join("test_input.txt");
    fs_err::write(&input_path, "content")?;

    let cmd = MockCmd {
        inputs: vec![input_path],
        outputs: vec![],
    };

    // Record success
    record_success(&cmd, "test", output_dir)?;

    // Verify TSV file was created
    let tsv_path = output_dir.join("test-mtimes.tsv");
    assert!(tsv_path.exists(), "TSV file should be created");

    // Verify content is not empty
    let content = fs_err::read_to_string(&tsv_path)?;
    assert!(!content.is_empty(), "TSV file should have content");

    Ok(())
}

#[test]
fn test_record_success_with_multiple_inputs() -> anyhow::Result<()> {
    let temp = TempDir::new()?;
    let output_dir = Utf8Path::from_path(temp.path()).unwrap();

    // Create test input files
    let input1 = output_dir.join("input1.txt");
    let input2 = output_dir.join("input2.txt");
    fs_err::write(&input1, "content1")?;
    fs_err::write(&input2, "content2")?;

    let cmd = MockCmd {
        inputs: vec![input1, input2],
        outputs: vec![],
    };

    // Record success
    record_success(&cmd, "test", output_dir)?;

    // Verify TSV file contains both inputs
    let tsv_path = output_dir.join("test-mtimes.tsv");
    let content = fs_err::read_to_string(&tsv_path)?;
    let lines: Vec<&str> = content.lines().collect();

    assert_eq!(lines.len(), 2, "TSV should have 2 lines for 2 inputs");

    Ok(())
}
