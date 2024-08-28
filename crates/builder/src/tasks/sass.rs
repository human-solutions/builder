use std::{collections::HashSet, process::Command};

use crate::{
    anyhow::{anyhow, bail, Context, Result},
    generate::{Asset, Generator},
};
use camino::Utf8PathBuf;
use lightningcss::{
    printer::PrinterOptions,
    stylesheet::StyleSheet,
    targets::{Browsers, Targets},
};
use serde::{Deserialize, Serialize};

use crate::generate::Output;

use super::setup::Config;

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct SassParams {
    pub file: Utf8PathBuf,
    pub optimize: bool,
    pub out: Output,
}

impl SassParams {
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

    pub fn process(
        &self,
        config: &Config,
        generator: &mut Generator,
        watched: &mut HashSet<String>,
    ) -> Result<()> {
        let css = self.process_inner(config)?;
        let filename = self.file_name(&None);
        let hash = self
            .out
            .write_file(css.as_bytes(), &config.site_dir("sass"), &filename)?;

        generator.add_asset(Asset::from_sass(self, hash), None);
        watched.insert(self.watched());

        Ok(())
    }

    fn process_inner(&self, config: &Config) -> Result<String> {
        let file = config
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
