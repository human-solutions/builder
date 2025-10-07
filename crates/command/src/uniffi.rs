use builder_mtimes::{InputFiles, OutputFiles};
use camino_fs::Utf8PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UniffiCmd {
    pub udl_file: Utf8PathBuf,

    pub config_file: Option<Utf8PathBuf>,

    /// Where to generate the bindings
    pub out_dir: Utf8PathBuf,

    /// the .dylib or .so file to generate bindings for
    /// normally in target/debug or target/release
    pub built_lib_file: Utf8PathBuf,

    pub library_name: String,

    pub swift: bool,

    pub kotlin: bool,
}

impl UniffiCmd {
    pub fn new<P1, P2, P3, S1>(
        udl_file: P1,
        out_dir: P2,
        built_lib_file: P3,
        library_name: S1,
    ) -> Self
    where
        P1: Into<Utf8PathBuf>,
        P2: Into<Utf8PathBuf>,
        P3: Into<Utf8PathBuf>,
        S1: Into<String>,
    {
        Self {
            udl_file: udl_file.into(),
            out_dir: out_dir.into(),
            config_file: None,
            built_lib_file: built_lib_file.into(),
            library_name: library_name.into(),
            swift: false,
            kotlin: false,
        }
    }

    pub fn with_config_file<P: Into<Utf8PathBuf>>(mut self, config_file: P) -> Self {
        self.config_file = Some(config_file.into());
        self
    }

    pub fn kotlin(mut self, kotlin: bool) -> Self {
        self.kotlin = kotlin;
        self
    }

    pub fn swift(mut self, swift: bool) -> Self {
        self.swift = swift;
        self
    }
}

impl InputFiles for UniffiCmd {
    fn input_files(&self) -> Vec<Utf8PathBuf> {
        let mut files = vec![self.udl_file.clone(), self.built_lib_file.clone()];
        if let Some(ref config) = self.config_file {
            files.push(config.clone());
        }
        files
    }
}

impl OutputFiles for UniffiCmd {
    fn output_files(&self) -> Vec<Utf8PathBuf> {
        let mut files = Vec::new();
        if self.swift {
            files.push(self.out_dir.join(format!("{}.swift", self.library_name)));
            files.push(
                self.out_dir
                    .join(format!("{}FFI.modulemap", self.library_name)),
            );
        }
        if self.kotlin {
            files.push(self.out_dir.join(format!("{}.kt", self.library_name)));
        }
        files
    }
}

impl crate::CommandMetadata for UniffiCmd {
    fn output_dir(&self) -> &camino_fs::Utf8Path {
        &self.out_dir
    }

    fn name(&self) -> &'static str {
        "uniffi"
    }
}
