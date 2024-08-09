use std::{
    fs::OpenOptions,
    io::Write,
    process::{Command, ExitCode},
};

use which::which;

fn main() -> ExitCode {
    let mut file = OpenOptions::new()
        .append(true)
        .create(true) // Create the file if it doesn't exist
        .open("../target/test.txt")
        .unwrap();
    if which("builder").is_err() {
        println!("cargo::warning=builder command not found, skipping");
        return ExitCode::SUCCESS;
    }

    writeln!(file, "\nassets").unwrap();
    match Command::new("builder").arg("prebuild").status() {
        Ok(status) if status.success() => ExitCode::SUCCESS,
        Ok(_) => ExitCode::FAILURE,
        Err(e) => {
            eprintln!("Failed to run builder: {}", e);
            ExitCode::FAILURE
        }
    }
}
