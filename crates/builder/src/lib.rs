use builder_command::{BuilderCmd, Cmd};
use builder_mtimes::{
    FileLock, InputFiles, OutputFiles, SkipDecision, record_success, should_skip,
};
use common::{LOG_LEVEL, RELEASE, asset_code_generation, setup_logging, site_fs};

pub use builder_command;

/// Execute BuilderCmd directly in-process
pub fn execute(builder: BuilderCmd) {
    RELEASE.set(builder.release).ok();
    setup_logging(builder.log_level, builder.log_destination.clone());
    LOG_LEVEL.set(builder.log_level).ok();

    run_commands(builder);
}

/// Process a single command with all its operations in sequence
fn process_command<T>(cmd: &mut T, runner: impl FnOnce(&mut T))
where
    T: builder_command::CommandMetadata + serde::Serialize + InputFiles + OutputFiles,
{
    use camino_fs::Utf8PathExt;

    // Clone the output directory to avoid borrow checker issues
    let output_dir = cmd.output_dir().to_path_buf();

    // Ensure the output directory exists before trying to create the lock file
    if let Err(e) = output_dir.mkdirs() {
        log::error!("Failed to create output directory: {}", e);
    } else {
        let lock_path = output_dir.join(".builder-lock");

        match FileLock::acquire(&lock_path) {
            Ok(_lock) => {
                let should_execute = match should_skip(cmd, cmd.name(), &output_dir) {
                    Ok(SkipDecision::Skip { reason }) => {
                        log::info!("{}: Skipped: {}", cmd.name().to_uppercase(), reason);
                        false
                    }
                    Ok(SkipDecision::Execute { reason }) => {
                        log::debug!("{}: Executing: {}", cmd.name().to_uppercase(), reason);
                        true
                    }
                    Err(e) => {
                        log::warn!(
                            "{}: Change detection failed, executing: {}",
                            cmd.name().to_uppercase(),
                            e
                        );
                        true
                    }
                };

                if should_execute {
                    runner(cmd);

                    if let Err(e) = record_success(cmd, cmd.name(), &output_dir) {
                        log::warn!("Failed to record success: {}", e);
                    }
                }
            }
            Err(e) => {
                log::error!("Failed to acquire lock: {}", e);
            }
        }
    }
}

/// Execute commands from a mutable BuilderCmd reference
fn run_commands(mut builder: BuilderCmd) {
    for cmd in &mut builder.cmds {
        match cmd {
            Cmd::Sass(sass_cmd) => process_command(sass_cmd, builder_sass::run),
            Cmd::Copy(copy_cmd) => process_command(copy_cmd, builder_copy::run),
            Cmd::Wasm(wasm_cmd) => process_command(wasm_cmd, builder_wasm::run),
            Cmd::Uniffi(uniffi_cmd) => process_command(uniffi_cmd, |c| builder_uniffi::run(c)),
            Cmd::FontForge(font_cmd) => process_command(font_cmd, builder_fontforge::run),
            Cmd::Localized(loc_cmd) => process_command(loc_cmd, builder_localized::run),
            Cmd::SwiftPackage(swift_cmd) => {
                process_command(swift_cmd, |c| builder_swift_package::run(c))
            }
        }
    }

    // Finalize hash output files after all commands have completed
    if let Err(e) = site_fs::finalize_hash_outputs() {
        eprintln!("Failed to write hash output files: {}", e);
    }

    // Finalize asset code generation after all commands have completed
    if let Err(e) = asset_code_generation::finalize_asset_code_outputs() {
        eprintln!("Failed to write asset code files: {}", e);
    }
}
