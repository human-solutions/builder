use builder_mtimes::{InputFiles, OutputFiles, SkipDecision, should_skip};
use camino_fs::{Utf8Path, Utf8PathBuf};
use tempfile::TempDir;

#[derive(Clone)]
struct MockCmd {
    inputs: Vec<Utf8PathBuf>,
    outputs: Vec<Utf8PathBuf>,
}

impl MockCmd {
    fn default() -> Self {
        Self {
            inputs: vec![],
            outputs: vec![],
        }
    }
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
fn test_should_skip_no_previous_build() -> anyhow::Result<()> {
    let temp = TempDir::new()?;
    let output_dir = Utf8Path::from_path(temp.path()).unwrap();

    let cmd = MockCmd::default();

    match should_skip(&cmd, "test", output_dir)? {
        SkipDecision::Execute { reason } => {
            assert!(reason.contains("No previous build"));
        }
        _ => panic!("Expected Execute decision"),
    }
    Ok(())
}

#[test]
fn test_should_skip_with_unchanged_inputs() -> anyhow::Result<()> {
    let temp = TempDir::new()?;
    let output_dir = Utf8Path::from_path(temp.path()).unwrap();

    // Create a test input file
    let input_path = output_dir.join("test_input.txt");
    fs_err::write(&input_path, "test content")?;

    // Create a test output file
    let output_path = output_dir.join("test_output.txt");
    fs_err::write(&output_path, "output")?;

    let cmd = MockCmd {
        inputs: vec![input_path.clone()],
        outputs: vec![output_path.clone()],
    };

    // First run - should execute
    match should_skip(&cmd, "test", output_dir)? {
        SkipDecision::Execute { reason } => {
            assert!(reason.contains("No previous build"));
        }
        _ => panic!("Expected Execute decision on first run"),
    }

    // Record success
    builder_mtimes::record_success(&cmd, "test", output_dir)?;

    // Second run - should skip
    match should_skip(&cmd, "test", output_dir)? {
        SkipDecision::Skip { reason } => {
            assert!(reason.contains("unchanged"));
        }
        _ => panic!("Expected Skip decision on second run"),
    }

    Ok(())
}

#[test]
fn test_should_skip_with_changed_input() -> anyhow::Result<()> {
    let temp = TempDir::new()?;
    let output_dir = Utf8Path::from_path(temp.path()).unwrap();

    // Create a test input file
    let input_path = output_dir.join("test_input.txt");
    fs_err::write(&input_path, "test content")?;

    // Create a test output file
    let output_path = output_dir.join("test_output.txt");
    fs_err::write(&output_path, "output")?;

    let cmd = MockCmd {
        inputs: vec![input_path.clone()],
        outputs: vec![output_path.clone()],
    };

    // Record initial state
    builder_mtimes::record_success(&cmd, "test", output_dir)?;

    // Modify input file (change mtime)
    std::thread::sleep(std::time::Duration::from_millis(10));
    fs_err::write(&input_path, "modified content")?;

    // Should detect change and execute
    match should_skip(&cmd, "test", output_dir)? {
        SkipDecision::Execute { reason } => {
            assert!(reason.contains("changed") || reason.contains("Input"));
        }
        _ => panic!("Expected Execute decision after input change"),
    }

    Ok(())
}

#[test]
fn test_should_skip_with_missing_output() -> anyhow::Result<()> {
    let temp = TempDir::new()?;
    let output_dir = Utf8Path::from_path(temp.path()).unwrap();

    // Create a test input file
    let input_path = output_dir.join("test_input.txt");
    fs_err::write(&input_path, "test content")?;

    // Create a test output file
    let output_path = output_dir.join("test_output.txt");
    fs_err::write(&output_path, "output")?;

    let cmd = MockCmd {
        inputs: vec![input_path.clone()],
        outputs: vec![output_path.clone()],
    };

    // Record initial state
    builder_mtimes::record_success(&cmd, "test", output_dir)?;

    // Delete output file
    fs_err::remove_file(&output_path)?;

    // Should detect missing output and execute
    match should_skip(&cmd, "test", output_dir)? {
        SkipDecision::Execute { reason } => {
            assert!(reason.contains("missing") || reason.contains("Output"));
        }
        _ => panic!("Expected Execute decision with missing output"),
    }

    Ok(())
}
