use std::{
    env,
    process::{Command, ExitCode},
};

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
    let build_profile = env("PROFILE")?;
    let manifest_dir = env("CARGO_MANIFEST_DIR")?;
    let out_dir = env("OUT_DIR")?;
    let pkg_name = env("CARGO_PKG_NAME")?;
    if which("builder").is_err() {
        println!("cargo::warning=builder command not found, skipping");
        return Ok(ExitCode::SUCCESS);
    }

    let output = Command::new("builder")
        .env("BUILDER_PKG_NAME", pkg_name)
        .env("BUILDER_PROFILE", build_profile)
        .env("BUILDER_MANIFEST_DIR", manifest_dir)
        .env("BUILDER_OUT_DIR", out_dir)
        .output()
        .map_err(|e| e.to_string())?;

    // forward builder output to cargo
    eprintln!("{}", String::from_utf8_lossy(&output.stderr));
    println!("{}", String::from_utf8_lossy(&output.stdout));
    Ok(if output.status.success() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    })
}

fn env(key: &str) -> Result<String, String> {
    let var = env::var(key).map_err(|e| format!("Could not resolve env {key}: {e}"))?;
    println!("{key}: {var}");
    Ok(var)
}
