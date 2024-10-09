use std::process::{Command, ExitCode};

use camino::Utf8PathBuf;
use clap::Parser;
use common::{
    dir,
    out::{self, OutputParams},
    setup_logging,
};
use lightningcss::{
    printer::PrinterOptions,
    stylesheet::StyleSheet,
    targets::{Browsers, Targets},
};

fn main() -> ExitCode {
    let args = Cli::parse();
    run(args);
    ExitCode::SUCCESS
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long, value_name = "FILE")]
    in_scss: Utf8PathBuf,

    #[arg(short, long, value_name = "FILE")]
    /// Output css file
    out_css: Utf8PathBuf,

    #[arg(long)]
    optimize: bool,

    #[arg(long)]
    no_brotli: bool,

    #[arg(long)]
    no_gzip: bool,

    #[arg(long)]
    no_uncompressed: bool,

    #[arg(long)]
    no_checksum: bool,

    #[arg(short, long)]
    verbose: bool,
}

impl OutputParams for Cli {
    fn brotli(&self) -> bool {
        !self.no_brotli
    }
    fn gzip(&self) -> bool {
        !self.no_gzip
    }
    fn uncompressed(&self) -> bool {
        !self.no_uncompressed
    }
    fn checksum(&self) -> bool {
        !self.no_checksum
    }
}

fn run(cli: Cli) {
    setup_logging(cli.verbose);

    log::info!("Running builder-sass");

    dir::remove_files_containing(
        cli.out_css.parent().unwrap(),
        cli.out_css.file_name().unwrap(),
    );

    log::info!("Running sass on {}", cli.in_scss);
    let cmd = Command::new("sass")
        .args([
            "--embed-sources",
            "--embed-source-map",
            cli.in_scss.as_str(),
        ])
        .output()
        .unwrap();

    let out = String::from_utf8(cmd.stdout).unwrap();
    let err = String::from_utf8(cmd.stderr).unwrap();

    if !cmd.status.success() {
        panic!("installed binary sass failed: {err}{out}")
    }

    if cli.optimize {
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
        out::write(&cli, out_css.code.as_bytes(), &cli.out_css);
    } else {
        out::write(&cli, out.as_bytes(), &cli.out_css);
    }
}
