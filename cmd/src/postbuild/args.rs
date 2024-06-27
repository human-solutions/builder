use crate::anyhow::{bail, Context, Result};
use camino::{Utf8Path, Utf8PathBuf};
use clap::Args;

#[derive(Args, Debug)]
pub struct PostbuildArgs {
    #[clap(long, env = "BUILDER_MANIFEST_DIR")]
    pub manifest_dir: Utf8PathBuf,
    #[clap(long, env = "BUILDER_PROFILE")]
    pub profile: String,
    #[clap(long, env = "BUILDER_PKG_NAME")]
    pub package: String,
    #[clap(long, env = "BUILDER_OUT_DIR")]
    pub target_dir: Utf8PathBuf,
}

impl PostbuildArgs {
    pub fn from_env() -> Result<Self> {
        let profile = env("BUILDER_PROFILE")?;
        let package = env("BUILDER_PKG_NAME")?;
        let out_dir = env("BUILDER_OUT_DIR")?;
        let manifst = env("BUILDER_MANIFEST_DIR")?;

        let target_dir_pos = out_dir
            .find("target")
            .with_context(|| format!("Expected to find 'target' in OUT_DIR: '{out_dir}'"))?;

        let target_dir = &out_dir[..target_dir_pos + "target/".len()];
        let manifest_dir = Utf8PathBuf::from(manifst);
        let target_dir = Utf8PathBuf::from(target_dir);

        Ok(Self {
            profile,
            package,
            manifest_dir,
            target_dir,
        })
    }

    pub fn site_dir(&self, assembly: &str) -> Utf8PathBuf {
        self.target_dir
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

fn env(key: &str) -> Result<String> {
    std::env::var(key).with_context(|| format!("{key} not found"))
}
