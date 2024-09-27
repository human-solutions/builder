use anyhow::{Context, Result};
use camino::{Utf8Path, Utf8PathBuf};
use fs_err as fs;
use serde::{Deserialize, Serialize};
use std::process::Command;
use which::which;

use crate::util::filehash;

use super::setup::Config;

#[derive(Debug, Default, Deserialize, Serialize)]
pub(super) struct FontForgeParams {
    pub item: Utf8PathBuf,
}

impl FontForgeParams {
    pub fn process(&self, info: &Config) -> Result<()> {
        let sum_file = info.args.dir.join(self.item.with_extension("sfd.hash"));
        let sfd_file = info.args.dir.join(&self.item);

        // check if sfd file exists
        if !sfd_file.exists() {
            anyhow::bail!("sfd file not found: {sfd_file}");
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
        let name = self.item.to_string();
        let woff = self.item.with_extension("woff2");
        let otf = self.item.with_extension("otf");
        let ff = format!("Open('{name}'); Generate('{woff}'); Generate('{otf}')");

        let Ok(fontforge) = which("fontforge") else {
            anyhow::bail!("fontforge is not installed");
        };

        log::info!("Generating {woff} and {otf} from {name}");
        let cmd = Command::new(fontforge)
            .args(["-lang=ff", "-c", &ff])
            .current_dir(info.args.dir.as_path())
            .output()
            .context("fontforge command failed")?;
        let out = String::from_utf8(cmd.stdout).unwrap();
        let err = String::from_utf8(cmd.stderr).unwrap();

        if !cmd.status.success() {
            anyhow::bail!("installed binary fontforge failed with error: {err}{out}")
        }

        let otf_file = info.args.dir.join(otf);

        // copy otf file to font directory (only macos)
        if cfg!(target_os = "macos") {
            log::info!("Copying {otf_file} to ~/Library/Fonts");
            let home = std::env::var("HOME").unwrap();
            let dest = Utf8Path::new(&home)
                .join("Library/Fonts")
                .join(otf_file.file_name().unwrap());
            fs::copy(&otf_file, dest)?;
        }
        fs::remove_file(&otf_file).context(format!("Failed to delete font file {otf_file}"))?;
        log::info!("Removed {otf_file}");
        Ok(())
    }
}
