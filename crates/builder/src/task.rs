use std::{
    fmt::Display,
    fs::{self, File},
    io::{Read, Write},
    str::FromStr,
};

use anyhow::{Context, Result};
use camino::Utf8PathBuf;
use fs4::fs_std::FileExt;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{ext::metadata::MetadataExt, CmdArgs};

const POSTBUILD_FILE: &str = "postbuild.yaml";

#[derive(Serialize, Deserialize)]
struct Task {
    pub tool: Tool,
    pub targets: Vec<String>,
    pub profiles: Vec<String>,
}

impl Task {
    fn from_value(key: &str, value: &Value) -> Result<Self> {
        let tool = Tool::from_str(key).context(format!("Invalid tool '{key}'"))?;

        match tool {
            Tool::FontForge => todo!(),
            Tool::WasmBindgen => todo!(),
            Tool::Uniffi => todo!(),
        }
        todo!()
    }

    // maybe return a generic struct containing data about the output of the task
    fn run(&self, target: &String, profile: &String) -> Result<()> {
        if (self.targets.is_empty() || self.targets.contains(target))
            && (self.profiles.is_empty() || self.profiles.contains(profile))
        {
            log::info!(
                "Running task for {} with target {} and profile {}",
                self.tool,
                target,
                profile
            );
            todo!()
        } else {
            log::info!("Skipping task for {}", self.tool);
        }

        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
enum Tool {
    FontForge,
    WasmBindgen,
    Uniffi,
}

impl Display for Tool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Tool::FontForge => write!(f, "font-forge"),
            Tool::WasmBindgen => write!(f, "wasm-bindgen"),
            Tool::Uniffi => write!(f, "uniffi"),
        }
    }
}

impl FromStr for Tool {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "font-forge" => Ok(Self::FontForge),
            "wasm-bindgen" => Ok(Self::WasmBindgen),
            "uniffi" => Ok(Self::Uniffi),
            _ => anyhow::bail!("Invalid tool: {}", s),
        }
    }
}

#[derive(Default, Serialize, Deserialize)]
struct Tasks(Vec<Task>);

impl Tasks {
    fn from_value(value: &Value) -> Result<Self> {
        let mut tasks = Vec::new();

        for (tool, tool_val) in value.as_object().context("Invalid builder metadata")? {
            tasks.push(Task::from_value(tool, tool_val)?);
        }

        Ok(Self(tasks))
    }
    fn run(&self, target: &String, profile: &String) -> Result<()> {
        for task in &self.0 {
            task.run(target, profile)?;
        }

        Ok(())
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[derive(Serialize, Deserialize)]
struct Setup {
    pub name: String,
    pub args: CmdArgs,
    pub dir: Utf8PathBuf,
    pub target_dir: Utf8PathBuf,
    pub prebuild: Tasks,
    pub postbuild: Tasks,
    pub deps: Vec<String>,
}

impl Setup {
    fn new(args: &CmdArgs) -> Result<Self> {
        let metadata = cargo_metadata::MetadataCommand::new()
            .manifest_path(args.dir.join("Cargo.toml"))
            .exec()?;

        let package = metadata.root_package().context("root package not found")?;

        let deps = metadata
            .local_dependency_packages()
            .map(|p| p.name.to_string())
            .collect::<Vec<_>>();

        let mut prebuild = Tasks::default();
        let mut postbuild = Tasks::default();

        if let Some(prebuild_val) = package.metadata.get("prebuild") {
            prebuild = Tasks::from_value(prebuild_val)?;
        }
        if let Some(postbuild_val) = package.metadata.get("postbuild") {
            postbuild = Tasks::from_value(postbuild_val)?;
        }

        Ok(Self {
            name: package.name.clone(),
            args: args.clone(),
            dir: package.manifest_path.clone(),
            target_dir: metadata.target_directory.clone(),
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

        let path = self.postbuild_file(&self.name);

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut file = File::create(&path)
            .context(format!("Failed to create configuration file to '{path}'"))?;
        file.write_all(string.as_bytes())
            .context("Failed to write configuration file")?;

        Ok(())
    }

    fn run(&self) -> Result<()> {
        // gather dependencies postbuild
        // run postbuilds
        // run current prebuilds
        // save current postbuilds
        todo!()
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
            log::info!("No prebuild tasks found for {}", self.name);
        } else {
            log::info!("Running prebuild for {}", self.name);
            self.prebuild.run(&self.args.target, &self.args.profile)?;
        }

        if self.postbuild.is_empty() {
            log::info!("No postbuild tasks found for {}", self.name);
        } else {
            log::info!("Saving postbuild tasks for {}", self.name);
            self.save()?;
        }

        Ok(())
    }

    fn run_postbuild(&self) -> Result<()> {
        log::info!("Running postbuild for {}", self.name);
        if self.postbuild.is_empty() {
            log::info!("No postbuild tasks found for {}", self.name);
        } else {
            log::info!("Running postbuild for {}", self.name);
            self.postbuild.run(&self.args.target, &self.args.profile)?;
        }

        Ok(())
    }

    fn postbuild_files(&self, package_name: &str) -> Result<Vec<Utf8PathBuf>> {
        let target_dir = self.target_dir.join(package_name);

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
        self.target_dir
            .join(package_name)
            .join(&self.args.target)
            .join(POSTBUILD_FILE)
    }
}
