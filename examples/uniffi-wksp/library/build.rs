use std::process::{Command, ExitCode};

use which::which;

fn main() -> ExitCode {
    uniffi::generate_scaffolding("./src/library.udl").unwrap();
    if which("builder").is_err() {
        println!("cargo::warning=builder command not found, skipping");
        return ExitCode::SUCCESS;
    }
    match Command::new("builder").arg("prebuild").status() {
        Ok(status) if status.success() => ExitCode::SUCCESS,
        Ok(_) => ExitCode::FAILURE,
        Err(e) => {
            eprintln!("Failed to run builder: {}", e);
            ExitCode::FAILURE
        }
    }
}
