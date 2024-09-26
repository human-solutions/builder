use std::{
    collections::HashMap,
    io::{Read, Write},
    sync::Arc,
};

use anyhow::{Context, Result};
use camino::Utf8Path;
use serde::{Deserialize, Serialize};
use swc::{
    config::{IsModule, JsMinifyOptions},
    try_with_handler, BoolOrDataConfig, JsMinifyExtras,
};
use swc_common::{FileName, SourceMap, GLOBALS};
use tempfile::NamedTempFile;
use wasm_bindgen_cli_support::Bindgen;

use crate::{generate::Output, util::timehash};

use super::{setup::Config, BuildStep};

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub(super) struct WasmParams {
    optimize_wasm: bool,
    minify_js: bool,
    out: Output,
}

impl WasmParams {
    pub fn process(&self, config: &Config, phase: &BuildStep) -> Result<()> {
        let hash = timehash();
        let debug = config.args.profile != "release";
        let profile = if config.args.profile == "dev" {
            "debug"
        } else {
            &config.args.profile
        };
        let input = config
            .target_dir
            .join("wasm32-unknown-unknown")
            .join(profile)
            .join(&config.package_name)
            .with_extension("wasm");

        log::info!("Processing wasm: {input}");

        let mut output = Bindgen::new()
            .input_path(input)
            .browser(true)?
            .debug(debug)
            .keep_debug(debug)
            .out_name(&format!("{hash}{}", config.package_name))
            .generate_output()?;

        let site_dir = config.site_dir("wasm-bindgen", phase);
        // check out the code for this, that's where much of the stuff done here comes from:
        // output.emit(&site_dir)?;

        let _wasm_hash = {
            let mut wasm = output.wasm_mut().emit_wasm();
            let filename = format!("{}.wasm", config.package_name);
            if self.optimize_wasm {
                Self::optimize_wasm(&mut wasm)?;
            }
            self.out.write_file(&wasm, &site_dir, &filename)
        }?;

        let _js_hash = {
            let filename = format!("{}.js", config.package_name);
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
        log::info!("Writing wasm snippets");

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
        log::info!("Writing wasm modules");

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
        log::info!("Optimizing wasm");

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
        log::info!("Minifying js");

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
                    JsMinifyExtras::default(),
                )
                .context("failed to minify")
            })
        })?;

        Ok(output.code)
    }
}
