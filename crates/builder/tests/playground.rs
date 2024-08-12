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
        ["build", "-p=client", "--target=wasm32-unknown-unknown"],
    );

    if gen.exists() {
        fs::remove_dir_all(&gen).unwrap();
    }

    cargo(&dir, ["build"]);

    insta::assert_snapshot!(gen.ls_ascii_replace_checksum(0, &[], "").unwrap(), @r###"
    gen:
      mobile.rs
      web.rs
    "###);

    cargo(&dir, ["build", "--release"]);

    let out = dir.join("target").join("assets");

    insta::assert_snapshot!(out.ls_ascii_replace_checksum(0, &["main.css", "polyglot.woff2"], "<checksum>").unwrap(), @r###"
    assets:
      prebuild-debug.log
      prebuild-release.log
      aarch64-apple-darwin:
        mobile:
          debug:
            main.css
            static:
              <checksum>polyglot.woff2
          release:
            main.css.br
            static:
              hfT-f2u761M=polyglot.woff2.br
              hfT-f2u761M=polyglot.woff2.gz
        web:
          debug:
            static:
              <checksum>main.css
              <checksum>polyglot.woff2
              badge:
                static:
                  badge:
                    MJjU0sjYbCw=apple_store.svg.en
                    MJjU0sjYbCw=apple_store.svg.fr
                    MJjU0sjYbCw=apple_store.svg.fr-CA
          release:
            static:
              4xved-FTXA0=main.css.br
              4xved-FTXA0=main.css.gz
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

    let out_wasm = dir.join("target").join("client");

    insta::assert_snapshot!(out_wasm.ls_replace_checksum("<checksum>").unwrap(), @r###"
/client/prebuild-debug.log
/client/prebuild-release.log
/client/wasm32-unknown-unknown/web/debug/static/<checksum>client.js
/client/wasm32-unknown-unknown/web/debug/static/<checksum>client.js.br
/client/wasm32-unknown-unknown/web/debug/static/<checksum>client.js.gz
/client/wasm32-unknown-unknown/web/debug/static/<checksum>client.wasm
/client/wasm32-unknown-unknown/web/debug/static/<checksum>client.wasm
/client/wasm32-unknown-unknown/web/debug/static/<checksum>client.wasm.br
/client/wasm32-unknown-unknown/web/debug/static/<checksum>client.wasm.gz
"###)
}
