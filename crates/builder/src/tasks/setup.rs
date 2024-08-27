use std::{
    fs::{self, File},
    io::{Read, Write},
};

use anyhow::{Context, Result};
use camino::{Utf8Path, Utf8PathBuf};
use fs4::fs_std::FileExt;
use serde::{Deserialize, Serialize};

use crate::{ext::metadata::MetadataExt, CmdArgs};

use super::{postbuild::PostbuildTasks, prebuild::PrebuildTasks};

const POSTBUILD_FILE: &str = "postbuild.yaml";

#[derive(Debug)]
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

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub package_name: String,
    pub args: CmdArgs,
    pub package_dir: Utf8PathBuf,
    pub target_dir: Utf8PathBuf,
}

impl Config {
    pub fn site_dir(&self, assembly: &str) -> Utf8PathBuf {
        self.target_dir
            .join(&self.package_name)
            .join(&self.args.target)
            .join(assembly)
            .join(&self.args.profile)
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

#[derive(Serialize, Deserialize)]
pub struct Setup {
    pub config: Config,
    pub prebuild: PrebuildTasks,
    pub postbuild: PostbuildTasks,
    pub deps: Vec<String>,
}

impl Setup {
    pub fn new(args: &CmdArgs) -> Result<Self> {
        let metadata = cargo_metadata::MetadataCommand::new()
            .manifest_path(args.dir.join("Cargo.toml"))
            .exec()?;

        let package = metadata.root_package().context("root package not found")?;

        let deps = metadata
            .local_dependency_packages()
            .map(|p| p.name.to_string())
            .collect::<Vec<_>>();

        let mut prebuild = PrebuildTasks::default();
        let mut postbuild = PostbuildTasks::default();

        if let Some(prebuild_val) = package.metadata.get("prebuild") {
            prebuild = PrebuildTasks::from_value(prebuild_val)?;
        }
        if let Some(postbuild_val) = package.metadata.get("postbuild") {
            postbuild = PostbuildTasks::from_value(postbuild_val)?;
        }

        let config = Config {
            package_name: package.name.clone(),
            args: args.clone(),
            package_dir: package.manifest_path.clone(),
            target_dir: metadata.target_directory.clone(),
        };

        Ok(Self {
            config,
            prebuild,
            postbuild,
            deps,
        })
    }

    fn load(path: &Utf8PathBuf) -> Result<Self> {
        let mut file = File::open(path)?;
        file.try_lock_exclusive()?;

        let mut string = String::new();
        file.read_to_string(&mut string)?;

        file.unlock()?;
        fs::remove_file(path)?;

        let setup: Self = serde_yaml::from_str(&string)
            .context(format!("Failed to parse output file '{}'", path))?;

        Ok(setup)
    }

    fn save(&self) -> Result<()> {
        let string =
            serde_yaml::to_string(self).context("Failed to serialize configuration file")?;

        let path = self.postbuild_file(&self.config.package_name);

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut file = File::create(&path)
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
                    Err(_) => {
                        log::warn!("Failed to load setup file '{:?}'", postbuild_file);
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
            self.save()?;
        }

        Ok(())
    }

    fn run_postbuild(&self) -> Result<()> {
        log::info!("Running postbuild for {}", self.config.package_name);
        if self.postbuild.is_empty() {
            log::info!("No postbuild tasks found for {}", self.config.package_name);
        } else {
            log::info!("Running postbuild for {}", self.config.package_name);
            self.postbuild.run(&self.config)?;
        }

        Ok(())
    }

    fn postbuild_files(&self, package_name: &str) -> Result<Vec<Utf8PathBuf>> {
        let target_dir = self.config.target_dir.join(package_name);

        if !target_dir.exists() {
            return Ok(Vec::new());
        }

        let mut files = Vec::new();

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
            .target_dir
            .join(package_name)
            .join(&self.config.args.target)
            .join(POSTBUILD_FILE)
    }
}
