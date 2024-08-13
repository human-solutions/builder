use crate::anyhow::{anyhow, bail, Context, Result};
use crate::generate::Output;
use crate::Config;
use camino::Utf8PathBuf;
use lightningcss::{
    printer::PrinterOptions,
    stylesheet::StyleSheet,
    targets::{Browsers, Targets},
};
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Sass {
    pub file: Utf8PathBuf,
    pub optimize: bool,
    pub out: Output,
}

impl Sass {
    pub fn file_name(&self, checksum: &Option<String>) -> String {
        let filename = self
            .file
            .file_stem()
            .map(|f| {
                let mut s = f.to_string();
                s.push_str(".css");
                s
            })
            .unwrap_or_default();
        format!("{}{filename}", checksum.as_deref().unwrap_or_default())
    }

    pub fn url(&self, checksum: &Option<String>) -> String {
        let folder = if let Some(folder) = self.out.folder.as_ref() {
            format!("/{folder}")
        } else {
            "".to_string()
        };
        let filename = self.file_name(checksum);
        format!("{folder}/{filename}")
    }

    pub fn process(&self, info: &Config) -> Result<String> {
        let file = info
            .existing_manifest_dir_path(&self.file)
            .context("sass file not found")?;

        let cmd = Command::new("sass")
            .args(["--embed-sources", "--embed-source-map", file.as_str()])
            .output()
            .context("Failed to run sass binary")?;

        let out = String::from_utf8(cmd.stdout).unwrap();
        let err = String::from_utf8(cmd.stderr).unwrap();

        if !cmd.status.success() {
            bail!("installed binary sass failed with error: {err}{out}")
        }

        if self.optimize {
            let stylesheet =
                StyleSheet::parse(&out, Default::default()).map_err(|e| anyhow!("{e}"))?;

            let targets = Targets {
                browsers: Browsers::from_browserslist([
                    ">0.3%, defaults, supports es6-module, maintained node versions",
                ])?,
                ..Default::default()
            };

            let out_css = stylesheet.to_css(PrinterOptions {
                minify: true,
                targets,
                ..Default::default()
            })?;
            return Ok(out_css.code);
        }

        Ok(out)
    }

    pub fn watched(&self) -> String {
        self.file.to_string()
    }
}
