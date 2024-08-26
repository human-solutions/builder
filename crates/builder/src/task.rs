use std::{
    collections::HashMap,
    fmt::Display,
    fs::{self, File},
    io::{Read, Write},
    str::FromStr,
    sync::Arc,
};

use anyhow::{Context, Result};
use camino::{Utf8Path, Utf8PathBuf};
use fs4::fs_std::FileExt;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use swc::{
    config::{IsModule, JsMinifyOptions},
    try_with_handler, BoolOrDataConfig,
};
use swc_common::{FileName, SourceMap, GLOBALS};
use tempfile::NamedTempFile;
use wasm_bindgen_cli_support::Bindgen;

use crate::{ext::metadata::MetadataExt, generate::Output, util::timehash, CmdArgs};

const POSTBUILD_FILE: &str = "postbuild.yaml";

trait IntoVecString {
    fn into_vec_string(&self, key: &str) -> Vec<String>;
}

impl IntoVecString for &Value {
    fn into_vec_string(&self, key: &str) -> Vec<String> {
        self.get(key)
            .and_then(Value::as_array)
            .map(|t| {
                t.iter()
                    .filter_map(Value::as_str)
                    .map(String::from)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    }
}

#[derive(Serialize, Deserialize)]
struct Task {
    pub tool: Tool,
    pub targets: Vec<String>,
    pub profiles: Vec<String>,
}

impl Task {
    fn from_value(key: &str, value: &Value) -> Result<Self> {
        let tool = Tool::from_str(key).context(format!("Invalid tool '{key}'"))?;

        let targets = value.into_vec_string("target");
        let profiles = value.into_vec_string("profile");

        match tool {
            Tool::FontForge => todo!(),
            Tool::WasmBindgen(_) => {
                let params: WasmParams = serde_json::from_value(value.clone()).context(format!(
                    "Failed to parse wasm-bindgen params for task '{key}'"
                ))?;
                Ok(Task {
                    tool,
                    targets,
                    profiles,
                })
            }
            Tool::Uniffi => todo!(),
        }
    }

    // maybe return a generic struct containing data about the output of the task
    fn run(&self, config: &Config) -> Result<()> {
        let target = &config.args.target;
        let profile = &config.args.profile;
        if (self.targets.is_empty() || self.targets.contains(target))
            && (self.profiles.is_empty() || self.profiles.contains(profile))
        {
            log::info!(
                "Running task for {} with target {} and profile {}",
                self.tool,
                target,
                profile
            );
            match &self.tool {
                Tool::FontForge => todo!(),
                Tool::WasmBindgen(params) => todo!(),
                Tool::Uniffi => todo!(),
            }
        } else {
            log::info!("Skipping task for {}", self.tool);
        }

        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
enum Tool {
    FontForge,
    WasmBindgen(WasmParams),
    Uniffi,
}

impl Display for Tool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Tool::FontForge => write!(f, "font-forge"),
            Tool::WasmBindgen(_) => write!(f, "wasm-bindgen"),
            Tool::Uniffi => write!(f, "uniffi"),
        }
    }
}

impl FromStr for Tool {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "font-forge" => Ok(Self::FontForge),
            "wasm-bindgen" => Ok(Self::WasmBindgen(WasmParams::default())),
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
            for _ in tool_val
                .as_array()
                .context(format!("Invalid tasks for tool '{tool}'"))?
            {
                tasks.push(Task::from_value(tool, tool_val)?);
            }
        }

