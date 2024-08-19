use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::Arc;

use crate::anyhow::{Context, Result};
use crate::generate::Output;
use crate::util::{run_cmd, timehash};
use crate::Config;
use camino::Utf8Path;
use serde::{Deserialize, Serialize};
use swc::config::{IsModule, JsMinifyOptions};
use swc::{try_with_handler, BoolOrDataConfig};
use swc_common::{FileName, SourceMap, GLOBALS};
use tempfile::NamedTempFile;
use wasm_bindgen_cli_support::Bindgen;

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct WasmBindgen {
    optimize_wasm: bool,
    minify_js: bool,
    out: Output,
}

impl WasmBindgen {
    pub fn process(&self, info: &Config, assembly: &str, profile: &str) -> Result<()> {
        if assembly == "web" {
            let hash = timehash();
            let debug = info.args.profile != "release";
            let profile = if info.args.profile == "dev" {
                "debug"
            } else {
                &info.args.profile
            };
            let input = info
                .target_dir
                .join("wasm32-unknown-unknown")
                .join(profile)
                .join(&info.package.name)
                .with_extension("wasm");

            let mut output = Bindgen::new()
                .input_path(input)
                .browser(true)?
                .debug(debug)
                .keep_debug(debug)
                .out_name(&format!("{hash}{}", info.package.name))
                .generate_output()?;

            let site_dir = info.site_dir(assembly);
            // check out the code for this, that's where much of the stuff done here comes from:
            // output.emit(&site_dir)?;

            let _wasm_hash = {
                let mut wasm = output.wasm_mut().emit_wasm();
                let filename = format!("{}.wasm", info.package.name);
                if self.optimize_wasm {
                    Self::optimize_wasm(&mut wasm)?;
                }
                self.out.write_file(&wasm, &site_dir, &filename)
            }?;

            let _js_hash = {
                let filename = format!("{}.js", info.package.name);
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
        } else if assembly == "android" {
            // build lib
            let profile = if profile == "release" {
                "lib-release"
            } else {
                "debug"
            };
            let lib_cmds = vec![
                "cargo",
                "build",
                "--profile",
                profile,
                "-p",
                assembly,
                "--color",
                "always",
            ];
            run_cmd(&lib_cmds).context("Failed to create android library")?;

            // build binaries
            let manifest_path = info.args.dir.join("Cargo.toml").to_string();
            let mut bin_cmds = vec![
                "cargo",
                "ndk",
                "--manifest-path",
                &manifest_path,
                "-o",
                "",
                "build",
            ];
            if profile == "release" {
                bin_cmds.push("--release");
            }
            run_cmd(&bin_cmds).context("Failed to create android binaries")?;

            // generate bindings
            let is_mac = cfg!(target_os = "macos");
            let ext = if is_mac { "dylib" } else { "so" };
            let lib_path = {
                let mut p = info.target_dir.join(profile).join(&info.package.name);
                p.set_extension(ext);
                p.to_string()
            };
            let out_dir = {
                let p = info.site_dir(assembly).join(
                    self.out
                        .folder
                        .as_deref()
                        .map(|f| f.to_string())
                        .unwrap_or_default(),
                );

                p.to_string()
            };
            let bind_cmds = vec![
                "cargo",
                "run",
                "--release",
                "--bin=unifi-bindgen",
                "--color=always",
                "--",
                "generate",
                "--library",
                &lib_path,
                "--out-dir",
                &out_dir,
                "--language=kotlin",
            ];
            run_cmd(&bind_cmds).context("Failed to generate android bindings")?;
        }
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
