use std::process::Command;

use builder_command::SassCmd;
use common::out;
use lightningcss::{
    printer::PrinterOptions,
    stylesheet::StyleSheet,
    targets::{Browsers, Targets},
};

pub fn run(sass_cmd: &SassCmd) {
    log::info!("Running builder-sass");

    log::info!("Running sass on {}", sass_cmd.in_scss);
    let cmd = Command::new("sass")
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
    let name = format!("{}.css", sass_cmd.in_scss.file_stem().unwrap());

    if sass_cmd.optimize {
        log::info!("Optimizing css");

        let stylesheet = StyleSheet::parse(&out, Default::default()).unwrap();

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
        out::write(&sass_cmd.output, out_css.code.as_bytes(), &name);
    } else {
        out::write(&sass_cmd.output, out.as_bytes(), &name);
    }
}