        Ok(Self(tasks))
    }
    fn run(&self, config: &Config) -> Result<()> {
        for task in &self.0 {
            task.run(config)?;
        }

        Ok(())
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[derive(Serialize, Deserialize)]
struct Config {
    pub args: CmdArgs,
    pub package_dir: Utf8PathBuf,
    pub target_dir: Utf8PathBuf,
}

#[derive(Serialize, Deserialize)]
struct Setup {
    pub name: String,
    pub config: Config,
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

        let config = Config {
            args: args.clone(),
            package_dir: package.manifest_path.clone(),
            target_dir: metadata.target_directory.clone(),
        };

        Ok(Self {
            name: package.name.clone(),
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
            self.prebuild.run(&self.config)?;
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

    pub fn site_dir(&self, assembly: &str) -> Utf8PathBuf {
        self.config
            .target_dir
            .join(&self.name)
            .join(&self.config.args.target)
            .join(assembly)
            .join(&self.config.args.profile)
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
struct WasmParams {
    optimize_wasm: bool,
    minify_js: bool,
    out: Output,
}

impl WasmParams {
    pub fn process(&self, setup: &Setup, assembly: &str) -> Result<()> {
        let hash = timehash();
        let debug = setup.config.args.profile != "release";
        let profile = if setup.config.args.profile == "dev" {
            "debug"
        } else {
            &setup.config.args.profile
        };
        let input = setup
            .config
            .target_dir
            .join("wasm32-unknown-unknown")
            .join(profile)
            .join(&setup.name)
            .with_extension("wasm");

        let mut output = Bindgen::new()
            .input_path(input)
            .browser(true)?
            .debug(debug)
            .keep_debug(debug)
            .out_name(&format!("{hash}{}", setup.name))
            .generate_output()?;

        let site_dir = setup.site_dir(assembly);
        // check out the code for this, that's where much of the stuff done here comes from:
        // output.emit(&site_dir)?;

        let _wasm_hash = {
            let mut wasm = output.wasm_mut().emit_wasm();
            let filename = format!("{}.wasm", setup.name);
            if self.optimize_wasm {
                Self::optimize_wasm(&mut wasm)?;
            }
            self.out.write_file(&wasm, &site_dir, &filename)
        }?;

        let _js_hash = {
            let filename = format!("{}.js", setup.name);
            let js = if self.minify_js {
                Self::minify(output.js().to_string())?
            } else {
                output.js().to_string()
            };
            let contents = js.as_bytes();
            self.out.write_file(contents, &site_dir, &filename)
        }?;

        self.write_snippets(output.snippets());
        self.write_modules(output.local_modules(), &site_dir)?;
        Ok(())
    }

    fn write_snippets(&self, snippets: &HashMap<String, Vec<String>>) {
        // Provide inline JS files
        let mut snippet_list = Vec::new();
        for (identifier, list) in snippets.iter() {
            for (i, _js) in list.iter().enumerate() {
                let name = format!("inline{}.js", i);
                snippet_list.push(format!(
                    "snippet handling not implemented: {identifier} {name}"
                ));
            }
        }
        if !snippet_list.is_empty() {
            panic!(
                "snippet handling not implemented: {}",
                snippet_list.join(", ")
            );
        }
    }

    fn write_modules(&self, modules: &HashMap<String, String>, _site_dir: &Utf8Path) -> Result<()> {
        // Provide snippet files from JS snippets
        for (path, _js) in modules.iter() {
            println!("module: {path}");
            // let site_path = Utf8PathBuf::from("snippets").join(path);
            // let file_path = proj.site.root_relative_pkg_dir().join(&site_path);

            // fs::create_dir_all(file_path.parent().unwrap()).await?;

            // let site_file = SiteFile {
            //     dest: file_path,
            //     site: site_path,
            // };

            // js_changed |= if proj.release && proj.js_minify {
            //     proj.site
            //         .updated_with(&site_file, minify(js)?.as_bytes())
            //         .await?
            // } else {
            //     proj.site.updated_with(&site_file, js.as_bytes()).await?
            // };
        }
        Ok(())
    }

    fn optimize_wasm(wasm: &mut Vec<u8>) -> Result<()> {
        let mut infile = NamedTempFile::new()?;
        infile.write_all(wasm)?;

        let mut outfile = NamedTempFile::new()?;

        wasm_opt::OptimizationOptions::new_optimize_for_size()
            .run(infile.path(), outfile.path())?;

        wasm.clear();
        outfile.read_to_end(wasm)?;
        Ok(())
    }

    fn minify(js: String) -> Result<String> {
        let cm = Arc::<SourceMap>::default();

        let c = swc::Compiler::new(cm.clone());
        let output = GLOBALS.set(&Default::default(), || {
            try_with_handler(cm.clone(), Default::default(), |handler| {
                let fm = cm.new_source_file(Arc::new(FileName::Anon), js);

                c.minify(
                    fm,
                    handler,
                    &JsMinifyOptions {
                        compress: BoolOrDataConfig::from_bool(true),
                        mangle: BoolOrDataConfig::from_bool(true),
                        // keep_classnames: true,
                        // keep_fnames: true,
                        module: IsModule::Bool(true),
                        ..Default::default()
                    },
                )
                .context("failed to minify")
            })
        })?;

        Ok(output.code)
    }
}
