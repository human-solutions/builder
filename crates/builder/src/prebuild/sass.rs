use crate::anyhow::{anyhow, bail, Context, Result};
use crate::generate::Output;
use crate::Config;
use camino::Utf8PathBuf;
use fs_err as fs;
use lightningcss::{
    printer::PrinterOptions,
    stylesheet::StyleSheet,
    targets::{Browsers, Targets},
};
use serde::Deserialize;
use std::process::Command;
use which::which;

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Sass {
    pub file: Utf8PathBuf,
    pub optimize: bool,
    pub out: Output,
}

impl Sass {
    pub fn url(&self, checksum: Option<String>) -> String {
        let folder = if let Some(folder) = self.out.folder.as_ref() {
            format!("/{folder}")
        } else {
            "".to_string()
        };
        let filename = self.file.file_name().unwrap();
        format!("{folder}/{}{filename}", checksum.unwrap_or_default())
    }

    pub fn process(&self, info: &Config) -> Result<String> {
        let file = info
            .existing_manifest_dir_path(&self.file)
            .context("sass file not found")?;
        let sass_string = fs::read_to_string(&file)?;

        if self.optimize {
            let css_style = grass::from_string(sass_string, &Default::default())?;

            let stylesheet =
                StyleSheet::parse(&css_style, Default::default()).map_err(|e| anyhow!("{e}"))?;

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
            Ok(out_css.code)
        } else if let Ok(sass_bin) = which("sass") {
            let cmd = Command::new(sass_bin)
                .args(["--embed-sources", "--embed-source-map", file.as_str()])
                .output()?;

            let out = String::from_utf8(cmd.stdout).unwrap();
            let err = String::from_utf8(cmd.stderr).unwrap();

            if !cmd.status.success() {
                bail!("installed binary sass failed with error: {err}{out}")
            }

            Ok(out)
        } else {
            Ok(grass::from_string(sass_string, &Default::default())?)
        }
    }

    pub fn watched(&self) -> String {
        self.file.to_string()
    }
}
