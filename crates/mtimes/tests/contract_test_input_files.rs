use builder_mtimes::InputFiles;
use camino_fs::Utf8PathBuf;

struct MockCmd {
    inputs: Vec<Utf8PathBuf>,
}

impl InputFiles for MockCmd {
    fn input_files(&self) -> Vec<Utf8PathBuf> {
        self.inputs.clone()
    }
}

#[test]
fn test_input_files_trait_returns_paths() {
    let cmd = MockCmd {
        inputs: vec!["src/main.rs".into(), "src/lib.rs".into()],
    };
    let files = cmd.input_files();
    assert_eq!(files.len(), 2);
    assert_eq!(files[0], "src/main.rs");
}

#[test]
fn test_input_files_trait_empty_vec() {
    let cmd = MockCmd { inputs: vec![] };
    assert!(cmd.input_files().is_empty());
}
