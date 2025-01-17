#[cfg(test)]
mod tests;

use builder_command::LocalizedCmd;
use camino_fs::*;
use common::site_fs::write_translations;
use icu_locid::LanguageIdentifier;

pub fn run(cmd: &LocalizedCmd) {
    log::info!("Running builder-localized");

    let variants = get_variants(cmd);
    let name = format!(
        "{name}.{ext}",
        name = cmd.input_dir.file_name().unwrap(),
        ext = cmd.file_extension
    );
    write_translations(&name, &variants, &cmd.output);
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
            variants.push((langid, content));
        }
    }

    variants.sort_by(|a, b| a.0.total_cmp(&b.0));

    variants
}
