use std::process::Command;

use builder_command::FontForgeCmd;
use camino_fs::*;
use common::{Timer, log_command, log_operation, log_trace};
use common::site_fs::{SiteFile, write_file_to_site};

pub fn run(cmd: &FontForgeCmd) {
    let _timer = Timer::new("FONTFORGE processing");
    let sfd_file = Utf8Path::new(&cmd.font_file);
    let sum_file = sfd_file.with_extension("hash");
    let name = sfd_file.file_stem().unwrap();

    log_command!("FONTFORGE", "Processing font file: {}", sfd_file);
    log_operation!("FONTFORGE", "Output destinations: {}", cmd.output.len());

    if !sfd_file.exists() {
        panic!("Font file not found: {:?}", sfd_file);
    }
    
    let sfd_bytes = sfd_file.read_bytes().unwrap();
    let hash = format!("{:x}", seahash::hash(&sfd_bytes));
    log_operation!("FONTFORGE", "Font file hash: {} ({} bytes)", hash, sfd_bytes.len());

    let sfd_dir = sfd_file.parent().unwrap();

    let generate_woff2 = if sum_file.exists() {
        log_trace!("FONTFORGE", "Checking existing hash file: {}", sum_file);
        let current_hash = sum_file.read_string().unwrap();
        let needs_regeneration = hash != current_hash;
        if needs_regeneration {
            log_operation!("FONTFORGE", "Hash changed, regeneration needed");
        } else {
            log_operation!("FONTFORGE", "Hash unchanged, skipping regeneration");
        }
        needs_regeneration
    } else {
        log_operation!("FONTFORGE", "No hash file found, initial generation needed");
        true
    };

    if generate_woff2 {
        generate_woff2_otf(sfd_dir, name);

        log_trace!("FONTFORGE", "Writing hash to: {}", sum_file);
        sum_file.write(hash).unwrap();

        let otf_file = sfd_dir.join(name).with_extension("otf");

        // copy otf file to font directory (only macos)
        if cfg!(target_os = "macos") {
            log_operation!("FONTFORGE", "Installing font to macOS system (target_os=macos)");
            macos_install_font(&otf_file, name);
        }
        log_trace!("FONTFORGE", "Removing temporary OTF file: {}", otf_file);
        otf_file.rm().unwrap();
    } else {
        log_command!("FONTFORGE", "No changes detected, skipping generation");
    }

    let woff2_filename = format!("{name}.woff2");
    let woff2_path = sfd_dir.join(&woff2_filename);
    let bytes = woff2_path.read_bytes().unwrap();

    log_operation!("FONTFORGE", "Writing WOFF2 output: {} ({} bytes)", woff2_filename, bytes.len());
    let site_file = SiteFile::new(name, "woff2");
    write_file_to_site(&site_file, &bytes, &cmd.output);
}

fn generate_woff2_otf(sfd_dir: &Utf8Path, name: &str) {
    let ff = format!("Open('{name}.sfd'); Generate('{name}.woff2'); Generate('{name}.otf')");

    log_operation!("FONTFORGE", "Generating {}.woff2 and {}.otf from {}.sfd", name, name, name);
    log_trace!("FONTFORGE", "FontForge command: {}", ff);
    
    let cmd = Command::new("fontforge")
        .args(["-lang=ff", "-c", &ff])
        .current_dir(sfd_dir)
        .status()
        .unwrap();

    if !cmd.success() {
        panic!("FontForge command failed")
    }
    
    log_operation!("FONTFORGE", "FontForge generation completed successfully");
}

fn macos_install_font(otf_file: &Utf8Path, name: &str) {
    let home = std::env::var("HOME").unwrap();
    let dest = Utf8Path::new(&home)
        .join("Library/Fonts")
        .join(name)
        .with_extension("otf");
        
    log_trace!("FONTFORGE", "Installing font: {} -> {}", otf_file, dest);
    otf_file.cp(&dest).unwrap();
    log_operation!("FONTFORGE", "Font installed to macOS Library/Fonts");
}
