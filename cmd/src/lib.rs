mod ext;
mod generate;
mod postbuild;
mod prebuild;
mod util;

use anyhow::Result;
use camino::Utf8PathBuf;
use cargo_metadata::{Metadata, Package, PackageId};
use clap::Args;
pub use ext::anyhow;
use ext::{anyhow::Context, metadata::MetadataExt};
pub use postbuild::{PostbuildArgs, PostbuildConfig};
pub use prebuild::{PrebuildArgs, PrebuildConfig, RawPrebuildArgs};

#[derive(Args, Debug)]
pub struct RawManifest {
    #[clap(long, env = "CARGO_MANIFEST_DIR")]
    pub manifest_dir: Utf8PathBuf,
}

impl RawManifest {
    pub fn try_into(self) -> anyhow::Result<Manifest> {
        let metadata = cargo_metadata::MetadataCommand::new()
            .manifest_path(&self.manifest_dir.join("Cargo.toml"))
            .exec()?;
        Manifest::from_metadata(metadata)
    }
}

#[derive(Debug)]
pub struct PackageConfig {
    pub name: String,
    pub id: PackageId,
    pub prebuild: Option<PrebuildConfig>,
    pub postbuild: Option<PostbuildConfig>,
    pub has_build_rs: bool,
}

impl PackageConfig {
    pub fn from_package(package: &Package) -> Result<Self> {
        let mut prebuild = None;
        let mut postbuild = None;
        if let Some(prebuild_val) = package.metadata.get("prebuild") {
            prebuild = Some(PrebuildConfig::from_json(prebuild_val)?);
        }
        if let Some(postbuild_val) = package.metadata.get("postbuild") {
            postbuild = Some(PostbuildConfig::from_json(postbuild_val)?);
        }

        let has_build_rs = package
            .targets
            .iter()
            .any(|target| target.name == "build-script-build");
        Ok(Self {
            name: package.name.clone(),
            id: package.id.clone(),
            prebuild,
            postbuild,
            has_build_rs,
        })
    }
}

#[derive(Debug)]
pub struct Manifest {
    pub manifest_dir: Utf8PathBuf,
    pub metadata: Metadata,
    pub package: PackageConfig,
    pub builder_deps: Vec<PackageConfig>,
}

impl Manifest {
    pub fn from_metadata(metadata: Metadata) -> Result<Self> {
        let manifest_dir = metadata.workspace_root.clone();
        let root_pack = metadata.root_package().context("root package not found")?;
        let package = PackageConfig::from_package(root_pack)?;
        let builder_deps = metadata
            .local_dependency_packages()
            .map(PackageConfig::from_package)
            .collect::<Result<_>>()?;
        Ok(Self {
            package,
            manifest_dir,
            metadata,
            builder_deps,
        })
    }

    pub fn process(&self) -> anyhow::Result<()> {
        println!("manifest: {:#?}", self.package);
        Ok(())
    }
}
