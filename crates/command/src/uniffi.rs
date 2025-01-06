use std::{convert::Infallible, fmt::Display, str::FromStr};

use camino_fs::Utf8PathBuf;

#[derive(Debug, Default, PartialEq, Eq)]
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

impl Display for UniffiCmd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "udl_file={}", self.udl_file)?;
        writeln!(f, "out_dir={}", self.out_dir)?;
        writeln!(f, "built_lib_file={}", self.built_lib_file)?;
        writeln!(f, "library_name={}", self.library_name)?;
        writeln!(f, "swift={}", self.swift)?;
        writeln!(f, "kotlin={}", self.kotlin)?;
        Ok(())
    }
}

impl FromStr for UniffiCmd {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut cmd = Self::default();
        for line in s.lines() {
            let (key, value) = line.split_once('=').unwrap();
            match key {
                "udl_file" => cmd.udl_file = value.into(),
                "out_dir" => cmd.out_dir = value.into(),
                "built_lib_file" => cmd.built_lib_file = value.into(),
                "library_name" => cmd.library_name = value.into(),
                "swift" => cmd.swift = value.parse().unwrap(),
                "kotlin" => cmd.kotlin = value.parse().unwrap(),
                _ => panic!("unknown key: {}", key),
            }
        }
        Ok(cmd)
    }
}
