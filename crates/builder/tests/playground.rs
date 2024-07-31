use std::process::Command;

use camino::Utf8PathBuf;

const BIN: &str = env!("CARGO_BIN_EXE_builder");

#[test]
fn test_builder_cmdargs() {
    cargo(["clean"]);
    cargo(["build"]);
}

fn cargo<I, S>(args: I)
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    let bin_path = Utf8PathBuf::from(BIN);
    assert!(bin_path.exists());

    let path_env = std::env::var("PATH").unwrap();
    let new_path = format!("{}:{path_env}", bin_path.parent().unwrap());
    // println!("new path: {new_path}");

    let dir = Utf8PathBuf::from("../../examples/playground");

    let out = Command::new("cargo")
        .args(args)
        .current_dir(&dir)
        .env("PATH", new_path)
        .output()
        .unwrap();
    println!("{}", String::from_utf8(out.stderr).unwrap());
    println!("{}", String::from_utf8(out.stdout).unwrap());

    assert!(out.status.success());
}
