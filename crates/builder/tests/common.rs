use anyhow::Result;
use camino::{Utf8Path, Utf8PathBuf};
use std::process::Command;

const BIN: &str = env!("CARGO_BIN_EXE_builder");

pub fn cargo<I, S>(dir: &Utf8Path, args: I)
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    let bin_path = Utf8PathBuf::from(BIN);
    assert!(bin_path.exists());

    let path_env = std::env::var("PATH").unwrap();
    let new_path = format!("{}:{path_env}", bin_path.parent().unwrap());
    // println!("new path: {new_path}");

    let out = Command::new("cargo")
        .args(args)
        .current_dir(dir)
        .env("PATH", new_path)
        .output()
        .unwrap();
    println!("{}", String::from_utf8(out.stderr).unwrap());
    println!("{}", String::from_utf8(out.stdout).unwrap());

    assert!(out.status.success());
}

pub trait PathExt {
    fn ls_ascii(&self, indent: usize) -> Result<String>;
    fn ls_no_checksum(&self) -> Result<String>;
}

impl PathExt for Utf8PathBuf {
    fn ls_ascii(&self, indent: usize) -> Result<String> {
        let mut entries = self.read_dir_utf8()?;
        let mut out = Vec::new();

        out.push(format!(
            "{}{}:",
            "  ".repeat(indent),
            self.file_name().unwrap_or_default()
        ));

        let indent = indent + 1;
        let mut files = Vec::new();
        let mut dirs = Vec::new();

        while let Some(Ok(entry)) = entries.next() {
            let path = entry.path().to_path_buf();

            if entry.file_type()?.is_dir() {
                dirs.push(path);
            } else {
                files.push(path);
            }
        }

        dirs.sort();
        files.sort();

        for file in files {
            out.push(format!(
                "{}{}",
                "  ".repeat(indent),
                file.file_name().unwrap_or_default()
            ));
        }

        for path in dirs {
            out.push(path.ls_ascii(indent)?);
        }
        Ok(out.join("\n"))
    }

    fn ls_no_checksum(&self) -> Result<String> {
        let mut files = Vec::new();

        gather_files(self, &mut files, "")?;

        files.sort();

        Ok(files.join("\n"))
    }
}

fn gather_files(path: &Utf8PathBuf, files: &mut Vec<String>, ancestors: &str) -> Result<()> {
    let parent = format!("{ancestors}/{}", path.file_name().unwrap_or_default());
    let mut entries = path.read_dir_utf8()?;
    while let Some(Ok(entry)) = entries.next() {
        let path = entry.path().to_path_buf();
        if entry.file_type()?.is_dir() {
            gather_files(&path, files, &parent)?;
        } else {
            let filename = path.file_name().unwrap_or_default();
            let filename = if let Some((_, n)) = filename.split_once('=') {
                n
            } else {
                filename
            };
            files.push(format!("{parent}/{filename}"))
        }
    }
    Ok(())
}
