use builder_command::SassCmd;
use common::site_fs::{write_file_to_site, SiteFile};
use lightningcss::{
    printer::PrinterOptions,
    stylesheet::StyleSheet,
    targets::{Browsers, Targets},
};
use std::process::Command;
use which::which;

pub fn run(sass_cmd: &SassCmd) {
    log::info!("Running builder-sass on {}", sass_cmd.in_scss);

    let mut css = if let Ok(sass) = which("sass") {
        log::debug!("Compiling sass with {sass:?}");

        let cmd = Command::new(sass)
            .args([
                "--embed-sources",
                "--embed-source-map",
                sass_cmd.in_scss.as_str(),
            ])
            .output()
            .unwrap();
        let out = String::from_utf8(cmd.stdout).unwrap();
        let err = String::from_utf8(cmd.stderr).unwrap();

        if !cmd.status.success() {
            panic!("installed binary sass failed: {err}{out}")
        }
        out
    } else {
        log::debug!("No installed sass compiler found. Compiling with builtin grass compiler");
        grass::from_path(&sass_cmd.in_scss, &Default::default()).unwrap()
    };
    for (from, to) in &sass_cmd.replacements {
        css = css.replace(from, to);
    }

    let site_file = SiteFile::new(sass_cmd.in_scss.file_stem().unwrap(), "css");

    if sass_cmd.optimize {
        log::info!("Optimizing css");

        let stylesheet = StyleSheet::parse(&css, Default::default()).unwrap();

        let targets = Targets {
            browsers: Browsers::from_browserslist([
                ">0.3%, defaults, supports es6-module, maintained node versions",
            ])
            .unwrap(),
            ..Default::default()
        };

        let out_css = stylesheet
            .to_css(PrinterOptions {
                minify: true,
                targets,
                ..Default::default()
            })
            .unwrap();
        write_file_to_site(&site_file, out_css.code.as_bytes(), &sass_cmd.output);
    } else {
        write_file_to_site(&site_file, css.as_bytes(), &sass_cmd.output);
    }
}
