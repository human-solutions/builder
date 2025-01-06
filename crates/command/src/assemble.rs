use std::{fmt::Display, str::FromStr};

use camino_fs::Utf8PathBuf;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct AssembleCmd {
    pub site_root: Utf8PathBuf,
    pub include_names: Vec<String>,

    /// Where to write the generated code.
    pub code_file: Option<Utf8PathBuf>,

    /// Where to write a rust file with the environment variables
    pub url_env_file: Option<Utf8PathBuf>,
}

impl AssembleCmd {
    pub fn new<P: Into<Utf8PathBuf>>(site_root: P) -> Self {
        Self {
            site_root: site_root.into(),
            ..Default::default()
        }
    }

    pub fn write_generated_code_to<P: Into<Utf8PathBuf>>(mut self, out_file: P) -> Self {
        self.code_file = Some(out_file.into());
        self
    }

    pub fn write_url_envs_to<P: Into<Utf8PathBuf>>(mut self, rs_file: P) -> Self {
        self.url_env_file = Some(rs_file.into());
        self
    }

    pub fn add_include_name<S: AsRef<str>>(mut self, name: S) -> Self {
        self.include_names.push(name.as_ref().into());
        self
    }
    pub fn include_names<I: IntoIterator<Item = S>, S: AsRef<str>>(mut self, names: I) -> Self {
        self.include_names
            .extend(names.into_iter().map(|s| s.as_ref().into()));
        self
    }
}

impl Display for AssembleCmd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "site_root={}", self.site_root)?;
        if let Some(code_file) = &self.code_file {
            writeln!(f, "code_file={}", code_file)?;
        }
        if let Some(url_env_file) = &self.url_env_file {
            writeln!(f, "url_env_file={}", url_env_file)?;
        }
        for name in &self.include_names {
            writeln!(f, "include_names={}", name)?;
        }
        Ok(())
    }
}

impl FromStr for AssembleCmd {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut cmd = AssembleCmd::default();
        for line in s.lines() {
            let (key, value) = line.split_once('=').unwrap();
            match key {
                "site_root" => cmd.site_root = value.into(),
                "code_file" => cmd.code_file = Some(value.into()),
                "url_env_file" => cmd.url_env_file = Some(value.into()),
                "include_names" => {
                    cmd.include_names.push(value.into());
                }
                _ => panic!("unknown key: {}", key),
            }
        }
        Ok(cmd)
    }
}

#[test]
fn roundtrip() {
    let cmd = AssembleCmd::new("site")
        .write_generated_code_to("gen/assets.rs")
        .write_url_envs_to("gen/asset-urls.rs")
        .include_names([
            "apple_store",
            "google_play",
            "polyglot.woff2",
            "style.css",
            "favicon.ico",
            "favicons",
        ]);

    let s = cmd.to_string();
    let cmd2 = AssembleCmd::from_str(&s).unwrap();

    assert_eq!(cmd.site_root, cmd2.site_root);
    assert_eq!(cmd.code_file, cmd2.code_file);
    assert_eq!(cmd.url_env_file, cmd2.url_env_file);
    assert_eq!(cmd.include_names, cmd2.include_names);
}
