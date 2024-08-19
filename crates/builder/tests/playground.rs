mod common;

use camino::Utf8PathBuf;
use common::{cargo, PathExt, Replacer};
use fs_err as fs;

#[test]
fn test_assets() {
    let dir = Utf8PathBuf::from("../../examples/assets-wksp");
    let gen = dir.join("assets").join("gen");

    cargo(&dir, ["clean"]);
    if gen.exists() {
        fs::remove_dir_all(&gen).unwrap();
    }

    cargo(&dir, ["build"]);

    insta::assert_snapshot!(gen.ls_ascii_replace::<NoChange>(0).unwrap(), @r###"
    gen:
      mobile.rs
      web.rs
    "###);

    cargo(&dir, ["build", "--release"]);

    let out = dir.join("target").join("assets");

    insta::assert_snapshot!(out.ls_ascii_replace::<RemoveTargetAndChecksum>(0).unwrap(), @r###"
    assets:
      prebuild-debug.log
      prebuild-release.log
      <target>:
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
}

#[test]
fn test_wasm() {
    let dir = Utf8PathBuf::from("../../examples/wasm-wksp");

    cargo(&dir, ["clean"]);
    cargo(
        &dir,
        ["build", "-p=client", "--target=wasm32-unknown-unknown"],
    );

    cargo(&dir, ["build"]);
    cargo(&dir, ["build", "--release"]);

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

struct NoChange;

impl Replacer for NoChange {
    fn replace(s: &str) -> String {
        s.to_string()
    }
}

struct RemoveTargetAndChecksum;

impl Replacer for RemoveTargetAndChecksum {
    fn replace(s: &str) -> String {
        if s == "aarch64-apple-darwin" || s == "x86_64-unknown-linux-gnu" {
            "<target>".to_string()
        } else if let Some((_, right)) = s.split_once('=') {
            let words = ["main.css", "polyglot.woff2"];
            if words.contains(&right) {
                format!("<checksum>{right}")
            } else {
                s.to_string()
            }
        } else {
            s.to_string()
        }
    }
}
