use std::process::Command;

use builder_command::FontForgeCmd;
use camino_fs::*;
use common::site_fs::{SiteFile, write_file_to_site};

pub fn run(cmd: &FontForgeCmd) {
    log::info!("Running builder-fontforge");
    let sfd_file = Utf8Path::new(&cmd.font_file);
    let sum_file = sfd_file.with_extension("hash");
    let name = sfd_file.file_stem().unwrap();

    if !sfd_file.exists() {
        panic!("File not found: {:?}", sfd_file);
    }
    let sfd_bytes = sfd_file.read_bytes().unwrap();
    let hash = format!("{:x}", seahash::hash(&sfd_bytes));

    let sfd_dir = sfd_file.parent().unwrap();

    let generate_woff2 = if sum_file.exists() {
        log::debug!("Reading hash from {sum_file}");
        let current_hash = sum_file.read_string().unwrap();
        hash != current_hash
    } else {
        true
    };

    if generate_woff2 {
        generate_woff2_otf(sfd_dir, name);

        log::debug!("Writing hash to {sum_file}");
        sum_file.write(hash).unwrap();

        let otf_file = sfd_dir.join(name).with_extension("otf");

        // copy otf file to font directory (only macos)
        if cfg!(target_os = "macos") {
            macos_install_font(&otf_file, name);
        }
        otf_file.rm().unwrap();
        log::info!("Removed {otf_file}");
    } else {
        log::info!("No change detected, skipping {sfd_file}");
    }

    let woff2_filename = format!("{name}.woff2");
    let bytes = sfd_dir.join(&woff2_filename).read_bytes().unwrap();

    log::info!("Generating output for {name}");
    let site_file = SiteFile::new(name, "woff2");
    write_file_to_site(&site_file, &bytes, &cmd.output);
}

fn generate_woff2_otf(sfd_dir: &Utf8Path, name: &str) {
    let ff = format!("Open('{name}.sfd'); Generate('{name}.woff2'); Generate('{name}.otf')");

    log::info!("Generating {name}.woff2 and {name}.otf from {name}.sfd");
    let cmd = Command::new("fontforge")
        .args(["-lang=ff", "-c", &ff])
        .current_dir(sfd_dir)
        .status()
        .unwrap();

    if !cmd.success() {
        panic!("installed binary fontforge failed")
    }
}

fn macos_install_font(otf_file: &Utf8Path, name: &str) {
    log::info!("Copying polygot.otf to ~/Library/Fonts");
    let home = std::env::var("HOME").unwrap();
    let dest = Utf8Path::new(&home)
        .join("Library/Fonts")
        .join(name)
        .with_extension("otf");
    otf_file.cp(dest).unwrap();
}
