use builder_command::{BuilderCmd, Cmd};
use common::{LOG_LEVEL, RELEASE, asset_code_generation, setup_logging, site_fs};

pub use builder_command;

/// Execute BuilderCmd directly in-process
pub fn execute(builder: BuilderCmd) {
    RELEASE.set(builder.release).ok();
    setup_logging(builder.log_level, builder.log_destination.clone());
    LOG_LEVEL.set(builder.log_level).ok();

    run_commands(builder);
}

/// Execute commands from a mutable BuilderCmd reference
fn run_commands(mut builder: BuilderCmd) {
    for cmd in &mut builder.cmds {
        match cmd {
            Cmd::Uniffi(cmd) => builder_uniffi::run(cmd),
            Cmd::Sass(cmd) => builder_sass::run(cmd),
            Cmd::Localized(cmd) => builder_localized::run(cmd),
            Cmd::FontForge(cmd) => builder_fontforge::run(cmd),
            Cmd::Wasm(cmd) => builder_wasm::run(cmd),
            Cmd::Copy(cmd) => builder_copy::run(cmd),
            Cmd::SwiftPackage(cmd) => builder_swift_package::run(cmd),
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
