use base64::{engine::general_purpose::URL_SAFE, Engine};
use builder_command::WasmCmd;
use camino::Utf8PathBuf;
use common::{is_release, out, Utf8PathExt};
use fs_err as fs;
use std::hash::Hasher;
use wasm_pack::{
    command::{
        build::{BuildOptions, Target},
        run_wasm_pack, Command,
    },
    install::InstallMode,
};

pub fn run(cmd: &WasmCmd) {
    let tmp_dir = cmd.output_dir.join("tmp");
    let package_relative_tmp_dir =
        Utf8PathBuf::from_iter(cmd.package_dir.components().map(|_| "..")).join(&tmp_dir);

    tmp_dir.create_dir_if_missing().unwrap();

    log::debug!("package_relative_tmp_dir: {package_relative_tmp_dir}");
    let release = is_release();

    let wasm = Command::Build(BuildOptions {
        path: Some(cmd.package_dir.as_std_path().to_path_buf()),
        scope: None,
        mode: InstallMode::Noinstall,
        // no typescript
        disable_dts: true,
        // enable JS weak refs proposal
        weak_refs: false,
        // enable WebAssembly reference types proposal
        reference_types: false,
        target: Target::Web,
        // deprecated
        debug: false,
        dev: !release,
        release,
        profiling: false,
        out_dir: package_relative_tmp_dir.as_str().to_string(),
        out_name: Some(cmd.name.clone()),
        no_pack: true,
        no_opt: !cmd.optimize,
        extra_options: vec![],
    });
    run_wasm_pack(wasm).unwrap();

    let files =
        tmp_dir.ls_files_matching(|p| p.extension() == Some("wasm") || p.extension() == Some("js"));

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
