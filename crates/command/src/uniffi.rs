use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct UniffiCmd {
    pub udl_file: Utf8PathBuf,

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
            built_lib_file: built_lib_file.into(),
            library_name: library_name.into(),
            swift: false,
            kotlin: false,
        }
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
