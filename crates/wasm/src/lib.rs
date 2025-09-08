use base64::{Engine, engine::general_purpose::URL_SAFE};
use builder_command::WasmCmd;
use camino_fs::*;
use common::{
    is_release,
    site_fs::{SiteFile, write_file_to_site},
};
use std::{fs::File, hash::Hasher};
use wasm_opt::OptimizationOptions;

pub fn run(cmd: &WasmCmd) {
    let release = is_release();
    let package_name = cmd.package.replace("-", "_");

    let tmp_dir = Utf8PathBuf::from("target/wasm_tmp");
    tmp_dir.mkdir().unwrap();

    let wasm_path = Utf8PathBuf::from(format!(
        "target/wasm32-unknown-unknown/{}/{package_name}.wasm",
        if release { "release" } else { "debug" }
    ));
    let wasm_mtime = wasm_path.mtime().unwrap();

    let wasm_mtime_path = wasm_path.with_extension("wasm.mtime");

    if wasm_mtime_path.exists() {
        let prev_mtime = wasm_mtime_path.mtime().unwrap();
        log::debug!("\nprev_mtime: {prev_mtime:?}, \nwasm_mtime: {wasm_mtime:?}");
        if wasm_mtime == prev_mtime {
            log::info!("No change detected, skipping wasmbindgen for {wasm_path}");
            return;
        }
    } else {
        wasm_mtime_path
            .write("this file has the mtime of the last time the wasm was built")
            .unwrap();
    }
    let wasm_mtime_file = File::open(&wasm_mtime_path).unwrap();
    wasm_mtime_file.set_modified(wasm_mtime).unwrap();

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

    let files = tmp_dir
        .ls()
        .filter(|p| p.extension() == Some("wasm") || p.extension() == Some("js"));

    if release {
        let tmp = tmp_dir.with_extension("wasm-opt.wasm");
        let wasm_path = tmp_dir.join(format!("{package_name}_bg.wasm"));
        OptimizationOptions::new_optimize_for_size_aggressively()
            .run(&wasm_path, &tmp)
            .unwrap();
        tmp.mv(wasm_path).unwrap();
    }

    let mut hasher = seahash::SeaHasher::new();
    let file_and_content = files
        .into_iter()
        .map(|p| {
            let p = p.relative_to(&tmp_dir).unwrap().to_path_buf();
            let content = tmp_dir.join(&p).read_bytes().unwrap();
            hasher.write(&content);
            (p, content)
        })
        .collect::<Vec<_>>();

    let hash = URL_SAFE.encode(hasher.finish().to_be_bytes());

    for opts in cmd.output.iter() {
        // TODO: use the hash as the dir name
        opts.dir
            .ls()
            .recurse()
            .filter(|dir| dir.file_name().is_some_and(|n| n.starts_with("wasm")))
            .for_each(|dir| {
                log::debug!("Removing old wasm dir {dir}");
                dir.rm().unwrap();
            });

        let hash_dir = if opts.checksum {
            Utf8PathBuf::from(format!("wasm.{hash}"))
        } else {
            Utf8PathBuf::from("wasm")
        };

        let mut opts = opts.clone();
        // The checksum is in the path of the dir
        opts.checksum = false;
        let opts = [opts];

        for (file, contents) in file_and_content.iter() {
            let site_file = SiteFile::from_file(file).with_dir(&hash_dir);
            write_file_to_site(&site_file, contents, &opts);
        }
    }
    log::debug!("Removing tmp dir {tmp_dir}");
    tmp_dir.rm().unwrap();
}
