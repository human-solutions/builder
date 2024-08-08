mod common;

use camino::Utf8PathBuf;
use common::{cargo, PathExt};
use fs_err as fs;

#[test]
fn test_playground() {
    let dir = Utf8PathBuf::from("../../examples/playground");
    let gen = dir.join("assets").join("gen");

    cargo(&dir, ["clean"]);
    cargo(
        &dir,
        ["build", "-p=wasm", "--target=wasm32-unknown-unknown"],
    );

    if gen.exists() {
        fs::remove_dir_all(&gen).unwrap();
    }

    cargo(&dir, ["build"]);

    insta::assert_snapshot!(gen.ls_ascii(0).unwrap(), @r###"
    gen:
      mobile.rs
      web.rs
    "###);

    cargo(&dir, ["build", "--release"]);

    let out = dir.join("target").join("assets");

    insta::assert_snapshot!(out.ls_ascii(0).unwrap(), @r###"
    assets:
      prebuild-debug.log
      prebuild-release.log
      mobile:
        debug:
          main.scss
          static:
            hfT-f2u761M=polyglot.woff2
        release:
          main.scss.br
          static:
            hfT-f2u761M=polyglot.woff2.br
            hfT-f2u761M=polyglot.woff2.gz
      web:
        debug:
          static:
            Ls-GLuljxGw=main.scss
            hfT-f2u761M=polyglot.woff2
            badge:
              static:
                badge:
                  MJjU0sjYbCw=apple_store.svg.en
                  MJjU0sjYbCw=apple_store.svg.fr
                  MJjU0sjYbCw=apple_store.svg.fr-CA
        release:
          static:
            4xved-FTXA0=main.scss.br
            4xved-FTXA0=main.scss.gz
            hfT-f2u761M=polyglot.woff2.br
            hfT-f2u761M=polyglot.woff2.gz
            badge:
              static:
                badge:
                  MJjU0sjYbCw=apple_store.svg.en.br
                  MJjU0sjYbCw=apple_store.svg.en.gz
                  MJjU0sjYbCw=apple_store.svg.fr-CA.br
                  MJjU0sjYbCw=apple_store.svg.fr-CA.gz
                  MJjU0sjYbCw=apple_store.svg.fr.br
                  MJjU0sjYbCw=apple_store.svg.fr.gz
    "###);

    let out_wasm = dir.join("target").join("wasm");

    insta::assert_snapshot!(out_wasm.ls_no_checksum().unwrap(), @r###"
/wasm/prebuild-debug.log
/wasm/prebuild-release.log
/wasm/web/debug/static/wasm.js
/wasm/web/debug/static/wasm.js.br
/wasm/web/debug/static/wasm.js.gz
/wasm/web/debug/static/wasm.wasm
/wasm/web/debug/static/wasm.wasm
/wasm/web/debug/static/wasm.wasm.br
/wasm/web/debug/static/wasm.wasm.gz
"###)
}
