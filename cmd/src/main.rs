use std::process::ExitCode;

use anyhow::Result;
use builder::RuntimeInfo;

fn main() -> ExitCode {
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
