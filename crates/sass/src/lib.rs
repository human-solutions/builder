use builder_command::SassCmd;
use common::site_fs::{SiteFile, write_file_to_site};
use common::{Timer, log_command, log_operation, log_trace};
use lightningcss::{
    printer::PrinterOptions,
    stylesheet::StyleSheet,
    targets::{Browsers, Targets},
};
use std::process::Command;
use which::which;

pub fn run(sass_cmd: &SassCmd) {
    let _timer = Timer::new("SASS processing");
    log_command!("SASS", "Processing file: {}", sass_cmd.in_scss);
    log_operation!(
        "SASS",
        "Optimize: {}, Replacements: {}",
        sass_cmd.optimize,
        sass_cmd.replacements.len()
    );

    let mut css = if let Ok(sass) = which("sass") {
        log_operation!("SASS", "Compiling with external sass binary: {:?}", sass);

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
            panic!("External sass binary failed: {err}{out}")
        }
        log_operation!(
            "SASS",
            "External sass compilation successful ({} bytes)",
            out.len()
        );
        out
    } else {
        log_operation!(
            "SASS",
            "Using built-in grass compiler (no external sass found)"
        );
        let css = grass::from_path(&sass_cmd.in_scss, &Default::default()).unwrap();
        log_operation!("SASS", "Grass compilation successful ({} bytes)", css.len());
        css
    };
    if !sass_cmd.replacements.is_empty() {
        let original_len = css.len();
        for (from, to) in &sass_cmd.replacements {
            css = css.replace(from, to);
            log_trace!("SASS", "Replacement: '{}' -> '{}'", from, to);
        }
        log_operation!(
            "SASS",
            "Applied {} replacements ({} -> {} bytes)",
            sass_cmd.replacements.len(),
            original_len,
            css.len()
        );
    }

    let site_file = SiteFile::new(sass_cmd.in_scss.file_stem().unwrap(), "css");

    if sass_cmd.optimize {
        log_operation!("SASS", "Optimizing CSS with Lightning CSS");

        let stylesheet = StyleSheet::parse(&css, Default::default()).unwrap();

        let targets = Targets {
            browsers: Browsers::from_browserslist([
                ">0.3%, defaults, supports es6-module, maintained node versions",
            ])
            .unwrap(),
            ..Default::default()
        };

        let original_size = css.len();
        let out_css = stylesheet
            .to_css(PrinterOptions {
                minify: true,
                targets,
                ..Default::default()
            })
            .unwrap();

        let savings =
            ((original_size - out_css.code.len()) as f64 / original_size as f64 * 100.0) as i32;
        log_operation!(
            "SASS",
            "CSS optimization complete: {} -> {} bytes ({}% reduction)",
            original_size,
            out_css.code.len(),
            savings
        );
        write_file_to_site(&site_file, out_css.code.as_bytes(), &sass_cmd.output);
    } else {
        log_operation!("SASS", "Writing unoptimized CSS ({} bytes)", css.len());
        write_file_to_site(&site_file, css.as_bytes(), &sass_cmd.output);
    }
}
