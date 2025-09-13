#[cfg(test)]
mod tests;

use builder_command::LocalizedCmd;
use camino_fs::*;
use common::site_fs::write_translations;
use common::{Timer, log_command, log_operation, log_trace};
use icu_locid::LanguageIdentifier;

pub fn run(cmd: &mut LocalizedCmd) {
    let _timer = Timer::new("LOCALIZED processing");
    log_command!(
        "LOCALIZED",
        "Processing localized files from: {}",
        cmd.input_dir
    );
    log_operation!(
        "LOCALIZED",
        "File extension: {}, Output destinations: {}",
        cmd.file_extension,
        cmd.output.len()
    );

    let variants = get_variants(cmd);
    log_operation!("LOCALIZED", "Found {} language variants", variants.len());

    if variants.is_empty() {
        log_command!("LOCALIZED", "No matching files found, skipping processing");
        return;
    }

    for (lang, content) in &variants {
        log_trace!("LOCALIZED", "Variant: {} ({} bytes)", lang, content.len());
    }

    let name = format!(
        "{name}.{ext}",
        name = cmd.input_dir.file_name().unwrap(),
        ext = cmd.file_extension
    );

    log_operation!("LOCALIZED", "Writing translations as: {}", name);
    write_translations(&name, &variants, &mut cmd.output);
}

fn get_variants(cmd: &LocalizedCmd) -> Vec<(LanguageIdentifier, Vec<u8>)> {
    let mut variants: Vec<(LanguageIdentifier, Vec<u8>)> = Vec::new();

    // list all file names in folder
    for file in cmd.input_dir.ls() {
        let file_extension_match = file
            .extension()
            .map(|ext| ext == cmd.file_extension)
            .unwrap_or_default();

        if file.is_file() && file_extension_match {
            let loc = file.file_stem().unwrap();
            let langid: LanguageIdentifier = loc.parse().unwrap();
            let content = file.read_bytes().unwrap();
            log_trace!(
                "LOCALIZED",
                "Processing file: {} -> language: {}",
                file,
                langid
            );
            variants.push((langid, content));
        } else if file.is_file() {
            log_trace!("LOCALIZED", "Skipping file (extension mismatch): {}", file);
        }
    }

    variants.sort_by(|a, b| a.0.total_cmp(&b.0));

    variants
}
