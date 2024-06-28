use crate::anyhow::{bail, Result};
use camino::{Utf8Path, Utf8PathBuf};
use cargo_metadata::{Metadata, PackageId};
use clap::Args;

#[derive(Args, Debug)]
pub struct RawPrebuildArgs {
    #[clap(long, env = "CARGO_MANIFEST_DIR")]
    pub manifest_dir: Utf8PathBuf,
    #[clap(long, env = "PROFILE")]
    pub profile: String,
    #[clap(long, env = "CARGO_PKG_NAME")]
    pub package: String,
}

impl TryInto<PrebuildArgs> for RawPrebuildArgs {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<PrebuildArgs> {
        let metadata = cargo_metadata::MetadataCommand::new()
            .manifest_path(&self.manifest_dir.join("Cargo.toml"))
            .exec()?;
        let package_id = metadata
            .root_package()
            .expect("Expected to have a root package in the cargo metadata")
            .id
            .clone();

        Ok(PrebuildArgs {
            package_id,
            metadata,
            manifest_dir: self.manifest_dir,
            profile: self.profile,
            package: self.package,
        })
    }
}

pub struct PrebuildArgs {
    pub package_id: PackageId,
    pub metadata: Metadata,
    pub manifest_dir: Utf8PathBuf,
    pub profile: String,
    pub package: String,
}

impl PrebuildArgs {
    pub fn site_dir(&self, assembly: &str) -> Utf8PathBuf {
        self.metadata
            .target_directory
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
