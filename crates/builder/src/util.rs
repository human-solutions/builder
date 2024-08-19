use std::io::{self, BufRead};
use std::process::{Command, Stdio};
use std::time::{Duration, SystemTime};

use crate::anyhow::Result;
use anyhow::Context;
use base64::engine::general_purpose::URL_SAFE;
use base64::prelude::*;
use camino::Utf8Path;
use fs_err as fs;

pub fn filehash(file: &Utf8Path) -> Result<String> {
    let content = fs::read(file)?;
    let hash = seahash::hash(&content);
    Ok(hash.to_string())
}

pub fn timehash() -> String {
    let epoch_to_y2k: Duration = Duration::from_secs(946_684_800);
    let epoch = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap();
    let secs = (epoch - epoch_to_y2k).as_secs();

    let bytes = secs.to_be_bytes();
    let mut start = 0;
    while bytes[start] == 0 {
        start += 1;
    }
    URL_SAFE.encode(&bytes[start..])
}

pub fn run_cmd(cmds: &[&str]) -> Result<()> {
    let cmd = cmds[0];
    let args = &cmds[1..];
    let mut child = Command::new(cmd)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context(format!("Failed to run command '{cmd}' with args {args:?}",))?;

    if let Some(stdout) = child.stdout.take() {
        let reader = io::BufReader::new(stdout);
        for line in reader.lines() {
            println!("{}", line?);
        }
    }
    if let Some(stderr) = child.stderr.take() {
        let reader = io::BufReader::new(stderr);
        for line in reader.lines() {
            println!("{}", line?);
        }
    }

    let status = child
        .wait()
        .context(format!("Failed to wait for command {cmd}"))?;
    if !status.success() {
        anyhow::bail!(format!("Failed to run command '{cmd}' with args {args:?}",));
    }
    Ok(())
}
