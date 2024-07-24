use ::anyhow::bail;

use crate::ext::anyhow;
use crate::ext::{anyhow::Context, metadata::MetadataExt};
use crate::postbuild::PostbuildConfig;
use crate::prebuild::PrebuildConfig;
use anyhow::Result;
use camino::{Utf8Path, Utf8PathBuf};
use cargo_metadata::{Metadata, Package, PackageId};
use clap::Args;

#[derive(Args, Debug)]
pub struct CmdArgs {
    #[clap(long, env = "CARGO_MANIFEST_DIR")]
    pub dir: Utf8PathBuf,
    #[clap(long, env = "PROFILE")]
    pub profile: String,
    #[clap(long, env = "CARGO_PKG_NAME")]
    pub package: String,
}

#[derive(Debug)]
pub struct PackageConfig {
    pub name: String,
    pub id: PackageId,
    pub dir: Utf8PathBuf,
    pub prebuild: Option<PrebuildConfig>,
    pub postbuild: Option<PostbuildConfig>,
    pub has_build_rs: bool,
}

impl PackageConfig {
    pub fn from_package(package: &Package) -> Result<Self> {
        let mut prebuild = None;
        let mut postbuild = None;
        if let Some(prebuild_val) = package.metadata.get("prebuild") {
            prebuild = Some(PrebuildConfig::from_json(prebuild_val).dot()?);
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
            dir: package.manifest_path.clone(),
            prebuild,
            postbuild,
            has_build_rs,
        })
    }
}

#[derive(Debug)]
pub struct Config {
    pub args: CmdArgs,
    pub metadata: Metadata,
    pub package: PackageConfig,
    pub builder_deps: Vec<PackageConfig>,
}

impl Config {
    pub fn from_path(args: CmdArgs) -> Result<Self> {
        let metadata = cargo_metadata::MetadataCommand::new()
            .manifest_path(&args.dir.join("Cargo.toml"))
            .exec()?;
        let root_pack = metadata.root_package().context("root package not found")?;
        let package = PackageConfig::from_package(root_pack)?;
        let builder_deps = metadata
            .local_dependency_packages()
            .map(PackageConfig::from_package)
            .collect::<Result<_>>()?;
        Ok(Self {
            args,
            package,
            metadata,
            builder_deps,
        })
    }

    pub fn run_prebuild(&self) -> anyhow::Result<()> {
        self.package
            .prebuild
            .as_ref()
            .expect("No prebuild config found")
            .process(self)
    }

    pub fn run_postbuild(&self) -> anyhow::Result<()> {
        self.package
            .postbuild
            .as_ref()
            .expect("No prebuild config found")
            .process(self)
    }

    pub fn site_dir(&self, assembly: &str) -> Utf8PathBuf {
        self.metadata
            .target_directory
            .join(&self.package.name)
            .join(assembly)
            .join(&self.args.profile)
    }

    pub fn existing_manifest_dir_path(&self, path: &Utf8Path) -> Result<Utf8PathBuf> {
        let file = if path.is_relative() {
            self.args.dir.join(path)
        } else {
            bail!("The path {path} must be relative to the manifest directory")
        };

        if !file.exists() {
            bail!("The path {file} doesn't exist");
        }
        Ok(file)
    }
}
