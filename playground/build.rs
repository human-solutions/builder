use std::process::{Command, ExitCode};

use which::which;

fn main() -> ExitCode {
    match try_main() {
        Ok(code) => code,
        Err(e) => {
            eprintln!("build.rs error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn try_main() -> Result<ExitCode, String> {
    if which("builder").is_err() {
        println!("cargo::warning=builder command not found, skipping");
        return Ok(ExitCode::SUCCESS);
    }

    let output = Command::new("builder")
        .arg("prebuild")
        .output()
        .map_err(|e| e.to_string())?;

    // forward builder output to cargo
    eprintln!("{}", String::from_utf8_lossy(&output.stderr));

    Ok(if output.status.success() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    })
}
