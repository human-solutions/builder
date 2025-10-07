use builder_mtimes::OutputFiles;
use camino_fs::Utf8PathBuf;

struct MockCmd {
    outputs: Vec<Utf8PathBuf>,
}

impl OutputFiles for MockCmd {
    fn output_files(&self) -> Vec<Utf8PathBuf> {
        self.outputs.clone()
    }
}

#[test]
fn test_output_files_trait_returns_paths() {
    let cmd = MockCmd {
        outputs: vec!["dist/main.css".into(), "dist/main.js".into()],
    };
    let files = cmd.output_files();
    assert_eq!(files.len(), 2);
    assert_eq!(files[0], "dist/main.css");
}

#[test]
fn test_output_files_trait_empty_vec() {
    let cmd = MockCmd { outputs: vec![] };
    assert!(cmd.output_files().is_empty());
}
