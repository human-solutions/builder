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
    fn ls_ascii_replace_checksum(
        &self,
        indent: usize,
        keys: &[&str],
        replacement: &str,
    ) -> Result<String>;

    fn ls_replace_checksum(&self, replacement: &str) -> Result<String>;
}

impl PathExt for Utf8PathBuf {
    fn ls_ascii_replace_checksum(
        &self,
        indent: usize,
        keys: &[&str],
        replacement: &str,
    ) -> Result<String> {
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
            let filename = replace_checksum(
                file.file_name().unwrap_or_default(),
                |c, n| keys.contains(&n) && !c.is_empty(),
                replacement,
            );
            out.push(format!("{}{}", "  ".repeat(indent), filename));
        }

        for path in dirs {
            out.push(path.ls_ascii_replace_checksum(indent, keys, replacement)?);
        }
        Ok(out.join("\n"))
    }

    fn ls_replace_checksum(&self, replacement: &str) -> Result<String> {
        let mut files = Vec::new();

        gather_files(self, &mut files, "", replacement)?;

        files.sort();

        Ok(files.join("\n"))
    }
}

fn replace_checksum<C>(filename: &str, condition: C, replacement: &str) -> String
where
    C: Fn(&str, &str) -> bool,
{
    if let Some((c, n)) = filename.split_once('=') {
        if condition(c, n) {
            return format!("{replacement}{n}");
        }
    }
    filename.to_owned()
}

fn gather_files(
    path: &Utf8PathBuf,
    files: &mut Vec<String>,
    ancestors: &str,
    replacement: &str,
) -> Result<()> {
    let parent = format!("{ancestors}/{}", path.file_name().unwrap_or_default());
    let mut entries = path.read_dir_utf8()?;
    while let Some(Ok(entry)) = entries.next() {
        let path = entry.path().to_path_buf();
        if entry.file_type()?.is_dir() {
            gather_files(&path, files, &parent, replacement)?;
        } else {
            let filename = replace_checksum(
                path.file_name().unwrap_or_default(),
                |c, n| !c.is_empty() && !n.is_empty(),
                replacement,
            );
            files.push(format!("{parent}/{filename}"))
        }
    }
    Ok(())
}
