use cargo_metadata::Metadata;
use fs_err as fs;
use std::io::{Read, Write};

use anyhow::{Context, Result};
use camino::{Utf8Path, Utf8PathBuf};
use fs4::fs_std::FileExt;
use serde::{Deserialize, Serialize};

use crate::{ext::metadata::MetadataExt, CmdArgs};

use super::{postbuild::PostbuildTasks, prebuild::PrebuildTasks};

const POSTBUILD_FILE: &str = "postbuild.yaml";

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum BuildStep {
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

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub phase: BuildStep,
    pub package_name: String,
    pub library_name: Option<String>,
    pub args: CmdArgs,
    pub target_dir: Utf8PathBuf,
}

impl Config {
    pub fn new(metadata: Metadata, args: &CmdArgs, phase: BuildStep) -> Self {
        let package = metadata.root_package().unwrap();

        Self {
            phase,
            package_name: package.name.clone(),
            library_name: metadata.library_name(),
            args: args.clone(),
            target_dir: metadata.target_directory.clone(),
        }
    }

    /// The dir <target_dir>/<phase>/<package_name>
    pub fn package_target_dir(&self) -> Utf8PathBuf {
        self.target_dir
            .join(self.phase.as_str())
            .join(&self.package_name)
    }

    pub fn site_dir(&self, folder: &str) -> Utf8PathBuf {
        self.package_target_dir()
            .join(&self.args.target)
            .join(&self.args.profile)
            .join(folder)
    }

    pub fn postbuild_package_target_dir(&self, package_name: &str) -> Utf8PathBuf {
        self.target_dir
            .join(BuildStep::Postbuild.as_str())
            .join(package_name)
    }

    pub fn existing_manifest_dir_path(&self, path: &Utf8Path) -> Result<Utf8PathBuf> {
        let file = if path.is_relative() {
            self.args.dir.join(path)
        } else {
            anyhow::bail!("The path {path} must be relative to the manifest directory")
        };

        if !file.exists() {
            anyhow::bail!("The path {file} doesn't exist");
        }
        Ok(file)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Setup {
    pub config: Config,
    pub prebuild: PrebuildTasks,
    pub postbuild: PostbuildTasks,
    pub deps: Vec<String>,
}

impl Setup {
    pub fn new(args: &CmdArgs, phase: BuildStep) -> Result<Self> {
        let cargo_path = args.dir.join("Cargo.toml");

        log::info!("Retrieving metadata from '{cargo_path}'");
        let metadata = cargo_metadata::MetadataCommand::new()
            .manifest_path(&cargo_path)
            .exec()
            .context(format!("Failed to retrieve metadata from {cargo_path}",))?;

        let package = metadata.root_package().context("root package not found")?;

        let deps = metadata
            .local_dependency_packages()
            .map(|p| p.name.to_string())
            .collect::<Vec<_>>();

        let mut prebuild = PrebuildTasks::default();
        let mut postbuild = PostbuildTasks::default();

        if let Some(prebuild_val) = package.metadata.get("prebuild") {
            log::info!("Retrieving prebuild metadata");
            prebuild = PrebuildTasks::from_value(prebuild_val)
                .context("Failed to retrieve prebuild metadata")?;
        }
        if let Some(postbuild_val) = package.metadata.get("postbuild") {
            log::info!("Retrieving postbuild metadata");
            postbuild = PostbuildTasks::from_value(postbuild_val)
                .context("Failed to retrieve postbuild metadata")?;
        }

        let config = Config::new(metadata, args, phase);

        Ok(Self {
            config,
            prebuild,
            postbuild,
            deps,
        })
    }

    fn load(path: &Utf8PathBuf) -> Result<Self> {
        log::info!("Loading setup file '{:?}'", path);

        let mut file = std::fs::File::open(path)?;
        log::info!("Locking file '{:?}'", path);
        file.try_lock_exclusive()?;

        let mut string = String::new();
        file.read_to_string(&mut string)?;

        file.unlock()?;
        log::info!("Unlocking file '{:?}'", path);
        fs::remove_file(path)?;

        let setup: Self =
            serde_yaml::from_str(&string).context(format!("Failed to parse file '{}'", path))?;

        Ok(setup)
    }

    fn save(&self) -> Result<()> {
        let string =
            serde_yaml::to_string(self).context("Failed to serialize configuration file")?;

        let path = self.postbuild_file(&self.config.package_name);

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).context(format!("Failed to create path '{parent}'"))?;
        }

        let mut file = fs::File::create(&path)
            .context(format!("Failed to create configuration file to '{path}'"))?;
        file.write_all(string.as_bytes())
            .context("Failed to write configuration file")?;

        Ok(())
    }

    pub fn run(&self, step: BuildStep) -> Result<()> {
        match step {
            BuildStep::Prebuild => self.run_prebuild(),
            BuildStep::Postbuild => self.run_postbuild(),
        }
    }

    fn run_prebuild(&self) -> Result<()> {
        for package in &self.deps {
            for postbuild_file in self.postbuild_files(package)? {
                let setup = match Self::load(&postbuild_file) {
                    Ok(s) => s,
                    Err(e) => {
                        log::warn!("Failed to load setup file '{:?}': {e}", postbuild_file);
                        continue;
                    }
                };

                setup.run_postbuild()?;
            }
        }

        if self.prebuild.is_empty() {
            log::info!("No prebuild tasks found for {}", self.config.package_name);
        } else {
            log::info!("Running prebuild for {}", self.config.package_name);
            self.prebuild.run(&self.config)?;
        }

        if self.postbuild.is_empty() {
            log::info!("No postbuild tasks found for {}", self.config.package_name);
        } else {
            log::info!("Saving postbuild tasks for {}", self.config.package_name);
            let mut conf = self.clone();
            conf.config.phase = BuildStep::Postbuild;
            conf.save()
                .context("Failed to save postbuild configuration file")?;
        }

        Ok(())
    }

    fn run_postbuild(&self) -> Result<()> {
        if self.postbuild.is_empty() {
            log::info!("No postbuild tasks found for {}", self.config.package_name);
        } else {
            log::info!("Running postbuild for {}", self.config.package_name);
            self.postbuild.run(&self.config)?;
        }

        Ok(())
    }

    fn postbuild_files(&self, package_name: &str) -> Result<Vec<Utf8PathBuf>> {
        let target_dir = self.config.postbuild_package_target_dir(package_name);

        if !target_dir.exists() {
            return Ok(Vec::new());
        }

        let mut files = Vec::new();

        log::info!("Searching for postbuild files in '{target_dir}'");

        for entry in fs::read_dir(target_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                let file = path.join(POSTBUILD_FILE);
                if file.exists() {
                    files.push(Utf8PathBuf::from_path_buf(file).map_err(|e| {
                        anyhow::Error::msg(format!("Failed to create postbuild file path :{:?}", e))
                    })?);
                }
            }
        }

        Ok(files)
    }

    fn postbuild_file(&self, package_name: &str) -> Utf8PathBuf {
        self.config
            .postbuild_package_target_dir(package_name)
            .join(&self.config.args.target)
            .join(POSTBUILD_FILE)
    }
}
