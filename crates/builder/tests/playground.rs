mod common;

use camino::Utf8PathBuf;
use common::{cargo, PathExt};
use fs_err as fs;

#[test]
fn test_playground() {
    let dir = Utf8PathBuf::from("../../examples/playground");
    let gen = dir.join("assets").join("gen");

    // cargo(&dir, ["clean"]);

    if gen.exists() {
        fs::remove_dir_all(&gen).unwrap();
    }

    cargo(&dir, ["build"]);

    let out = dir.join("target").join("assets");

    insta::assert_snapshot!(out.ls_ascii(0).unwrap(), @r###"
    assets:
      prebuild-debug.log
      mobile:
        debug:
          main.scss
          static:
            hfT-f2u761M=polyglot.woff2
      web:
        debug:
          static:
            My9cobbYRHU=main.scss
            hfT-f2u761M=polyglot.woff2
            badge:
              static:
                badge:
                  MJjU0sjYbCw=apple_store.svg.en
                  MJjU0sjYbCw=apple_store.svg.fr
                  MJjU0sjYbCw=apple_store.svg.fr-CA
    "###);

    insta::assert_snapshot!(gen.ls_ascii(0).unwrap(), @r###"
    gen:
      mobile.rs
      web.rs
    "###);

    cargo(&dir, ["build", "--release"]);
}
