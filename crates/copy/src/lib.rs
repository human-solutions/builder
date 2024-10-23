use builder_command::CopyCmd;
use camino::Utf8Path;
use common::{out, Utf8PathExt};
use fs_err as fs;

pub fn run(cmd: &CopyCmd) {
    log::info!("Running builder-copy");

    if !cmd.src_dir.exists() {
        log::warn!("Directory not found: {}", cmd.src_dir);
        return;
    }

    let to_copy = cmd.src_dir.ls_files_matching(|f| {
        f.extension()
            .map_or(false, |ext| cmd.file_extensions.contains(&ext.to_string()))
    });

    for file in to_copy {
        let content = fs::read(&file).unwrap();
        let filename = Utf8Path::new(file.file_name().unwrap());

        out::write(&cmd.output, &content, &filename);
    }
}
