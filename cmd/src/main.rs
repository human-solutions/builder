use anyhow::Result;
use builder::RuntimeInfo;
use std::env;
use std::process::ExitCode;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.contains(&"-v".to_string()) || args.contains(&"--version".to_string()) {
        println!("builder {}", VERSION);
        return ExitCode::SUCCESS;
    }
    match try_main() {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("builder error: {e:?}");
            ExitCode::FAILURE
        }
    }
}

fn try_main() -> Result<()> {
    let info = RuntimeInfo::from_env()?;
    let manifest = builder::Manifest::try_parse(&info)?;
    manifest.process(&info)
}
