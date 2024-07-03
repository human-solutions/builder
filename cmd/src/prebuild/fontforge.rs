use crate::anyhow::{bail, Context, Result};
use crate::util::filehash;
use crate::Config;
use camino::{Utf8Path, Utf8PathBuf};
use fs_err as fs;
use serde::Deserialize;
use std::process::Command;
use which::which;

#[derive(Debug, Default, Deserialize)]
#[serde(transparent)]
pub struct FontForge {
    pub file: Utf8PathBuf,
}

impl FontForge {
    pub fn process(&self, info: &Config) -> Result<()> {
        let sum_file = info.args.dir.join(self.file.with_extension("sfd.hash"));
        let sfd_file = info.args.dir.join(&self.file);

        // check if sfd file exists
        if !sfd_file.exists() {
            bail!("sfd file not found: {sfd_file}");
        }

        let hash = filehash(&sfd_file)?;

        if !sum_file.exists() {
            fs::write(&sum_file, hash.as_bytes())?;
            return Ok(());
        }

        let current_hash = fs::read_to_string(&sum_file)?;

        // only update if changed (cargo detects changes by file modification time)
        // but we need to check the hash of the fontforge file in order to detect changes
        // so that we don't unnecessarily update the woff2 file. Note that the woff2 file
        // changes everytime it is generated, so we can't use it to detect changes.
        if hash != current_hash {
            self.generate(info)?;
            Ok(fs::write(&sum_file, hash.as_bytes())?)
        } else {
            Ok(())
        }
    }

    fn generate(&self, info: &Config) -> Result<()> {
        let Ok(command) = which("fontforge") else {
            println!("cargo::warning=fontforge command not found, skipping woff2 update");
            return Ok(());
        };
        let name = self.file.to_string();
        let woff = self.file.with_extension("woff2");
        let otf = self.file.with_extension("otf");
        let ff = format!("Open('{name}'); Generate('{woff}'); Generate('{otf}')");

        let cmd = Command::new(command)
            .args(["-lang=ff", "-c", &ff])
            .current_dir(info.args.dir.as_path())
            .output()
            .context("fontforge command failed")?;
        let out = String::from_utf8(cmd.stdout).unwrap();
        let err = String::from_utf8(cmd.stderr).unwrap();

        if !cmd.status.success() {
            bail!("installed binary fontforge failed with error: {err}{out}")
        }

        let otf_file = info.args.dir.join(otf);

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
