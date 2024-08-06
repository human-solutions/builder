use fs_err as fs;
use log::LevelFilter;

use crate::postbuild::PostbuildConfig;
use crate::prebuild::PrebuildConfig;
use crate::{
    ext::{
        anyhow::{bail, Context, Result},
        metadata::MetadataExt,
    },
    setup_logging,
};
use camino::{Utf8Path, Utf8PathBuf};
use cargo_metadata::{Metadata, Package, PackageId};
use clap::{Args, Subcommand};

#[derive(Args, Debug, Clone)]
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

    pub fn save_postbuild(&self) -> Result<()> {
        todo!()
    }

    pub fn load_postbuild(&self) -> Result<()> {
        todo!()
    }
}

#[derive(Debug)]
enum BuildStep {
    Prebuild,
    Postbuild,
}

impl BuildStep {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Prebuild => "prebuild",
            Self::Postbuild => "postbuild",
        }
    }
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Prebuild(CmdArgs),
    Postbuild(CmdArgs),
}

impl Commands {
    pub fn run(&self) -> Result<()> {
        let (args, step) = match self {
            Self::Prebuild(args) => (args, BuildStep::Prebuild),
            Self::Postbuild(args) => (args, BuildStep::Postbuild),
        };

        let conf = Config::new(args)?;

        let log_path = conf.metadata.target_directory.join(&conf.package.name);
        fs::create_dir_all(&log_path)?;

        let log_file = log_path.join(format!("{}-{}.log", step.as_str(), args.profile));
        setup_logging(log_file.as_str(), LevelFilter::Debug);
        log::info!("Args: {args:?}");

        conf.run(step)
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
    pub fn new(args: &CmdArgs) -> Result<Self> {
        let metadata = cargo_metadata::MetadataCommand::new()
            .manifest_path(args.dir.join("Cargo.toml"))
            .exec()?;

        let root_pack = metadata.root_package().context("root package not found")?;
        let package = PackageConfig::from_package(root_pack)?;
        let builder_deps = metadata
            .local_dependency_packages()
            .map(PackageConfig::from_package)
            .collect::<Result<_>>()?;
        Ok(Self {
            args: args.clone(),
            package,
            metadata,
            builder_deps,
        })
    }

    fn run(&self, step: BuildStep) -> Result<()> {
        log::info!("Running {} step", step.as_str());
        match step {
            BuildStep::Prebuild => self.run_prebuild(),
            BuildStep::Postbuild => self.run_postbuild(),
        }
    }

    pub fn run_prebuild(&self) -> Result<()> {
        for package in self.metadata.local_dependency_names() {
            let path = self.postbuild_file(package);

            if !path.exists() {
                continue;
            }

            let postbuild = match PostbuildConfig::load(&path) {
                Ok(p) => p,
                Err(e) => {
                    log::error!("Failed to load postbuild config from {path}: {e}");
                    continue;
                }
            };

            postbuild.process(self)?;
        }

        self.package
            .prebuild
            .as_ref()
            .expect("No prebuild config found")
            .process(self)?;

        if let Some(postbuild) = &self.package.postbuild {
            postbuild.save(&self.postbuild_file(&self.package.name))?;
        }

        Ok(())
    }

    fn postbuild_file(&self, package_name: &str) -> Utf8PathBuf {
        self.metadata
            .target_directory
            .join("builder")
            .join(package_name)
            .join("postbuild.yaml")
    }

    pub fn run_postbuild(&self) -> Result<()> {
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
