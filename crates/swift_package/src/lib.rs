use builder_command::SwiftPackageCmd;
use common::{is_release, is_verbose};
use swift_package::{CliArgs, build_cli};

pub fn run(cmd: &SwiftPackageCmd) {
    let cli = CliArgs {
        quiet: false,
        package: None,
        verbose: if is_verbose() { 1 } else { 0 },
        unstable_flags: None,
        release: is_release(),
        profile: None,
        features: vec![],
        all_features: false,
        no_default_features: false,
        target_dir: None,
        manifest_path: Some(cmd.manifest_dir.clone()),
    };
    build_cli(cli).unwrap()
}
