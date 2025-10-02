use builder_command::SwiftPackageCmd;
use common::{Timer, is_release, is_verbose, log_command, log_operation};
use swift_package::{CliArgs, build_cli};

pub fn run(cmd: &SwiftPackageCmd) {
    let _timer = Timer::new("SWIFT_PACKAGE processing");
    log_command!(
        "SWIFT_PACKAGE",
        "Building Swift package from: {}",
        cmd.manifest_dir
    );

    let verbose_level = if is_verbose() { 1 } else { 0 };
    let release_mode = is_release();

    log_operation!(
        "SWIFT_PACKAGE",
        "Configuration: release={}, verbose={}",
        release_mode,
        verbose_level > 0
    );

    // manifest_path must point to Cargo.toml file, not just the directory
    let manifest_path = cmd.manifest_dir.join("Cargo.toml");
    let cli = CliArgs {
        quiet: false,
        package: None,
        verbose: verbose_level,
        unstable_flags: None,
        release: release_mode,
        profile: None,
        features: vec![],
        all_features: false,
        no_default_features: false,
        target_dir: None,
        manifest_path: Some(manifest_path),
    };

    log_operation!("SWIFT_PACKAGE", "Executing swift-package build command");
    build_cli(cli).unwrap();
    log_operation!(
        "SWIFT_PACKAGE",
        "Swift package build completed successfully"
    );
}
