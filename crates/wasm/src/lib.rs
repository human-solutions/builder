mod dwarf;

use anyhow::Context;
use base64::{Engine, engine::general_purpose::URL_SAFE};
use builder_command::{DebugSymbolsMode, WasmProcessingCmd};
use camino_fs::*;
use common::site_fs::{SiteFile, write_file_to_site};
use common::{Timer, log_command, log_operation, log_trace};
use std::hash::Hasher;
use wasm_opt::OptimizationOptions;

use crate::dwarf::split_debug_symbols;

pub fn run(cmd: &mut WasmProcessingCmd) {
    let _timer = Timer::new("WASM processing");
    let release = matches!(cmd.profile, builder_command::Profile::Release);
    let package_name = cmd.package.replace("-", "_");

    log_command!(
        "WASM",
        "Processing package '{}' ({} mode)",
        package_name,
        if release { "release" } else { "debug" }
    );
    log_operation!("WASM", "Debug symbols mode: {:?}", cmd.debug_symbols);

    let tmp_dir = Utf8PathBuf::from("target/wasm_tmp");
    log_trace!("WASM", "Creating temp directory: {}", tmp_dir);
    tmp_dir.mkdir().unwrap();

    let wasm_path = Utf8PathBuf::from(format!(
        "target/wasm32-unknown-unknown/{}/{package_name}.wasm",
        cmd.profile.as_target_folder()
    ));
    log_operation!("WASM", "Source WASM path: {}", wasm_path);

    let wasm_mtime = wasm_path
        .mtime()
        .with_context(|| format!("Failed to get mtime for {}", wasm_path))
        .unwrap();

    let wasm_mtime_path = wasm_path.with_extension("wasm.mtime");

    if wasm_mtime_path.exists() {
        let prev_mtime = wasm_mtime_path.mtime().unwrap();
        log_trace!(
            "WASM",
            "Previous mtime: {:?}, current mtime: {:?}",
            prev_mtime,
            wasm_mtime
        );
        if wasm_mtime == prev_mtime {
            log_command!("WASM", "No changes detected, skipping processing");
            return;
        }
        log_operation!("WASM", "WASM file changed, proceeding with processing");
    } else {
        log_operation!(
            "WASM",
            "First time processing, creating mtime tracking file"
        );
        wasm_mtime_path
            .write("this file has the mtime of the last time the wasm was built")
            .unwrap();
    }
    // we use the std::fs as fs_err doesn't support setting mtime
    let wasm_mtime_file = std::fs::File::open(&wasm_mtime_path).unwrap();
    wasm_mtime_file.set_modified(wasm_mtime).unwrap();

    let keep_debug = !matches!(cmd.debug_symbols, DebugSymbolsMode::Strip);
    log_operation!("WASM", "Keep debug symbols: {}", keep_debug);

    log_operation!("WASM", "Generating bindings (typescript=false, web=true)");
    wasm_bindgen_cli_support::Bindgen::new()
        .input_path(&wasm_path)
        .typescript(false)
        .omit_default_module_path(false)
        .web(true)
        .unwrap()
        .out_name(&package_name)
        // Include otherwise-extraneous debug checks in output
        .debug(!release)
        // Keep debug sections in Wasm files
        .keep_debug(keep_debug)
        .generate(&tmp_dir)
        .unwrap();

    let files = tmp_dir
        .ls()
        .filter(|p| p.extension() == Some("wasm") || p.extension() == Some("js"));

    let wasm_file_path = tmp_dir.join(format!("{package_name}_bg.wasm"));

    if release {
        log_operation!(
            "WASM",
            "Running wasm-opt (size optimization, debug_info={})",
            keep_debug
        );
        let tmp = tmp_dir.with_extension("wasm-opt.wasm");
        let original_size = wasm_file_path.metadata().unwrap().len();

        OptimizationOptions::new_optimize_for_size_aggressively()
            .debug_info(keep_debug)
            .run(&wasm_file_path, &tmp)
            .unwrap();

        let optimized_size = tmp.metadata().unwrap().len();
        tmp.mv(&wasm_file_path).unwrap();

        let savings =
            ((original_size - optimized_size) as f64 / original_size as f64 * 100.0) as i32;
        log_operation!(
            "WASM",
            "Optimization complete: {} -> {} bytes ({}% reduction)",
            original_size,
            optimized_size,
            savings
        );
    }

    // Handle debug symbols based on mode
    match &cmd.debug_symbols {
        DebugSymbolsMode::Strip => {
            log_operation!("WASM", "Debug symbols stripped");
        }
        DebugSymbolsMode::Keep => {
            log_operation!("WASM", "Debug symbols kept in main WASM file");
        }
        DebugSymbolsMode::WriteTo(debug_path) => {
            log_operation!("WASM", "Splitting debug symbols to: {}", debug_path);
            split_debug_symbols(&wasm_file_path, debug_path).unwrap();
        }
        DebugSymbolsMode::WriteAdjacent => {
            let name = wasm_file_path.file_stem().unwrap();
            let debug_path = wasm_file_path.with_file_name(format!("{name}_debug.wasm"));
            log_operation!(
                "WASM",
                "Splitting debug symbols to adjacent file: {}",
                debug_path
            );
            split_debug_symbols(&wasm_file_path, &debug_path).unwrap();
        }
    }

    log_operation!("WASM", "Computing checksums and collecting files");
    let mut hasher = seahash::SeaHasher::new();
    let file_and_content = files
        .into_iter()
        .map(|p| {
            let p = p.relative_to(&tmp_dir).unwrap().to_path_buf();
            let content = tmp_dir.join(&p).read_bytes().unwrap();
            hasher.write(&content);
            log_trace!("WASM", "Processed file: {} ({} bytes)", p, content.len());
            (p, content)
        })
        .collect::<Vec<_>>();

    let hash = URL_SAFE.encode(hasher.finish().to_be_bytes());
    let total_size: usize = file_and_content
        .iter()
        .map(|(_, content)| content.len())
        .sum();
    log_operation!(
        "WASM",
        "Computed checksums ({} files, {} bytes total, hash={})",
        file_and_content.len(),
        total_size,
        hash
    );

    for opts in cmd.output.iter() {
        log_operation!("WASM", "Writing output to: {}", opts.dir);

        // Clean up old wasm directories
        let old_dirs: Vec<_> = opts
            .dir
            .ls()
            .recurse()
            .filter(|dir| dir.file_name().is_some_and(|n| n.starts_with("wasm")))
            .collect();

        for dir in &old_dirs {
            log_trace!("WASM", "Removing old wasm dir: {}", dir);
            dir.rm().unwrap();
        }

        if !old_dirs.is_empty() {
            log_operation!("WASM", "Cleaned up {} old wasm directories", old_dirs.len());
        }

        let hash_dir = if opts.checksum {
            Utf8PathBuf::from(format!("wasm.{hash}"))
        } else {
            Utf8PathBuf::from("wasm")
        };

        let mut opts = opts.clone();
        // The checksum is in the path of the dir
        opts.checksum = false;
        let mut opts = [opts];

        log_operation!(
            "WASM",
            "Writing {} files to {}",
            file_and_content.len(),
            hash_dir
        );
        for (file, contents) in file_and_content.iter() {
            let site_file = SiteFile::from_file(file).with_dir(&hash_dir);
            log_trace!("WASM", "Writing file: {} -> {}", file, site_file);
            write_file_to_site(&site_file, contents, &mut opts);
        }
    }
    log_trace!("WASM", "Removing tmp dir: {}", tmp_dir);
    tmp_dir.rm().unwrap();
}
