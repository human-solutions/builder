use std::process::Command;

use anyhow::{anyhow, bail, Result};
use fs_err as fs;

use lightningcss::{
    printer::PrinterOptions,
    stylesheet::StyleSheet,
    targets::{Browsers, Targets},
};
use which::which;

use crate::{config::Sass, RuntimeInfo};

impl Sass {
    pub fn process(&self, info: &RuntimeInfo) -> Result<String> {
        let file = if self.file.is_relative() {
            info.manifest_dir.join(&self.file)
        } else {
            self.file.clone()
        };
        if !file.exists() {
            bail!("The sass file {file} doesn't exist");
        }
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
