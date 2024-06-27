use crate::anyhow::{bail, Context, Result};
use camino::{Utf8Path, Utf8PathBuf};
use clap::Args;

#[derive(Args, Debug)]
pub struct RawPrebuildArgs {
    #[clap(long, env = "CARGO_MANIFEST_DIR")]
    pub manifest_dir: Utf8PathBuf,
    #[clap(long, env = "PROFILE")]
    pub profile: String,
    #[clap(long, env = "CARGO_PKG_NAME")]
    pub package: String,
    #[clap(long, env = "OUT_DIR")]
    pub out_dir: String,
}

impl TryInto<PrebuildArgs> for RawPrebuildArgs {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<PrebuildArgs> {
        Ok(PrebuildArgs {
            manifest_dir: self.manifest_dir,
            profile: self.profile,
            package: self.package,
            out_dir: target_dir(&self.out_dir)?,
        })
    }
}

pub struct PrebuildArgs {
    pub manifest_dir: Utf8PathBuf,
    pub profile: String,
    pub package: String,
    pub out_dir: Utf8PathBuf,
}

fn target_dir(out_dir: &str) -> Result<Utf8PathBuf> {
    let target_dir_pos = out_dir
        .find("target")
        .with_context(|| format!("Expected to find 'target' in OUT_DIR: '{out_dir}'"))?;

    let target_dir = &out_dir[..target_dir_pos + "target".len()];
    Ok(Utf8PathBuf::from(target_dir))
}

impl PrebuildArgs {
    pub fn site_dir(&self, assembly: &str) -> Utf8PathBuf {
        self.out_dir
            .join(&self.package)
            .join(assembly)
            .join(&self.profile)
    }

    pub fn existing_manifest_dir_path(&self, path: &Utf8Path) -> Result<Utf8PathBuf> {
        let file = if path.is_relative() {
            self.manifest_dir.join(path)
        } else {
            bail!("The path {path} must be relative to the manifest directory")
        };

        if !file.exists() {
            bail!("The path {file} doesn't exist");
        }
        Ok(file)
    }
}
