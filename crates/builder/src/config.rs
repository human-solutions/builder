use std::fs::File;
use std::io::{Read, Write};

use fs4::fs_std::FileExt;
use fs_err as fs;
use log::LevelFilter;
use serde::{Deserialize, Serialize};

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
use cargo_metadata::{Package, PackageId};
use clap::{Args, Subcommand};

#[derive(Args, Debug, Clone, Serialize, Deserialize)]
pub struct CmdArgs {
    #[clap(long, env = "CARGO_MANIFEST_DIR")]
    pub dir: Utf8PathBuf,
    #[clap(long, env = "PROFILE")]
    pub profile: String,
    #[clap(long, env = "CARGO_PKG_NAME")]
    pub package: String,
    #[clap(long, env = "TARGET")]
    pub target: String,
}

#[derive(Debug, Serialize, Deserialize)]
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

        let log_path = conf.target_dir.join(&conf.package.name);
        fs::create_dir_all(&log_path)?;

        let log_file = log_path.join(format!("{}-{}.log", step.as_str(), args.profile));
        setup_logging(log_file.as_str(), LevelFilter::Debug);
        log::info!("Args: {args:?}");

        conf.run(step)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub args: CmdArgs,
    pub target_dir: Utf8PathBuf,
    pub package: PackageConfig,
    pub deps: Vec<String>,
}

impl Config {
    pub fn new(args: &CmdArgs) -> Result<Self> {
        let metadata = cargo_metadata::MetadataCommand::new()
            .manifest_path(args.dir.join("Cargo.toml"))
            .exec()?;

        let root_pack = metadata.root_package().context("root package not found")?;
        let package = PackageConfig::from_package(root_pack)?;
        let deps = metadata
            .local_dependency_packages()
            .map(|p| p.name.to_string())
            .collect::<Vec<_>>();

        let target_dir = metadata.target_directory.clone();
        Ok(Self {
            args: args.clone(),
            package,
            target_dir,
            deps,
        })
    }

    fn run(&self, step: BuildStep) -> Result<()> {
        match step {
            BuildStep::Prebuild => self.run_prebuild(),
            BuildStep::Postbuild => self.run_postbuild(),
        }
    }

    pub fn run_prebuild(&self) -> Result<()> {
        for package in &self.deps {
            let path = self.postbuild_file(package);

            if !path.exists() {
                continue;
            }

            let conf = match Self::load(&path) {
                Ok(p) => p,
                Err(e) => {
                    log::error!("Failed to load postbuild config from {path}: {e}");
                    continue;
                }
            };

            conf.run_postbuild()?;
        }

        log::info!("Running prebuild for {}", self.package.name);

        if let Some(prebuild) = &self.package.prebuild {
            prebuild.process(self)?;
        } else {
            log::info!("No prebuild configuration found for {}", self.package.name);
        }

        // save the config only if the postbuild step is present
        // to make it easier to skip in the next package prebuild
        if self.package.postbuild.is_some() {
            log::info!(
                "Saving postbuild configuration file for {}",
                self.package.name
            );
            self.save()
                .context("Failed to save postbuild configuration file")?;
        }

        Ok(())
    }

    fn postbuild_file(&self, package_name: &str) -> Utf8PathBuf {
        self.target_dir.join(package_name).join("postbuild.yaml")
    }

    fn save(&self) -> Result<()> {
        let string =
            serde_yaml::to_string(self).context("Failed to serialize configuration file")?;

        let path = self.postbuild_file(&self.package.name);

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut file = File::create(&path)
            .context(format!("Failed to create configuration file to '{path}'"))?;
        file.write_all(string.as_bytes())
            .context("Failed to write configuration file")?;

        Ok(())
    }

    fn load(path: &Utf8PathBuf) -> Result<Self> {
        let mut file = File::open(path)?;
        file.try_lock_exclusive()?;

        let mut string = String::new();
        file.read_to_string(&mut string)?;

        file.unlock()?;
        fs::remove_file(path)?;

        let conf: Self = serde_yaml::from_str(&string)
            .context(format!("Failed to parse output file '{}'", path))?;

        Ok(conf)
    }

    pub fn run_postbuild(&self) -> Result<()> {
        log::info!("Running postbuild for {}", self.package.name);
        self.package
            .postbuild
            .as_ref()
            .expect("No prebuild config found")
            .process(self)
    }

    pub fn site_dir(&self, assembly: &str) -> Utf8PathBuf {
        self.target_dir
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
