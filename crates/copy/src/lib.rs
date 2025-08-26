use builder_command::CopyCmd;
use common::site_fs::copy_files_to_site;

pub fn run(cmd: &CopyCmd) {
    log::info!("Running builder-copy");

    if !cmd.src_dir.exists() {
        log::warn!("Directory not found: {}", cmd.src_dir);
        return;
    }

    copy_files_to_site(
        &cmd.src_dir,
        cmd.recursive,
        |file| {
            file.extension()
                .is_some_and(|ext| cmd.file_extensions.contains(&ext.to_string()))
        },
        &cmd.output,
    );
}
