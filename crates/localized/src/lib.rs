#[cfg(test)]
mod tests;

use builder_command::LocalizedCmd;
use common::{dir::remove_content_of_dir, out::write_checksummed_variants};
use fs_err as fs;
use icu_locid::LanguageIdentifier;

pub fn run(cmd: &LocalizedCmd) {
    log::info!("Running builder-localized");

    let variants = get_variants(cmd);

    for out in &cmd.output {
        remove_content_of_dir(&out.dir);

        if !out.dir.exists() {
            fs::create_dir_all(&out.dir).unwrap();
        }
        write_checksummed_variants(out, &cmd.file_extension, &variants);
    }
}

fn get_variants(cmd: &LocalizedCmd) -> Vec<(String, Vec<u8>)> {
    let mut variants: Vec<(String, Vec<u8>)> = Vec::new();

    // list all file names in folder
    for file in cmd.input_dir.read_dir_utf8().unwrap() {
        let file = file.unwrap();
        let file_type = file.file_type().unwrap();

        let file_extension_match = file
            .path()
            .extension()
            .map(|ext| ext == cmd.file_extension)
            .unwrap_or_default();

        if file_type.is_file() && file_extension_match {
            let loc = file.path().file_stem().unwrap();
            let langid: LanguageIdentifier = loc.parse().unwrap();
            let content = fs::read(file.path()).unwrap();
            variants.push((langid.to_string(), content));
        }
    }

    variants.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));

    variants
}
