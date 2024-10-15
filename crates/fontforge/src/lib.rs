use std::process::Command;

use builder_command::FontForgeCmd;
use camino::Utf8Path;
use common::out;
use fs_err as fs;

pub fn run(cmd: &FontForgeCmd) {
    log::info!("Running builder-fontforge");
    let sfd_file = Utf8Path::new(&cmd.font_file);
    let sum_file = sfd_file.with_extension("hash");
    let name = sfd_file.file_stem().unwrap();

    if !sfd_file.exists() {
        panic!("File not found: {:?}", sfd_file);
    }
    let sfd_bytes = fs::read(sfd_file).unwrap();
    let hash = format!("{:x}", seahash::hash(&sfd_bytes));

    let sfd_dir = sfd_file.parent().unwrap();

    let generate_woff2 = if sum_file.exists() {
        log::debug!("Reading hash from {sum_file}");
        let current_hash = fs::read_to_string(&sum_file).unwrap();
        hash != current_hash
    } else {
        true
    };

    if generate_woff2 {
        generate_woff2_otf(sfd_dir, name);

        log::debug!("Writing hash to {sum_file}");
        fs::write(sum_file, hash.as_bytes()).unwrap();

        let otf_file = sfd_dir.join(name).with_extension("otf");

        // copy otf file to font directory (only macos)
        if cfg!(target_os = "macos") {
            macos_install_font(&otf_file, name);
        }
        fs::remove_file(&otf_file).unwrap();
        log::info!("Removed {otf_file}");
    } else {
        log::info!("No change detected, skipping {sfd_file}");
    }

    let contents = fs::read(&sfd_dir.join(name).with_extension("woff2")).unwrap();

    log::info!("Generating output for {name}");
    out::write(
        &cmd.output,
        &contents,
        &Utf8Path::new(name).with_extension("woff2"),
    );
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
    fs::copy(&otf_file, dest).unwrap();
}
