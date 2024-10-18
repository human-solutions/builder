use base64::{engine::general_purpose::URL_SAFE, Engine};
use builder_command::WasmCmd;
use camino::Utf8PathBuf;
use common::{is_release, out, Utf8PathExt};
use fs_err as fs;
use std::{hash::Hasher, process::Command};
use wasm_opt::OptimizationOptions;

// "cargo" "build" "--lib" "--target" "wasm32-unknown-unknown"
// wasm-bindgen target/wasm32-unknown-unknown/debug/app_web.wasm --out-dir target/wasm/tmp --no-typescript --target web --out-name app --debug
pub fn run(cmd: &WasmCmd) {
    let release = is_release();
    let mut cargo = Command::new("cargo");

    let package_name = cmd.package.replace("-", "_");
    cargo
        .arg("build")
        .arg("--package")
        .arg(&cmd.package)
        .arg("--lib")
        .arg("--target")
        .arg("wasm32-unknown-unknown");

    if release {
        cargo.arg("--release");
    }

    let cargo_status = cargo.status().unwrap();
    if !cargo_status.success() {
        panic!("cargo build failed");
    }

    let tmp_dir = Utf8PathBuf::from("target/wasm_tmp");
    tmp_dir.create_dir_if_missing().unwrap();

    let wasm_path = Utf8PathBuf::from(format!(
        "target/wasm32-unknown-unknown/{}/{package_name}.wasm",
        if release { "release" } else { "debug" },
    ));
    let wasm_bytes = fs::read(&wasm_path).unwrap();
    let hash = format!("{:x}", seahash::hash(&wasm_bytes));
    let wasm_sum_path = wasm_path.with_extension("wasm.sum");

    if wasm_sum_path.exists() {
        let current_hash = fs::read_to_string(&wasm_sum_path).unwrap();
        if hash == current_hash {
            log::debug!("No change detected, skipping {wasm_path}");
            return;
        }
    }
    fs::write(wasm_sum_path, hash.as_bytes()).unwrap();

    wasm_bindgen_cli_support::Bindgen::new()
        .input_path(wasm_path)
        .typescript(false)
        .omit_default_module_path(false)
        .web(true)
        .unwrap()
        .out_name(&package_name)
        .debug(true)
        .generate(&tmp_dir)
        .unwrap();

    let files =
        tmp_dir.ls_files_matching(|p| p.extension() == Some("wasm") || p.extension() == Some("js"));

    if release {
        let tmp = tmp_dir.with_extension("wasm-opt.wasm");
        let wasm_path = tmp_dir.join(format!("{package_name}_bg.wasm"));
        OptimizationOptions::new_optimize_for_size_aggressively()
            .run(&wasm_path, &tmp)
            .unwrap();
        fs::rename(&tmp, &wasm_path).unwrap();
    }
    let mut hasher = seahash::SeaHasher::new();
    let file_and_content = files
        .into_iter()
        .map(|p| {
            let p = p.relative_to(&tmp_dir).unwrap();
            let content = fs::read(tmp_dir.join(&p)).unwrap();
            hasher.write(&content);
            (p, content)
        })
        .collect::<Vec<_>>();

    let hash = URL_SAFE.encode(hasher.finish().to_be_bytes());

    for opts in cmd.output.iter() {
        let hash_dir = if opts.checksum {
            Utf8PathBuf::from(format!("{hash}wasm"))
        } else {
            Utf8PathBuf::from("wasm")
        };

        let mut opts = opts.clone();
        // The checksum is in the path of the dir
        opts.checksum = false;
        let opts = [opts];

        for (file, contents) in file_and_content.iter() {
            log::debug!("Join file {file} with dir {hash_dir}");
            let path = hash_dir.join(&file);

            out::write(opts.iter(), &contents, &path);
        }
    }
    log::debug!("Removing tmp dir {tmp_dir}");
    fs::remove_dir_all(&tmp_dir).unwrap();
}
