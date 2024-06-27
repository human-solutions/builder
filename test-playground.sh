#/bin/sh

echo "Install builder"
cargo install --path=cmd --profile=dev --offline

echo "Build playground"
touch playground/build.rs
cargo build -p playground --target=wasm32-unknown-unknown -r

builder postbuild --manifest-dir=playground --profile=release --package=playground --target-dir=target