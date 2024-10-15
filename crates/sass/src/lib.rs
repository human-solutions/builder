use builder_command::SassCmd;
use camino::Utf8Path;
use common::out;
use fs_err as fs;
use lightningcss::{
    printer::PrinterOptions,
    stylesheet::StyleSheet,
    targets::{Browsers, Targets},
};
use std::process::Command;
use which::which;

pub fn run(sass_cmd: &SassCmd) {
    log::info!("Running builder-sass");

    log::info!("Running sass on {}", sass_cmd.in_scss);
    let css = if let Ok(sass) = which("sass") {
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
        let scss = fs::read_to_string(&sass_cmd.in_scss).unwrap();
        grass::from_string(scss, &grass::Options::default()).unwrap()
    };

    let name = format!("{}.css", sass_cmd.in_scss.file_stem().unwrap());

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
        out::write(
            &sass_cmd.output,
            out_css.code.as_bytes(),
            &Utf8Path::new(&name),
        );
    } else {
        out::write(&sass_cmd.output, css.as_bytes(), &Utf8Path::new(&name));
    }
}
