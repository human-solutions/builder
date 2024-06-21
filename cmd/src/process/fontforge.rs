use crate::{config::FontForge, RuntimeInfo};
use anyhow::{bail, Context, Result};
use camino::Utf8Path;
use fs_err as fs;
use std::process::Command;
use which::which;

impl FontForge {
    pub fn process(&self, info: &RuntimeInfo) -> Result<()> {
        let Ok(command) = which("fontforge") else {
            println!("cargo::warning=fontforge command not found, skipping woff2 update");
            return Ok(());
        };

        let name = self.file.to_string();
        let woff = self.file.with_extension("new.woff2");
        let otf = self.file.with_extension("otf");
        let ff = format!("Open('{name}'); Generate('{woff}'); Generate('{otf}')");

        let cmd = Command::new(command)
            .args(["-lang=ff", "-c", &ff])
            .current_dir(info.manifest_dir.as_path())
            .output()
            .context("fontforge command failed")?;
        let out = String::from_utf8(cmd.stdout).unwrap();
        let err = String::from_utf8(cmd.stderr).unwrap();

        if !cmd.status.success() {
            bail!("installed binary fontforge failed with error: {err}{out}")
        }

        // only update if changed (cargo detects changes by file modification time)
        let woff_path = info.manifest_dir.join(self.file.with_extension(""));
        let woff_old = woff_path.with_extension("woff2");
        let woff_new = woff_path.with_extension("new.woff2");
        let woff_old_checksum = seahash::hash(&fs::read(&woff_old)?);
        let woff_new_checksum = seahash::hash(&fs::read(&woff_new)?);

        let otf_file = info.manifest_dir.join(otf);

        if woff_old_checksum == woff_new_checksum {
            fs::remove_file(woff_new)?;
            fs::remove_file(&otf_file)?;
            return Ok(());
        }

        fs::rename(woff_new, woff_old)?;

        // copy otf file to font directory (only macos)
        if cfg!(target_os = "macos") {
            let home = std::env::var("HOME").unwrap();
            let dest = Utf8Path::new(&home)
                .join("Library/Fonts")
                .join(otf_file.file_name().unwrap());
            fs::copy(&otf_file, dest)?;
        }
        fs::remove_file(&otf_file)?;
        println!("removed {otf_file}    ");
        Ok(())
    }
}
