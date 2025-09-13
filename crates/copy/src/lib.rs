use builder_command::CopyCmd;
use common::site_fs::copy_files_to_site;
use common::{Timer, log_command, log_operation};

pub fn run(cmd: &mut CopyCmd) {
    let _timer = Timer::new("COPY processing");
    log_command!("COPY", "Copying files from: {}", cmd.src_dir);
    log_operation!(
        "COPY",
        "Recursive: {}, Extensions: {:?}",
        cmd.recursive,
        cmd.file_extensions
    );
    log_operation!("COPY", "Output destinations: {}", cmd.output.len());

    if !cmd.src_dir.exists() {
        log_command!("COPY", "Source directory not found: {}", cmd.src_dir);
        return;
    }

    copy_files_to_site(
        &cmd.src_dir,
        cmd.recursive,
        |file| {
            file.extension()
                .is_some_and(|ext| cmd.file_extensions.contains(&ext.to_string()))
        },
        &mut cmd.output,
    );
}
